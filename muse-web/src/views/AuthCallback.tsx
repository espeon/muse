import { useEffect } from "react";
import { useNavigate } from "@tanstack/react-router";
import { consumeCallbackFragment } from "@/lib/auth";

/** Reads the token pair from the URL fragment (set by Maki's spa callback mode)
 *  and redirects home. */
export function AuthCallback() {
  const navigate = useNavigate();
  useEffect(() => {
    consumeCallbackFragment();
    void navigate({ to: "/", replace: true });
  }, [navigate]);
  return (
    <main className="flex min-h-dvh flex-col items-center justify-center gap-4">
      <div className="flex h-12 w-12 items-center justify-center rounded-2xl bg-gradient-to-br from-neutral-800 to-neutral-900 shadow-2xl shadow-black/50">
        <svg viewBox="0 0 512 512" class="h-7 w-7" fill="url(#cbNote)" xmlns="http://www.w3.org/2000/svg">
          <defs>
            <linearGradient id="cbNote" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0" stop-color="#f5f5f5"/>
              <stop offset="1" stop-color="#a0a0a0"/>
            </linearGradient>
          </defs>
          <g>
            <rect x="290" y="96" width="28" height="260" rx="4"/>
            <ellipse cx="248" cy="356" rx="56" ry="44" transform="rotate(-20 248 356)"/>
            <path d="M318 96 C 390 140, 390 200, 318 240 L 318 200 C 360 170, 360 130, 318 96 Z"/>
          </g>
        </svg>
      </div>
      <div className="flex items-center gap-2 text-sm text-muted-foreground">
        <span className="inline-block h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent" />
        Finishing sign-in…
      </div>
    </main>
  );
}
