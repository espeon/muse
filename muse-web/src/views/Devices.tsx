import { useState, useEffect } from "react";
import { Link } from "@tanstack/react-router";
import {
  ArrowLeft,
  Globe,
  Loader2,
  Server,
  Smartphone,
  Speaker,
  CheckCircle2,
  Volume2,
  WifiOff,
} from "lucide-react";
import { useRemote } from "@/remote/remote-client";
import { useSession } from "@/lib/auth";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import type { DeviceKind } from "@/remote/protocol";

function deviceIcon(kind: DeviceKind) {
  switch (kind) {
    case "ios":
    case "android":
      return Smartphone;
    case "web":
      return Globe;
    case "server":
      return Server;
  }
}

function kindLabel(kind: DeviceKind): string {
  switch (kind) {
    case "ios":
      return "iPhone / iPad";
    case "android":
      return "Android";
    case "web":
      return "Web";
    case "server":
      return "Server player";
  }
}

/** Format a Unix-ms timestamp as a relative time string. */
function relativeTime(ms: number): string {
  const diff = Date.now() - ms;
  if (diff < 5_000) return "just now";
  if (diff < 60_000) return `${Math.floor(diff / 1000)}s ago`;
  if (diff < 3_600_000) return `${Math.floor(diff / 60_000)}m ago`;
  return `${Math.floor(diff / 3_600_000)}h ago`;
}

/** Hook that re-renders on a fixed interval so relative timestamps stay fresh. */
function useTick(intervalMs = 5000) {
  const [, setTick] = useState(0);
  useEffect(() => {
    const id = setInterval(() => setTick((t) => t + 1), intervalMs);
    return () => clearInterval(id);
  }, [intervalMs]);
}

