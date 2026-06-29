import { useSyncExternalStore } from "react";
import { MAKI_URL } from "@/lib/config";

/**
 * OIDC session for the SPA.
 *
 * Flow: `login()` navigates to `${MAKI}/auth/login?platform=spa`. Maki 302s to
 * the OIDC provider; after auth the provider returns to Maki's callback, which
 * (spa mode) 302s back to this app's `/auth/callback` with the token pair in
 * the URL *fragment*. `consumeCallbackFragment()` reads and stores them.
 *
 * Token transport (see maki/src/api/middleware/jwt.rs): API requests send
 * `Authorization: Bearer authjs.session-token:<JWE>` — the cookie name, a colon,
 * then the JWE. Refresh sends `Bearer <refreshToken>` (raw). The session JWE is
 * short-lived (1h); refresh is long-lived (4w) and exchanged via `/auth/refresh`.
 */

const SESSION_KEY = "muse-web.session";

export interface Session {
  /** Session JWE. */
  sessionToken: string;
  /** Unix seconds. */
  sessionExpiry: number;
  /** Opaque refresh token. */
  refreshToken: string;
  /** Unix seconds. */
  refreshExpiry: number;
}

function loadSession(): Session | null {
  try {
    const raw = localStorage.getItem(SESSION_KEY);
    return raw ? (JSON.parse(raw) as Session) : null;
  } catch {
    return null;
  }
}

const listeners = new Set<() => void>();
let currentSession = loadSession();

function saveSession(s: Session | null) {
  currentSession = s;
  try {
    if (s) localStorage.setItem(SESSION_KEY, JSON.stringify(s));
    else localStorage.removeItem(SESSION_KEY);
  } catch {
    /* storage unavailable */
  }
  listeners.forEach((l) => l());
}

export function isLoggedIn(): boolean {
  return currentSession != null;
}

/** Begin OIDC login. Navigates away to Maki → provider → back to /auth/callback.
 *  Passes this origin as `redirect_uri`; Maki validates it against WEB_ORIGINS. */
export function login() {
  const redirectUri = window.location.origin;
  window.location.href = `${MAKI_URL}/auth/login?platform=spa&redirect_uri=${encodeURIComponent(redirectUri)}`;
}

/** Clear local session; best-effort revoke the refresh token at Maki. */
export function logout() {
  const s = currentSession;
  saveSession(null);
  if (s) {
    void fetch(`${MAKI_URL}/auth/logout`, {
      method: "POST",
      headers: { Authorization: `Bearer ${s.refreshToken}` },
    }).catch(() => {
      /* best-effort */
    });
  }
}

let refreshing: Promise<string | null> | null = null;

async function refreshSession(s: Session): Promise<string | null> {
  const res = await fetch(`${MAKI_URL}/auth/refresh`, {
    method: "POST",
    headers: { Authorization: `Bearer ${s.refreshToken}` },
  });
  if (!res.ok) {
    if (res.status === 401) saveSession(null); // refresh token invalid/expired
    return null;
  }
  const data = (await res.json()) as { session_token: string; expiry: number };
  saveSession({ ...s, sessionToken: data.session_token, sessionExpiry: data.expiry });
  return data.session_token;
}

/** A valid session JWE, refreshing transparently when within 60s of expiry.
 *  Returns null if there is no session (or refresh failed). */
export async function getValidSessionToken(): Promise<string | null> {
  const s = currentSession;
  if (!s) return null;
  const now = Math.floor(Date.now() / 1000);
  if (now < s.sessionExpiry - 60) return s.sessionToken;
  if (!refreshing) {
    refreshing = refreshSession(s).finally(() => {
      refreshing = null;
    });
  }
  return refreshing;
}

/** Full `name:token` value for the Authorization Bearer header, or null. */
export async function getAuthHeader(): Promise<string | null> {
  const token = await getValidSessionToken();
  return token ? `authjs.session-token:${token}` : null;
}

/** Read & store the token pair from the /auth/callback fragment. Returns success. */
export function consumeCallbackFragment(): boolean {
  const hash = window.location.hash.replace(/^#/, "");
  if (!hash) return false;
  const params = new URLSearchParams(hash);
  const sessionToken = params.get("session_token");
  const refreshToken = params.get("refresh_token");
  if (!sessionToken || !refreshToken) return false;
  saveSession({
    sessionToken,
    sessionExpiry: Number(params.get("session_expiry") ?? 0),
    refreshToken,
    refreshExpiry: Number(params.get("refresh_expiry") ?? 0),
  });
  // Strip the fragment so tokens aren't left visible / re-consumed on refresh.
  history.replaceState(null, "", window.location.pathname);
  return true;
}

function subscribe(cb: () => void): () => void {
  listeners.add(cb);
  return () => {
    listeners.delete(cb);
  };
}

/** React hook: re-renders on login/logout/token-refresh. */
export function useSession() {
  useSyncExternalStore(subscribe, () => currentSession, () => currentSession);
  return { session: currentSession, isLoggedIn: currentSession != null };
}
