/**
 * Remote control protocol types — mirror maki's remote/WS protocol
 * (maki/src/api/remote/protocol.rs) and the iOS RemoteModels.swift.
 *
 * Wire format uses snake_case with tagged unions discriminated by `type`
 * (client/server messages) and `kind` (commands). All timestamps are
 * Unix milliseconds.
 */

// --------------------------------------------------------------- devices

export type DeviceKind = "ios" | "android" | "web" | "server";

export interface RemoteDevice {
  device_id: string;
  name: string;
  kind: DeviceKind;
  is_active_player: boolean;
  last_seen: number;
}

// --------------------------------------------------------------- queue + state

export interface RemoteQueueItem {
  item_id: string;
  track_id: number;
}

export interface RemotePlaybackState {
  current_item_id: string | null;
  position_ms: number;
  is_playing: boolean;
  queue: RemoteQueueItem[];
  updated_at: number;
}

// --------------------------------------------------------------- commands (outbound)

export type RemoteCommand =
  | { kind: "play" }
  | { kind: "pause" }
  | { kind: "toggle" }
  | { kind: "next" }
  | { kind: "previous" }
  | { kind: "seek"; position_ms: number }
  | { kind: "set_queue"; track_ids: number[]; start_index: number }
  | { kind: "add_to_queue"; track_id: number; after_item_id?: string }
  | { kind: "remove_from_queue"; item_id: string }
  | { kind: "reorder_queue"; item_id: string; after_item_id?: string };

// --------------------------------------------------------------- client messages (outbound)

export type ClientMessage =
  | {
      type: "identify";
      device_id: string;
      name: string;
      kind: DeviceKind;
    }
  | { type: "publish_state"; state: RemotePlaybackState }
  | { type: "command"; command: RemoteCommand }
  | { type: "transfer"; to_device_id: string }
  | { type: "heartbeat" };

// --------------------------------------------------------------- server messages (inbound)

export type ServerMessage =
  | {
      type: "welcome";
      your_device_id: string;
      active_device_id: string | null;
      devices: RemoteDevice[];
      last_state: RemotePlaybackState | null;
    }
  | {
      type: "state";
      state: RemotePlaybackState;
      from_device_id: string;
      seq: number;
    }
  | {
      type: "device_list";
      devices: RemoteDevice[];
      active_device_id: string | null;
    }
  | {
      type: "command";
      command: RemoteCommand;
      from_device_id: string;
    }
  | { type: "request_publish" }
  | { type: "error"; code: string; message: string };
