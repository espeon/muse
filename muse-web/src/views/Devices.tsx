import { useState } from "react";
import { Link } from "@tanstack/react-router";
import {
  ArrowLeft,
  Globe,
  Server,
  Smartphone,
  Speaker,
  Volume2,
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

export function Devices() {
  const remote = useRemote();
  const { isLoggedIn } = useSession();
  const [transferring, setTransferring] = useState<string | null>(null);

  if (!isLoggedIn) {
    return (
      <div className="mx-auto max-w-2xl px-4 py-8">
        <p className="text-muted-foreground">Sign in to manage devices.</p>
      </div>
    );
  }

  const activeDevice = remote.devices.find((d) => d.is_active_player);

  const handleTransfer = async (deviceId: string) => {
    setTransferring(deviceId);
    remote.transfer(deviceId);
    // Clear after a short delay — the WS round-trip updates the state
    setTimeout(() => setTransferring(null), 2000);
  };

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
              <Button size="sm" onClick={() => handleTransfer(remote.myDeviceId)}>
                Play on this device
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
                  <p className="text-xs text-muted-foreground">Active player</p>
                </div>
              </div>
              {activeDevice.device_id !== remote.myDeviceId && (
                <Button
                  size="sm"
                  variant="outline"
                  onClick={() => handleTransfer(remote.myDeviceId)}
                  disabled={transferring === remote.myDeviceId}
                >
                  Transfer to this device
                </Button>
              )}
            </div>
          ) : (
            <p className="text-sm text-muted-foreground">Loading…</p>
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
            <div className="flex h-10 w-10 items-center justify-center rounded-full bg-primary/10">
              <Globe size={18} className="text-primary" />
            </div>
            <div>
              <p className="text-sm font-medium">{remote.myDeviceName}</p>
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
                  className="flex items-center gap-3 rounded-xl border border-border bg-card p-4"
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
                      {kindLabel(device.kind)}
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
                      Transfer
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
