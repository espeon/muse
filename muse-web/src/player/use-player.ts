import { useSyncExternalStore } from "react";
import {
  resolveTrackUrl,
  setPlaying,
  scrobbleTrack,
  fetchHlsProfiles,
  signTrackHls,
} from "@/lib/api";
import { GaplessQueue } from "./queue";
import { GaplessTrack } from "./track";
import { HlsTrack } from "./hls-track";
import {
  setMediaHandlers,
  setMediaMetadata,
  setMediaPlaybackState,
  setMediaPositionState,
} from "./media-session";
import { remoteClient } from "@/remote/remote-client";
import type { RemoteCommand, RemotePlaybackState } from "@/remote/protocol";
import type { HlsProfile } from "@/types";
import type { TrackInfo, TrackState } from "./types";

export interface PlayerSnapshot {
  currentIndex: number;
  current: TrackInfo | null;
  isPlaying: boolean;
  currentTime: number;
  duration: number;
  /** How much of the track has been decoded/buffered (seconds). */
  bufferedTime: number;
  length: number;
  shuffle: boolean;
  volume: number;
  queue: TrackInfo[];
}

const EMPTY: PlayerSnapshot = {
  currentIndex: -1,
  current: null,
  isPlaying: false,
  currentTime: 0,
  duration: 0,
  bufferedTime: 0,
  length: 0,
  shuffle: false,
  volume: 1,
  queue: [],
};

let queue: GaplessQueue | null = null;
const listeners = new Set<() => void>();
let snapshot: PlayerSnapshot = EMPTY;

function recompute() {
  const q = queue;
  snapshot = q
    ? {
        currentIndex: q.currentIndex,
        current: q.currentInfo,
        isPlaying: q.isPlaying,
        currentTime: q.currentTime,
        duration: q.duration,
        bufferedTime: q.current?.bufferedTime ?? 0,
        length: q.length,
        shuffle: q.shuffle,
        volume: q.volume,
        queue: q.queue,
      }
    : EMPTY;
  listeners.forEach((l) => l());
}

// --- scrobble tracking ---
// Last.fm scrobble rule: scrobble when the track has played for at least half
// its duration, OR at least 4 minutes (whichever comes first). Track per-id
// to avoid double-scrobbling on seeks.
let lastTrackId: number | null = null;
let scrobbleThresholdReached = false;

function handleTrackChange(_index: number, info: TrackInfo): void {
  // New track: set now-playing, reset scrobble state
  if (lastTrackId === info.id) return;
  lastTrackId = info.id;
  scrobbleThresholdReached = false;

  // Media Session
  setMediaMetadata(info);
  setMediaPositionState(info.duration, 0);

  // Now playing → maki (also sets Last.fm now-playing)
  void setPlaying(info.id).catch(() => {});

  // Publish to other devices if we're the active player
  publishRemoteState();
}

function handleTimeUpdate(time: number): void {
  const q = queue;
  if (!q || !q.current) return;

  // Update media session position + playback state
  setMediaPositionState(q.duration, time);

  // Scrobble threshold check
  if (scrobbleThresholdReached || !q.current) return;
  const duration = q.duration;
  if (duration <= 0) return;
  const halfDuration = duration / 2;
  const fourMinutes = 240;
  if (time >= Math.min(halfDuration, fourMinutes)) {
    scrobbleThresholdReached = true;
    void scrobbleTrack(q.current.info.id).catch(() => {});
  }
}

function handleStateChange(_state: "idle" | "loading" | "html5" | "webaudio"): void {
  const q = queue;
  setMediaPlaybackState(q?.isPlaying ? "playing" : "paused");
  publishRemoteState();
}

// --- HLS mode ---
// When enabled, tracks use HlsTrack (hls.js segment decode → WebAudio).
// When disabled, tracks use GaplessTrack (full-file decode → WebAudio).
const USE_HLS_KEY = "muse-web.useHLS";
const HLS_PROFILE_KEY = "muse-web.hlsProfile";

function loadBool(key: string): boolean {
  try {
    return localStorage.getItem(key) === "true";
  } catch {
    return false;
  }
}

let useHLS = loadBool(USE_HLS_KEY);
let hlsProfiles: HlsProfile[] = [];
let selectedProfile: string | null = (() => {
  try {
    return localStorage.getItem(HLS_PROFILE_KEY);
  } catch {
    return null;
  }
})();
let currentHlsProfile: HlsProfile | null = null;

function setUseHLS(value: boolean) {
  useHLS = value;
  try {
    localStorage.setItem(USE_HLS_KEY, String(value));
  } catch { /* noop */ }
}

function setSelectedProfile(name: string | null) {
  selectedProfile = name;
  try {
    if (name) localStorage.setItem(HLS_PROFILE_KEY, name);
    else localStorage.removeItem(HLS_PROFILE_KEY);
  } catch { /* noop */ }
}

/** Track factory — creates HlsTrack or GaplessTrack depending on useHLS. */
function createTrack(params: {
  info: TrackInfo;
  output?: AudioNode;
  onStateChange?: (state: TrackState) => void;
  onTimeUpdate?: (time: number) => void;
  onEnded?: () => void;
  onError?: (error: unknown) => void;
}) {
  if (useHLS) {
    return new HlsTrack({
      info: params.info,
      resolveHlsUrl: signTrackHls,
      output: params.output,
      onStateChange: params.onStateChange,
      onTimeUpdate: params.onTimeUpdate,
      onEnded: params.onEnded,
      onError: params.onError,
      profileName: selectedProfile,
    });
  }
  return new GaplessTrack({
    info: params.info,
    resolveUrl: resolveTrackUrl,
    output: params.output,
    onStateChange: params.onStateChange,
    onTimeUpdate: params.onTimeUpdate,
    onEnded: params.onEnded,
    onError: params.onError,
  });
}

