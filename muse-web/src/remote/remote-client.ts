import { useSyncExternalStore } from "react";
import { MAKI_URL } from "@/lib/config";
import { getAuthHeader } from "@/lib/auth";
import type {
  ClientMessage,
  RemoteCommand,
  RemoteDevice,
  RemotePlaybackState,
  ServerMessage,
} from "./protocol";

// --------------------------------------------------------------- state

export type ConnectionState =
  | "disconnected"
  | "connecting"
  | "connected"
  | "failed";

export interface RemoteSnapshot {
  connectionState: ConnectionState;
  myDeviceId: string;
  myDeviceName: string;
  activeDeviceId: string | null;
  devices: RemoteDevice[];
  lastState: RemotePlaybackState | null;
  lastError: string | null;
  isActivePlayer: boolean;
}

const DEVICE_ID_KEY = "muse-web.remoteDeviceId";

function loadOrCreateDeviceId(): string {
  try {
    const existing = localStorage.getItem(DEVICE_ID_KEY);
    if (existing) return existing;
  } catch {
    /* storage unavailable */
  }
  const id = crypto.randomUUID();
  try {
    localStorage.setItem(DEVICE_ID_KEY, id);
  } catch {
    /* storage unavailable */
  }
  return id;
}

function getDeviceName(): string {
  const ua = typeof navigator !== "undefined" ? navigator.userAgent : "";
  if (/Mobi|Android/i.test(ua)) return "Mobile browser";
  if (/Macintosh|MacIntel/i.test(ua)) return "Mac browser";
  if (/Windows|Win32/i.test(ua)) return "Windows browser";
  if (/Linux/i.test(ua)) return "Linux browser";
  return "Web browser";
}

// --------------------------------------------------------------- client

class RemoteClient {
  private ws: WebSocket | null = null;
  private heartbeatTimer: ReturnType<typeof setInterval> | null = null;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private reconnectAttempt = 0;
  private explicitShutdown = false;

  private myDeviceId = loadOrCreateDeviceId();
  private myDeviceName = getDeviceName();

  private connectionState: ConnectionState = "disconnected";
  private activeDeviceId: string | null = null;
  private devices: RemoteDevice[] = [];
  private lastState: RemotePlaybackState | null = null;
  private lastError: string | null = null;
  private lastSeq = 0;

  /** Cached snapshot — rebuilt only when state changes, so React's
   *  useSyncExternalStore doesn't see a new reference every render. */
  private cachedSnapshot: RemoteSnapshot | null = null;

  /** Hook for the player engine to receive routed commands. */
  onIncomingCommand: ((command: RemoteCommand) => void) | null = null;
  /** Hook for the player engine to know when active-player status changes. */
  onActivePlayerChanged: ((isActive: boolean) => void) | null = null;

  private listeners = new Set<() => void>();

  // --- snapshot

  private buildSnapshot(): RemoteSnapshot {
    return {
      connectionState: this.connectionState,
      myDeviceId: this.myDeviceId,
      myDeviceName: this.myDeviceName,
      activeDeviceId: this.activeDeviceId,
      devices: this.devices,
      lastState: this.lastState,
      lastError: this.lastError,
      isActivePlayer: this.activeDeviceId === this.myDeviceId,
    };
  }

  get isActivePlayer(): boolean {
    return this.activeDeviceId === this.myDeviceId;
  }

  private emit() {
    this.cachedSnapshot = null; // invalidate cache
    this.listeners.forEach((l) => l());
  }

  // --- lifecycle

  /** Open the connection. Idempotent. */
  async start() {
    if (this.connectionState === "connecting" || this.connectionState === "connected") return;
    this.explicitShutdown = false;
    await this.connect();
  }

  /** Close and don't reconnect. */
  shutdown() {
    this.explicitShutdown = true;
    this.cleanupTimers();
    if (this.ws) {
      try {
        this.ws.close(1000, "shutdown");
      } catch {
        /* noop */
      }
      this.ws = null;
    }
    this.connectionState = "disconnected";
    this.activeDeviceId = null;
    this.devices = [];
    this.lastState = null;
    this.lastSeq = 0;
    this.emit();
  }