export function Devices() {
  const remote = useRemote();
  const { isLoggedIn } = useSession();
  const [transferring, setTransferring] = useState<string | null>(null);
  const [transferSuccess, setTransferSuccess] = useState<string | null>(null);
  useTick(5000);

  const handleTransfer = async (deviceId: string) => {
    setTransferring(deviceId);
    setTransferSuccess(null);
    remote.transfer(deviceId);
    // Wait for the WS round-trip; the device list update confirms success.
    // We use a short timeout since the WS response is async.
    setTimeout(() => {
      setTransferring((prev) => (prev === deviceId ? null : prev));
      setTransferSuccess(deviceId);
      setTimeout(() => setTransferSuccess(null), 3000);
    }, 1500);
  };

  if (!isLoggedIn) {
    return (
      <div className="mx-auto max-w-2xl px-4 py-8">
        <p className="text-muted-foreground">Sign in to manage devices.</p>
      </div>
    );
  }

  const activeDevice = remote.devices.find((d) => d.is_active_player);

  // Resolve the "now playing" track from the remote playback state
  const nowPlayingTrackId = remote.lastState?.current_item_id
    ? remote.lastState.queue.find(
        (q) => q.item_id === remote.lastState!.current_item_id,
      )?.track_id
    : null;

  return (
    <div className="mx-auto max-w-2xl px-4 py-8">
      <div className="mb-6 flex items-center gap-3">
        <Link
          to="/settings"
          className="inline-flex h-9 w-9 items-center justify-center rounded-full text-muted-foreground transition-colors hover:bg-accent"
        >
          <ArrowLeft size={20} />
        </Link>
        <h1 className="text-2xl font-bold">Devices</h1>
      </div>

      {/* Connection status banner */}
      {(remote.connectionState === "reconnecting" ||
        remote.connectionState === "failed") && (
        <div
          className={cn(
            "mb-6 flex items-center gap-3 rounded-xl border p-4",
            remote.connectionState === "reconnecting"
              ? "border-yellow-500/20 bg-yellow-500/5"
              : "border-red-500/20 bg-red-500/5",
          )}
        >
          <WifiOff
            size={18}
            className={cn(
              remote.connectionState === "reconnecting"
                ? "text-yellow-500"
                : "text-red-500",
            )}
          />
          <div className="flex-1">
            <p className="text-sm font-medium">
              {remote.connectionState === "reconnecting"
                ? `Reconnecting${remote.reconnectAttempt > 0 ? ` (attempt ${remote.reconnectAttempt})` : ""}…`
                : "Connection failed"}
            </p>
            <p className="text-xs text-muted-foreground">
              {remote.lastError || "Connection lost"}
            </p>
          </div>
          {remote.connectionState === "failed" && (
            <Button size="sm" variant="outline" onClick={() => void remote.retry()}>
              Retry now
            </Button>
          )}
        </div>
      )}

      {/* Now Playing On */}
      <section className="mb-8">
        <h2 className="mb-2 text-sm font-semibold uppercase tracking-wide text-muted-foreground">
          Now Playing On
        </h2>
        <div className="rounded-xl border border-border bg-card p-4">
          {!remote.activeDeviceId ? (
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2 text-muted-foreground">
                <Speaker size={18} />
                <span className="text-sm">Nothing is playing</span>
              </div>
              <Button
                size="sm"
                onClick={() => handleTransfer(remote.myDeviceId)}
                disabled={transferring === remote.myDeviceId}
              >
                {transferring === remote.myDeviceId ? (
                  <Loader2 size={14} className="animate-spin" />
                ) : (
                  "Play here"
                )}
              </Button>
            </div>
          ) : activeDevice ? (
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-3">
                <div className="flex h-10 w-10 items-center justify-center rounded-full bg-green-500/10">
                  <Volume2 size={18} className="text-green-500" />
                </div>
                <div>
                  <p className="text-sm font-medium">{activeDevice.name}</p>
                  <p className="text-xs text-muted-foreground">
                    Active player
                    {nowPlayingTrackId && remote.lastState?.is_playing
                      ? " · Playing"
                      : nowPlayingTrackId
                        ? " · Paused"
                        : ""}
                  </p>
                </div>
              </div>
              {activeDevice.device_id !== remote.myDeviceId && (
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => handleTransfer(remote.myDeviceId)}
                  disabled={transferring === remote.myDeviceId}
                >
                  {transferring === remote.myDeviceId ? (
                    <Loader2 size={14} className="animate-spin" />
                  ) : transferSuccess === remote.myDeviceId ? (
                    <CheckCircle2 size={14} className="text-green-500" />
                  ) : (
                    "Transfer here"
                  )}
                </Button>
              )}
            </div>
          ) : (
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <Loader2 size={14} className="animate-spin" />
              Loading…
            </div>
          )}
        </div>
      </section>

      {/* This Device */}
      <section className="mb-8">
        <h2 className="mb-2 text-sm font-semibold uppercase tracking-wide text-muted-foreground">
          This Device
        </h2>
        <div className="rounded-xl border border-border bg-card p-4">
          <div className="flex items-center gap-3">
            <div
              className={cn(
                "flex h-10 w-10 items-center justify-center rounded-full",
                remote.isActivePlayer
                  ? "bg-green-500/10"
                  : "bg-primary/10",
              )}
            >
              {remote.isActivePlayer ? (
                <Volume2 size={18} className="text-green-500" />
              ) : (
                <Globe size={18} className="text-primary" />
              )}
            </div>
            <div className="min-w-0 flex-1">
              <p className="text-sm font-medium">
                {remote.myDeviceName}
                {remote.isActivePlayer && (
                  <span className="ml-2 text-xs text-green-500">● Active</span>
                )}
              </p>
              <p className="font-mono text-xs text-muted-foreground">
                {remote.myDeviceId.slice(0, 8)}
              </p>
            </div>
          </div>
        </div>
      </section>

      {/* All Devices */}
      <section>
        <h2 className="mb-2 text-sm font-semibold uppercase tracking-wide text-muted-foreground">
          All Devices ({remote.devices.length})
        </h2>
        <div className="space-y-2">
          {remote.devices.length === 0 ? (
            <div className="rounded-xl border border-border bg-card p-4">
              <p className="text-sm text-muted-foreground">No devices connected.</p>
            </div>
          ) : (
            remote.devices.map((device) => {
              const Icon = deviceIcon(device.kind);
              const isSelf = device.device_id === remote.myDeviceId;
              return (
                <div
                  key={device.device_id}
                  className={cn(
                    "flex items-center gap-3 rounded-xl border border-border bg-card p-4 transition-colors",
                    transferSuccess === device.device_id && "border-green-500/30",
                  )}
                >
                  <div
                    className={cn(
                      "flex h-10 w-10 items-center justify-center rounded-full",
                      device.is_active_player
                        ? "bg-green-500/10"
                        : "bg-muted",
                    )}
                  >
                    <Icon
                      size={18}
                      className={device.is_active_player ? "text-green-500" : "text-muted-foreground"}
                    />
                  </div>
                  <div className="min-w-0 flex-1">
                    <p className="truncate text-sm font-medium">
                      {device.name}
                      {isSelf && (
                        <span className="ml-1 text-xs text-muted-foreground">(this device)</span>
                      )}
                    </p>
                    <p className="text-xs text-muted-foreground">
                      {kindLabel(device.kind)} · {relativeTime(device.last_seen)}
                    </p>
                  </div>
                  {device.is_active_player ? (
                    <Volume2 size={16} className="text-green-500" />
                  ) : !isSelf ? (
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={() => handleTransfer(device.device_id)}
                      disabled={transferring === device.device_id}
                    >
                      {transferring === device.device_id ? (
                        <Loader2 size={14} className="animate-spin" />
                      ) : transferSuccess === device.device_id ? (
                        <CheckCircle2 size={14} className="text-green-500" />
                      ) : (
                        "Transfer"
                      )}
                    </Button>
                  ) : null}
                </div>
              );
            })
          )}
        </div>
      </section>
    </div>
  );
}
