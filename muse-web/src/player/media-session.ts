import type { TrackInfo } from "./types";

/**
 * Media Session API integration — exposes playback metadata and transport
 * controls to the OS (lock screen, media keys, OS-level now-playing widget).
 *
 * https://developer.mozilla.org/en-US/docs/Web/API/Media_Session_API
 */

export interface MediaSessionHandlers {
  play: () => void;
  pause: () => void;
  previoustrack: () => void;
  nexttrack: () => void;
  seekto: (time: number) => void;
}

const isSupported = (): boolean =>
  typeof navigator !== "undefined" && "mediaSession" in navigator;

/** Set metadata for the current track (title, artist, album, artwork). */
export function setMediaMetadata(track: TrackInfo): void {
  if (!isSupported()) return;
  const artwork =
    track.artUrl != null
      ? [
          { src: track.artUrl, sizes: "512x512", type: "image/jpeg" },
          {
            src: track.artUrl.replace(/w=\d+/, "w=1024"),
            sizes: "1024x1024",
            type: "image/jpeg",
          },
        ]
      : [];
  navigator.mediaSession.metadata = new MediaMetadata({
    title: track.title,
    artist: track.artistName,
    album: track.albumName,
    artwork,
  });
}

/** Update the OS-level playback state. */
export function setMediaPlaybackState(
  state: "playing" | "paused" | "none",
): void {
  if (!isSupported()) return;
  navigator.mediaSession.playbackState = state;
}

/** Update the position state (duration + current position). */
export function setMediaPositionState(
  duration: number,
  currentTime: number,
): void {
  if (!isSupported()) return;
  if (!("setPositionState" in navigator.mediaSession)) return;
  if (!isFinite(duration) || duration <= 0) return;
  navigator.mediaSession.setPositionState({
    duration,
    position: Math.min(currentTime, duration),
    playbackRate: 1,
  });
}

/** Register action handlers for hardware media keys and OS controls. */
export function setMediaHandlers(handlers: MediaSessionHandlers): void {
  if (!isSupported()) return;
  const ms = navigator.mediaSession;
  const set = (action: MediaSessionAction, handler: () => void) => {
    try {
      ms.setActionHandler(action, handler);
    } catch {
      // Some actions may not be supported on this browser — ignore.
    }
  };
  set("play", handlers.play);
  set("pause", handlers.pause);
  set("previoustrack", handlers.previoustrack);
  set("nexttrack", handlers.nexttrack);
  try {
    ms.setActionHandler("seekto", (details: MediaSessionActionDetails) => {
      if (details.seekTime != null) handlers.seekto(details.seekTime);
    });
  } catch {
    // not supported
  }
  // Seek buttons on some OSes
  try {
    ms.setActionHandler("seekbackward", () => {
      // no-op — most web players handle seek via the UI
    });
  } catch {
    /* noop */
  }
  try {
    ms.setActionHandler("seekforward", () => {
      /* no-op */
    });
  } catch {
    /* noop */
  }
}

/** Clear all metadata and handlers (e.g. on logout). */
export function clearMediaSession(): void {
  if (!isSupported()) return;
  navigator.mediaSession.metadata = null;
  navigator.mediaSession.playbackState = "none";
  setMediaHandlers({
    play: () => {},
    pause: () => {},
    previoustrack: () => {},
    nexttrack: () => {},
    seekto: () => {},
  });
}