  private cleanupTimers() {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer);
      this.heartbeatTimer = null;
    }
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
  }

  private async connect() {
    this.connectionState = "connecting";
    this.emit();

    const auth = await getAuthHeader();
    if (!auth) {
      this.connectionState = "failed";
      this.lastError = "Not authenticated";
      this.emit();
      return;
    }

    const wsUrl = this.makeWebSocketUrl(MAKI_URL);
    if (!wsUrl) {
      this.connectionState = "failed";
      this.lastError = "Invalid server URL";
      this.emit();
      return;
    }

    // Pass auth as a query param since WebSocket doesn't support custom headers
    // from the browser. Maki's WS handler reads the token from the query string.
    const url = `${wsUrl}?token=${encodeURIComponent(auth)}`;

    let ws: WebSocket;
    try {
      ws = new WebSocket(url);
    } catch (e) {
      this.handleDisconnect(e instanceof Error ? e.message : String(e));
      return;
    }
    this.ws = ws;

    ws.onopen = () => {
      // Send IDENTIFY
      this.send({
        type: "identify",
        device_id: this.myDeviceId,
        name: this.myDeviceName,
        kind: "web",
      });
      // Start heartbeat
      this.heartbeatTimer = setInterval(() => {
        this.send({ type: "heartbeat" });
      }, 10_000);
    };

    ws.onmessage = (ev) => {
      this.handleMessage(ev.data);
    };

    ws.onerror = () => {
      // The close event will follow; handle reconnect there.
    };

    ws.onclose = (ev) => {
      this.handleDisconnect(
        ev.reason || `WebSocket closed (${ev.code})`,
      );
    };
  }

  private makeWebSocketUrl(serverUrl: string): string | null {
    let url: URL;
    try {
      url = new URL(serverUrl);
    } catch {
      return null;
    }
    const scheme = url.protocol === "https:" ? "wss" : "ws";
    return `${scheme}://${url.host}/api/v1/remote/ws`;
  }

  private handleMessage(raw: string) {
    let msg: ServerMessage;
    try {
      msg = JSON.parse(raw) as ServerMessage;
    } catch {
      return;
    }
    this.applyServerMessage(msg);
  }

  private applyServerMessage(msg: ServerMessage) {
    switch (msg.type) {
      case "welcome": {
        this.myDeviceId = msg.your_device_id;
        const prevActive = this.activeDeviceId;
        this.activeDeviceId = msg.active_device_id;
        this.devices = msg.devices;
        this.lastState = msg.last_state;
        this.lastSeq = 0;
        this.connectionState = "connected";
        this.reconnectAttempt = 0;
        this.lastError = null;
        if (prevActive !== this.activeDeviceId) {
          this.onActivePlayerChanged?.(this.isActivePlayer);
        }
        this.emit();
        break;
      }
      case "state": {
        if (msg.seq <= this.lastSeq && this.lastSeq !== 0) return;
        this.lastState = msg.state;
        this.lastSeq = msg.seq;
        this.emit();
        break;
      }
      case "device_list": {
        const prevActive = this.activeDeviceId;
        this.activeDeviceId = msg.active_device_id;
        this.devices = msg.devices;
        if (prevActive !== this.activeDeviceId) {
          this.onActivePlayerChanged?.(this.isActivePlayer);
        }
        this.emit();
        break;
      }
      case "command": {
        this.onIncomingCommand?.(msg.command);
        break;
      }
      case "request_publish": {
        // Server wants the active player to re-publish. The player engine
        // handles this via its periodic publish timer.
        break;
      }
      case "error": {
        this.lastError = msg.message || msg.code || "Unknown error";
        this.emit();
        break;
      }
    }
  }

  private handleDisconnect(reason: string) {
    this.cleanupTimers();
    this.ws = null;
    if (this.explicitShutdown) return;
    this.connectionState = "failed";
    this.lastError = reason;
    this.emit();
    this.scheduleReconnect();
  }

  private scheduleReconnect() {
    if (this.explicitShutdown) return;
    const backoff = Math.min(30, 1 << Math.min(this.reconnectAttempt, 5));
    this.reconnectAttempt++;
    this.reconnectTimer = setTimeout(() => {
      if (!this.explicitShutdown) void this.connect();
    }, backoff * 1000);
  }

  // --- send

  private send(msg: ClientMessage) {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) return;
    try {
      this.ws.send(JSON.stringify(msg));
    } catch {
      // The close handler will trigger reconnect.
    }
  }

  /** Publish this device's playback state (only meaningful when active player). */
  publishState(state: RemotePlaybackState) {
    this.send({ type: "publish_state", state });
  }

  /** Send a command to the active player. */
  sendCommand(command: RemoteCommand) {
    this.send({ type: "command", command });
  }

  /** Request that a device become the active player. */
  transfer(deviceId: string) {
    this.send({ type: "transfer", to_device_id: deviceId });
  }

  // --- React binding

  subscribe = (cb: () => void): (() => void) => {
    this.listeners.add(cb);
    return () => {
      this.listeners.delete(cb);
    };
  };

  snapshot = (): RemoteSnapshot => {
    if (!this.cachedSnapshot) {
      this.cachedSnapshot = this.buildSnapshot();
    }
    return this.cachedSnapshot;
  };
}

// --------------------------------------------------------------- singleton

export const remoteClient = new RemoteClient();

/** React hook for the remote client. */
export function useRemote(): RemoteSnapshot & {
  start: () => void;
  shutdown: () => void;
  transfer: (deviceId: string) => void;
  sendCommand: (command: RemoteCommand) => void;
} {
  useSyncExternalStore(remoteClient.subscribe, remoteClient.snapshot, remoteClient.snapshot);
  return {
    ...remoteClient.snapshot(),
    start: () => void remoteClient.start(),
    shutdown: () => remoteClient.shutdown(),
    transfer: (id: string) => remoteClient.transfer(id),
    sendCommand: (cmd: RemoteCommand) => remoteClient.sendCommand(cmd),
  };
}
