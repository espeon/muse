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
    <main className="flex min-h-dvh items-center justify-center text-sm text-muted-foreground">
      Finishing sign-in…
    </main>
  );
}