async function ensureHlsProfiles() {
  if (hlsProfiles.length > 0) return;
  try {
    hlsProfiles = await fetchHlsProfiles();
  } catch {
    /* profiles unavailable — ABR still works */
  }
}

// --- remote control integration ---

/** Item IDs for the remote protocol — one per queue entry, stable for its lifetime. */
const queueItemIds: string[] = [];

function ensureItemIds(count: number): void {
  while (queueItemIds.length < count) {
    queueItemIds.push(crypto.randomUUID());
  }
}

function buildRemoteState(): RemotePlaybackState {
  const q = queue;
  if (!q) {
    return {
      current_item_id: null,
      position_ms: 0,
      is_playing: false,
      queue: [],
      updated_at: Date.now(),
    };
  }
  ensureItemIds(q.length);
  const idx = q.currentIndex;
  return {
    current_item_id: idx >= 0 ? queueItemIds[idx] : null,
    position_ms: Math.round(q.currentTime * 1000),
    is_playing: q.isPlaying,
    queue: q.queue.map((info, i) => ({
      item_id: queueItemIds[i],
      track_id: info.id,
    })),
    updated_at: Date.now(),
  };
}

function publishRemoteState(): void {
  if (remoteClient.isActivePlayer) {
    remoteClient.publishState(buildRemoteState());
  }
}

/** Route a command from a controller to the local queue. */
function handleRemoteCommand(cmd: RemoteCommand): void {
  const q = queue;
  if (!q) return;
  switch (cmd.kind) {
    case "play":
      q.play();
      break;
    case "pause":
      q.pause();
      break;
    case "toggle":
      q.togglePlay();
      break;
    case "next":
      void q.next();
      break;
    case "previous":
      void q.previous();
      break;
    case "seek":
      q.seekTo(cmd.position_ms / 1000);
      break;
    // set_queue, add_to_queue, remove_from_queue, reorder_queue
    // require track-id resolution which the queue doesn't support yet
    // (it works with TrackInfo, not bare IDs). These will be added when
    // the queue gains a setQueueByIds method.
    default:
      break;
  }
}

/** Pause local playback when another device takes over as active player. */
function handleActivePlayerChanged(isActive: boolean): void {
  if (!isActive) {
    queue?.pause();
  }
  recompute();
}

// Periodic publish — keeps other devices' state fresh (every 5s, like iOS)
let publishTimer: ReturnType<typeof setInterval> | null = null;

function startRemoteIntegration(): void {
  remoteClient.onIncomingCommand = handleRemoteCommand;
  remoteClient.onActivePlayerChanged = handleActivePlayerChanged;
  if (!publishTimer) {
    publishTimer = setInterval(() => publishRemoteState(), 5000);
  }
}

function getQueue(): GaplessQueue {
  if (!queue) {
    queue = new GaplessQueue({
      resolveUrl: resolveTrackUrl,
      trackFactory: createTrack,
      callbacks: {
        onTrackChange: (index, info) => {
          handleTrackChange(index, info);
          recompute();
        },
        onStateChange: (state) => {
          handleStateChange(state);
          recompute();
        },
        onTimeUpdate: (time) => {
          handleTimeUpdate(time);
          recompute();
        },
        onQueueEnd: () => {
          setMediaPlaybackState("none");
          recompute();
        },
      },
    });

    // Register media key handlers — route to the queue transport
    setMediaHandlers({
      play: () => queue?.play(),
      pause: () => queue?.pause(),
      previoustrack: () => void queue?.previous(),
      nexttrack: () => void queue?.next(),
      seekto: (time) => queue?.seekTo(time),
    });

    // Start remote control integration
    startRemoteIntegration();
    void remoteClient.start();

    // Fetch HLS profiles if HLS mode is enabled
    if (useHLS) void ensureHlsProfiles();
  }
  return queue;
}

function subscribe(cb: () => void): () => void {
  listeners.add(cb);
  return () => {
    listeners.delete(cb);
  };
}

function getSnapshot(): PlayerSnapshot {
  return snapshot;
}

/**
 * React bridge for the gapless queue. Subscribes via useSyncExternalStore so
 * components re-render on track/state/time changes. The queue is a
 * module-level singleton shared across the app.
 */
export function usePlayer() {
  useSyncExternalStore(subscribe, getSnapshot, getSnapshot);
  const q = getQueue();
  return {
    ...snapshot,
    playTracks(infos: TrackInfo[], startIndex = 0) {
      return q.setQueue(infos, startIndex);
    },
    addToQueue(infos: TrackInfo[]) {
      q.addTracks(infos);
      recompute();
    },
    toggle() {
      q.togglePlay();
    },
    next() {
      return q.next();
    },
    previous() {
      return q.previous();
    },
    seek(time: number) {
      q.seekTo(time);
    },
    jumpTo(index: number) {
      q.jumpTo(index);
    },
    setShuffle(enabled: boolean) {
      q.setShuffle(enabled);
      recompute();
    },
    setVolume(volume: number) {
      q.setVolume(volume);
      recompute();
    },
    // HLS controls
    getUseHLS() {
      return useHLS;
    },
    setUseHLS(value: boolean) {
      setUseHLS(value);
      if (value) void ensureHlsProfiles();
      recompute();
    },
    getHlsProfiles() {
      return hlsProfiles;
    },
    getSelectedProfile() {
      return selectedProfile;
    },
    setSelectedProfile(name: string | null) {
      setSelectedProfile(name);
      recompute();
    },
    getCurrentHlsProfile() {
      return currentHlsProfile;
    },
  };
}
