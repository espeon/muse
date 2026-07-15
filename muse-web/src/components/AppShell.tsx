import { Link, Outlet } from "@tanstack/react-router";
import {
  Home as HomeIcon,
  Library,
  LogOut,
  Search,
  Settings as SettingsIcon,
} from "lucide-react";
import { PlayerBar } from "@/components/PlayerBar";
import { ConnectionBanner } from "@/components/ConnectionBanner";
import { TheatreMode } from "@/components/TheatreMode";
import { Button } from "@/components/ui/button";
import { login, logout, useSession } from "@/lib/auth";
import { cn } from "@/lib/utils";
import { Glass } from "@samasante/liquid-glass";
import { useState } from "react";

const navItems = [
  { to: "/", label: "Home", icon: HomeIcon },
  { to: "/search", label: "Search", icon: Search },
  { to: "/library", label: "Library", icon: Library },
  { to: "/settings", label: "Settings", icon: SettingsIcon },
] as const;

const linkClass =
  "flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground [&[data-status=active]]:bg-accent [&[data-status=active]]:text-primary-foreground [&[data-status=active]]:bg-primary/90";

/** App chrome: persistent sidebar (desktop), bottom nav (mobile), global
 *  sign-in/out, and the always-present player bar. Page content via <Outlet/>. */
export function AppShell() {
  const { isLoggedIn } = useSession();
  const [theatreOpen, setTheatreOpen] = useState(false);

  return (
    <div className="min-h-dvh">
      <ConnectionBanner />
      <Glass
        className="fixed inset-y-0 left-0 z-40 hidden w-60 flex-col border-r border-border bg-card p-3 m-3 md:flex"
        style={{
          borderRadius: "1rem",
        }}
        optics={{
          mapSize: 512,
          clipToShape: true,
          softEdge: true,
          strength: 0.18,
          depth: 0.2,
          curvature: 0.55,
          bend: 0.25,
          bendWidth: 0.08,
          dispersion: 0.15,
          specular: 0.5,
          sheenAngle: 50,
          glow: 0.15,
          glowSpread: 1,
          glowFalloff: 1.5,
          sheen: 0.95,
          sheenWidth: 2,
          sheenFalloff: 1.5,
          frost: 6,
          brightness: 0.05,
        }}
      >
        <div className="px-3 py-4">
          <span className="text-xl font-bold tracking-tight text-primary">
            muse
          </span>
        </div>
        <nav className="flex flex-col gap-0.5">
          {navItems.map((item) => (
            <Link key={item.to} to={item.to} className={linkClass}>
              <item.icon size={20} />
              {item.label}
            </Link>
          ))}
        </nav>
        <div className="mt-auto p-1">
          {isLoggedIn ? (
            <Button
              variant="ghost"
              size="sm"
              onClick={logout}
              className="w-full justify-start gap-3 px-3 text-muted-foreground"
            >
              <LogOut size={18} />
              Sign out
            </Button>
          ) : (
            <Button
              variant="outline"
              size="sm"
              onClick={login}
              className="w-full"
            >
              Sign in
            </Button>
          )}
        </div>
      </Glass>

      <main className="pb-48 md:pb-32 md:pl-60">
        <div className="mx-auto max-w-screen-2xl lg:px-24 px-6">
          <Outlet />
        </div>
      </main>

      {/* mobile bottom nav (sits above the floating player bar) */}
      <nav className="fixed inset-x-0 bottom-[76px] z-40 flex justify-around border-t border-border bg-card/95 backdrop-blur md:hidden">
        {navItems.map((item) => (
          <Link
            key={item.to}
            to={item.to}
            className={cn(
              "flex flex-1 flex-col items-center gap-0.5 py-2 text-[11px] text-muted-foreground",
              "[&[data-status=active]]:text-primary",
            )}
          >
            <item.icon size={20} />
            {item.label}
          </Link>
        ))}
      </nav>

      <PlayerBar onOpenTheatre={() => setTheatreOpen(true)} />
      {theatreOpen && <TheatreMode onClose={() => setTheatreOpen(false)} />}
    </div>
  );
}
