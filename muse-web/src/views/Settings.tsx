import { useState, useEffect, useCallback } from "react";
import { Link } from "@tanstack/react-router";
import { ArrowLeft, CheckCircle2, ChevronRight, Loader2, XCircle } from "lucide-react";
import {
  completeLastfmSession,
  disconnectLastfm,
  fetchLastfmToken,
  fetchMe,
} from "@/lib/api";
import { logout, useSession } from "@/lib/auth";
import { usePlayer } from "@/player/use-player";
import { useRemote } from "@/remote/remote-client";
import { Button } from "@/components/ui/button";
import { Toggle } from "@/components/ui/toggle";
import { cn } from "@/lib/utils";

type PairingStep = "idle" | "awaitingApproval" | "connecting";

// --- localStorage helpers for config ---
const LLM_API_KEY_KEY = "muse-web.llm.apiKey";
const LLM_BASE_URL_KEY = "muse-web.llm.baseURL";
const LLM_MODEL_KEY = "muse-web.llm.model";

function loadString(key: string, fallback: string): string {
  try {
    return localStorage.getItem(key) ?? fallback;
  } catch {
    return fallback;
  }
}

export function Settings() {
  const { isLoggedIn } = useSession();
  const remote = useRemote();
  const player = usePlayer();

  // Last.fm
  const [lastfmConnected, setLastfmConnected] = useState(false);
  const [lastfmUsername, setLastfmUsername] = useState<string | null>(null);
  const [pairingStep, setPairingStep] = useState<PairingStep>("idle");
  const [error, setError] = useState<string | null>(null);
  const [pendingToken, setPendingToken] = useState<string | null>(null);

  // Config
  const [llmApiKey, setLlmApiKey] = useState(() => loadString(LLM_API_KEY_KEY, ""));
  const [llmBaseUrl, setLlmBaseUrl] = useState(() =>
    loadString(LLM_BASE_URL_KEY, "https://openrouter.ai/api/v1"),
  );
  const [llmModel, setLlmModel] = useState(() =>
    loadString(LLM_MODEL_KEY, "openai/gpt-4o-mini"),
  );

  useEffect(() => {
    if (!isLoggedIn) return;
    let cancelled = false;
    fetchMe()
      .then((me) => {
        if (cancelled) return;
        setLastfmConnected(me.lastfm_connected);
      })
      .catch(() => {});
    return () => {
      cancelled = true;
    };
  }, [isLoggedIn]);

  const handleConnect = useCallback(async () => {
    setError(null);
    try {
      const resp = await fetchLastfmToken();
      setPendingToken(resp.token);
      setPairingStep("awaitingApproval");
      window.open(resp.url, "_blank");
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      setPairingStep("idle");
    }
  }, []);

  const handleComplete = useCallback(async () => {
    if (!pendingToken) {
      setPairingStep("idle");
      return;
    }
    setPairingStep("connecting");
    setError(null);
    try {
      const resp = await completeLastfmSession(pendingToken);
      setLastfmConnected(true);
      setLastfmUsername(resp.username);
      setPendingToken(null);
      setPairingStep("idle");
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      setPairingStep("awaitingApproval");
    }
  }, [pendingToken]);

  const handleDisconnect = useCallback(async () => {
    setError(null);
    try {
      await disconnectLastfm();
      setLastfmConnected(false);
      setLastfmUsername(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }, []);

  if (!isLoggedIn) {
    return (
      <div className="mx-auto max-w-2xl px-4 py-8">
        <p className="text-muted-foreground">Sign in to access settings.</p>
      </div>
    );
  }

  return (
    <div className="mx-auto max-w-2xl px-4 py-8">
      <div className="mb-6 flex items-center gap-3">
        <Link
          to="/"
          className="inline-flex h-9 w-9 items-center justify-center rounded-full text-muted-foreground transition-colors hover:bg-accent"
        >
          <ArrowLeft size={20} />
        </Link>
        <h1 className="text-2xl font-bold">Settings</h1>
      </div>

      {/* Account */}
      <section className="mb-8">
        <h2 className="mb-3 text-sm font-semibold uppercase tracking-wide text-muted-foreground">
          Account
        </h2>
        <div className="rounded-xl border border-border bg-card p-4">
          <Button variant="destructive" size="sm" onClick={logout}>
            Sign out
          </Button>
        </div>
      </section>

      {/* Devices / Remote */}
      <section className="mb-8">
        <h2 className="mb-3 text-sm font-semibold uppercase tracking-wide text-muted-foreground">
          Devices
        </h2>
        <div className="rounded-xl border border-border bg-card p-4">
          <div className="mb-3 flex items-center justify-between">
            <span className="text-sm text-muted-foreground">Connection</span>
            <ConnectionBadge state={remote.connectionState} />
          </div>
          {!remote.activeDeviceId && (
            <Button
              size="sm"
              variant="outline"
              className="mb-3"
              onClick={() => remote.transfer(remote.myDeviceId)}
            >
              Play on this device
            </Button>
          )}
          <Link
            to="/devices"
            className="flex items-center justify-between rounded-lg px-2 py-2 transition-colors hover:bg-accent"
          >
            <span className="text-sm font-medium">Manage Devices</span>
            <div className="flex items-center gap-2 text-muted-foreground">
              <span className="text-xs">{remote.devices.length}</span>
              <ChevronRight size={16} />
            </div>
          </Link>
        </div>
      </section>

      {/* Last.fm */}
      <section className="mb-8">
        <h2 className="mb-3 text-sm font-semibold uppercase tracking-wide text-muted-foreground">
          Last.fm
        </h2>
        <div className="rounded-xl border border-border bg-card p-4">
          {lastfmConnected ? (
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <CheckCircle2 size={18} className="text-green-500" />
                <div>
                  <p className="text-sm font-medium">
                    Connected{lastfmUsername ? ` as ${lastfmUsername}` : ""}
                  </p>
                  <p className="text-xs text-muted-foreground">
                    Scrobbling is active — plays are sent automatically.
                  </p>
                </div>
              </div>
              <Button variant="outline" size="sm" onClick={handleDisconnect}>
                Disconnect
              </Button>
            </div>
          ) : pairingStep === "awaitingApproval" || pairingStep === "connecting" ? (
            <div>
              <div className="mb-3 flex items-center gap-2">
                <Loader2
                  size={18}
                  className={cn(
                    "animate-spin",
                    pairingStep === "connecting" ? "text-primary" : "text-muted-foreground",
                  )}
                />
                <p className="text-sm font-medium">
                  {pairingStep === "connecting" ? "Connecting…" : "Waiting for approval"}
                </p>
              </div>
              <p className="mb-3 text-xs text-muted-foreground">
                Approve the connection on last.fm, then click below to finish.
              </p>
              <Button size="sm" onClick={handleComplete} disabled={pairingStep === "connecting"}>
                I've approved — finish
              </Button>
            </div>
          ) : (
            <div>
              <p className="mb-3 text-sm text-muted-foreground">
                Connect your Last.fm account to scrobble plays automatically.
              </p>
              <Button size="sm" onClick={handleConnect}>
                Connect Last.fm
              </Button>
            </div>
          )}
          {error && (
            <div className="mt-3 flex items-center gap-2 text-sm text-red-500">
              <XCircle size={14} />
              <span>{error}</span>
            </div>
          )}
        </div>
      </section>

      {/* Adaptive Streaming */}
      <section className="mb-8">
        <h2 className="mb-3 text-sm font-semibold uppercase tracking-wide text-muted-foreground">
          Playback
        </h2>
        <div className="rounded-xl border border-border bg-card p-4">
          <label className="flex items-center justify-between">
            <div className="pr-4">
              <p className="text-sm font-medium">Adaptive Streaming</p>
              <p className="text-xs text-muted-foreground">
                Uses HLS with adaptive bitrate. Takes effect on next track.
              </p>
            </div>
            <Toggle
              pressed={player.getUseHLS()}
              onPressedChange={(pressed) => player.setUseHLS(pressed)}
              aria-label="Adaptive streaming"
            />
          </label>

          {player.getUseHLS() && player.getHlsProfiles().length > 0 && (
            <div className="mt-3">
              <label className="mb-1 block text-sm font-medium">Quality</label>
              <select
                value={player.getSelectedProfile() ?? "auto"}
                onChange={(e) =>
                  player.setSelectedProfile(
                    e.target.value === "auto" ? null : e.target.value,
                  )
                }
                className="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm outline-none transition-colors focus:border-primary"
              >
                <option value="auto">Auto (ABR)</option>
                {player.getHlsProfiles().map((profile) => (
                  <option key={profile.name} value={profile.name}>
                    {profile.name}
                    {profile.bitrate ? ` (${Math.round(profile.bitrate / 1000)}k)` : " (Lossless)"}
                  </option>
                ))}
              </select>
            </div>
          )}
        </div>
      </section>

      {/* LLM Translation */}
      <section className="mb-8">
        <h2 className="mb-3 text-sm font-semibold uppercase tracking-wide text-muted-foreground">
          LLM Translation
        </h2>
        <div className="space-y-3 rounded-xl border border-border bg-card p-4">
          <ConfigInput
            label="API Key"
            type="password"
            placeholder="sk-or-v1-..."
            value={llmApiKey}
            onChange={(v) => {
              setLlmApiKey(v);
              try { localStorage.setItem(LLM_API_KEY_KEY, v); } catch { /* noop */ }
            }}
          />
          <ConfigInput
            label="Base URL"
            placeholder="https://openrouter.ai/api/v1"
            value={llmBaseUrl}
            onChange={(v) => {
              setLlmBaseUrl(v);
              try { localStorage.setItem(LLM_BASE_URL_KEY, v); } catch { /* noop */ }
            }}
            hint="OpenRouter, OpenAI, or any OpenAI-compatible provider"
          />
          <ConfigInput
            label="Model"
            placeholder="openai/gpt-4o-mini"
            value={llmModel}
            onChange={(v) => {
              setLlmModel(v);
              try { localStorage.setItem(LLM_MODEL_KEY, v); } catch { /* noop */ }
            }}
          />
        </div>
      </section>
    </div>
  );
}

function ConnectionBadge({ state }: { state: string }) {
  if (state === "connected") {
    return (
      <span className="flex items-center gap-1.5 text-sm text-green-500">
        <CheckCircle2 size={14} />
        Connected
      </span>
    );
  }
  if (state === "connecting") {
    return (
      <span className="flex items-center gap-1.5 text-sm text-muted-foreground">
        <Loader2 size={14} className="animate-spin" />
        Connecting…
      </span>
    );
  }
  if (state === "failed") {
    return <span className="text-sm text-red-500">Disconnected</span>;
  }
  return <span className="text-sm text-muted-foreground">Disconnected</span>;
}

function ConfigInput({
  label,
  value,
  onChange,
  placeholder,
  type = "text",
  hint,
}: {
  label: string;
  value: string;
  onChange: (v: string) => void;
  placeholder?: string;
  type?: string;
  hint?: string;
}) {
  return (
    <div>
      <label className="mb-1 block text-sm font-medium">{label}</label>
      <input
        type={type}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder}
        className="w-full rounded-lg border border-border bg-background px-3 py-2 text-sm outline-none transition-colors focus:border-primary"
      />
      {hint && <p className="mt-1 text-xs text-muted-foreground">{hint}</p>}
    </div>
  );
}
