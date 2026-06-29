/**
 * Shared types for the gapless player engine.
 *
 * The engine is specified in detail in gapless.md at the repo root; these
 * types mirror the concepts described there.
 */

/** Finite states a GaplessTrack moves through (gapless.md state diagram). */
export type TrackState =
  /** Nothing loaded, or deactivated after being swapped out. */
  | "idle"
  /** fetch + decodeAudioData in flight. */
  | "loading"
  /** Playing (or paused) via an HTMLAudioElement — immediate, but not sample-accurate. */
  | "html5"
  /** On the shared AudioContext clock — sample-accurate, gapless-capable. */
  | "webaudio";

/** Minimal track metadata the engine needs. Mirrors a subset of the Maki Track. */
export interface TrackInfo {
  id: number;
  title: string;
  artistName: string;
  albumName: string;
  artUrl?: string;
  /** Duration in seconds from Maki metadata. Used until the decoded buffer is available. */
  duration: number;
}

/**
 * Resolve a Maki track id to a playable, CORS-enabled audio URL.
 * Maps to `GET /api/v1/track/:id/sign → { url }`.
 */
export type ResolveTrackUrl = (id: number) => Promise<string>;
