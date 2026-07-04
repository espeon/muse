import { getAudioContext } from "@/player/audio-context";
import { GaplessTrack } from "./track";
import type { ResolveTrackUrl, TrackInfo, TrackState } from "./types";

/** How many tracks ahead to fetch + decode when a track becomes current. */
const PRELOAD_AHEAD = 2;
/** Schedule the gapless handoff this many seconds before the current track ends. */
const SCHEDULE_LOOKAHEAD = 5;
/** Begin preloading within this many seconds of the current track's end. */
const PRELOAD_WINDOW = 25;

export interface QueueCallbacks {
  onTrackChange?: (index: number, info: TrackInfo) => void;
  onTimeUpdate?: (time: number) => void;
  onStateChange?: (state: TrackState) => void;
  onQueueEnd?: () => void;
  onError?: (error: unknown, info: TrackInfo) => void;
}

/** Interface that both GaplessTrack and HlsTrack implement. */
export interface PlayableTrack {
  readonly info: TrackInfo;
  readonly currentState: TrackState;
  readonly isPlaying: boolean;
  readonly isDecoded: boolean;
  readonly isFullyDecoded: boolean;
  readonly duration: number;
  /** How much of the track has been decoded/buffered (seconds). */
  readonly bufferedTime: number;
  readonly currentTime: number;
  readonly scheduledStart: number | null;
  /** The AudioContext time when the last scheduled source node will finish,
   *  or null if no sources are scheduled. This is the gapless scheduling
   *  signal — derived from PCM buffers, never from metadata duration. */
  readonly audioContextEndTime: number | null;
  preload(): Promise<void>;
  play(): Promise<void>;
  pause(): void;
  resume(): Promise<void>;
  seek(time: number): void;
  setVolume(volume: number): void;
  reconnect(output: AudioNode): void;
  deactivate(): void;
  dispose(): void;
  scheduleGaplessStart(endTime: number): void;
}

export interface GaplessQueueParams {
  resolveUrl: ResolveTrackUrl;
  callbacks?: QueueCallbacks;
  /** Factory to create tracks. Defaults to GaplessTrack. */
  trackFactory?: (params: {
    info: TrackInfo;
    output?: AudioNode;
    onStateChange?: (state: TrackState) => void;
    onTimeUpdate?: (time: number) => void;
    onEnded?: () => void;
    onError?: (error: unknown) => void;
  }) => PlayableTrack;
}

/**
 * Orchestrates a list of {@link GaplessTrack}s into a gapless queue.
 *
 * Responsibilities (gapless.md "Scheduling, Step by Step"):
 *  - preload the next {@link PRELOAD_AHEAD} tracks as the current one plays;
 *  - within {@link SCHEDULE_LOOKAHEAD} of the current track's end, arm the next
 *    track's source at the exact end time on the shared AudioContext clock;
 *  - on the current track's natural end, advance the index — the next track is
 *    already playing;
 *  - fall back to HTML5 (small gap) if the next buffer isn't decoded in time.
 *
 * All transport methods assume the AudioContext has been resumed by a prior
 * user gesture (see audio-context.ts).
 */
export class GaplessQueue {
  private tracks: PlayableTrack[] = [];
  private index = -1;
  private masterGain: GainNode | null = null;
  /** Indices of tracks armed via scheduleGaplessStart (pending handoff). */
  private scheduled = new Set<number>();
  private resolveUrl: ResolveTrackUrl;
  private callbacks: QueueCallbacks;
  private trackFactory: GaplessQueueParams["trackFactory"];
  private _shuffle = false;

  constructor(params: GaplessQueueParams) {
    this.resolveUrl = params.resolveUrl;
    this.callbacks = params.callbacks ?? {};
    this.trackFactory = params.trackFactory;
  }

  get current(): PlayableTrack | null {
    return this.tracks[this.index] ?? null;
  }

  get currentIndex(): number {
    return this.index;
  }

  get currentInfo(): TrackInfo | null {
    return this.current?.info ?? null;
  }

  get length(): number {
    return this.tracks.length;
  }

  get isPlaying(): boolean {
    return this.current?.isPlaying ?? false;
  }

  get currentTime(): number {
    return this.current?.currentTime ?? 0;
  }

  get duration(): number {
    return this.current?.duration ?? 0;
  }

  get shuffle(): boolean {
    return this._shuffle;
  }

  get volume(): number {
    return this.masterGain?.gain.value ?? 1;
  }

  get queue(): TrackInfo[] {
    return this.tracks.map((t) => t.info);
  }

  // ------------------------------------------------------------------- queue mgmt

  /**
   * Replace the queue and start playing at `startIndex`. Must be invoked within
   * a user gesture that has already resumed the AudioContext.
   */
  async setQueue(infos: TrackInfo[], startIndex = 0): Promise<void> {
    this.disposeAll();
    this.ensureMasterGain();
    this.tracks = infos.map((info) => this.makeTrack(info));
    this.scheduled.clear();
    await this.playAt(startIndex);
  }

  /** Append tracks without disturbing current playback. */
  addTracks(infos: TrackInfo[]): void {
    this.ensureMasterGain();
    const made = infos.map((info) => this.makeTrack(info));
    this.tracks.push(...made);
  }

  private makeTrack(info: TrackInfo): PlayableTrack {
    const trackParams = {
      info,
      output: this.masterGain ?? undefined,
      onStateChange: (state: TrackState) => {
        if (this.tracks[this.index] === track) this.callbacks.onStateChange?.(state);
      },
      onTimeUpdate: (time: number) => {
        if (this.tracks[this.index] === track) this.onCurrentTimeUpdate(time);
      },
      onEnded: () => this.onTrackEnded(track),
      onError: (error: unknown) => this.callbacks.onError?.(error, info),
    };

    const track = this.trackFactory
      ? this.trackFactory(trackParams)
      : new GaplessTrack({
          resolveUrl: this.resolveUrl,
          ...trackParams,
        });
    return track;
  }

  private async playAt(target: number): Promise<void> {
    this.cancelScheduledNext();
    this.tracks.forEach((t, i) => {
      if (i !== target) t.deactivate();
    });
    this.index = target;
    const cur = this.tracks[target];
    if (!cur) return;
    await cur.play();
    this.callbacks.onTrackChange?.(target, cur.info);
    this.preloadAhead();
  }

  private disposeAll() {
    this.tracks.forEach((t) => t.dispose());
    this.tracks = [];
    this.index = -1;
    this.scheduled.clear();
  }

  // ------------------------------------------------------------------- transport

  play() {
    const cur = this.current;
    if (cur && !cur.isPlaying) void cur.resume();
  }

  pause() {
    // A gapless-scheduled next track is armed on the AudioContext clock, which
    // keeps running while the current source is paused — cancel it so the next
    // track doesn't fire anyway. It will be re-armed on resume.
    this.cancelScheduledNext();
    this.current?.pause();
  }

  togglePlay() {
    if (this.isPlaying) this.pause();
    else this.play();
  }

  async next() {
    if (this._shuffle && this.tracks.length > 1) {
      // Pick a random track that isn't the current one.
      let nextIdx: number;
      do {
        nextIdx = Math.floor(Math.random() * this.tracks.length);
      } while (nextIdx === this.index);
      await this.playAt(nextIdx);
    } else if (this.index + 1 < this.tracks.length) {
      await this.playAt(this.index + 1);
    }
  }

  async previous() {
    const cur = this.current;
    if (cur && cur.currentTime > 3) {
      this.cancelScheduledNext();
      cur.seek(0);
      return;
    }
    if (this.index - 1 >= 0) await this.playAt(this.index - 1);
  }

  jumpTo(index: number) {
    if (index >= 0 && index < this.tracks.length) {
      void this.playAt(index);
    }
  }

  seekTo(time: number) {
    this.cancelScheduledNext();
    this.current?.seek(time);
  }

  setVolume(volume: number) {
    if (this.masterGain) this.masterGain.gain.value = volume;
  }

  setShuffle(enabled: boolean) {
    this._shuffle = enabled;
  }

  // ------------------------------------------------------------------- scheduling

  private onCurrentTimeUpdate(time: number) {
    this.callbacks.onTimeUpdate?.(time);
    this.maybePreloadAhead(time);
    this.maybeScheduleGapless(time);
  }

  private maybePreloadAhead(time: number) {
    const cur = this.current;
    if (!cur) return;
    const dur = cur.bufferedTime || cur.duration;
    if (dur > 0 && dur - time <= PRELOAD_WINDOW) this.preloadAhead();
  }

  private preloadAhead() {
    for (let i = 1; i <= PRELOAD_AHEAD; i++) {
      const t = this.tracks[this.index + i];
      if (t) void t.preload();
    }
  }

  private maybeScheduleGapless(_time: number) {
    const cur = this.current;
    if (!cur) return;
    const nextIndex = this.index + 1;
    const next = this.tracks[nextIndex];
    if (!next) return;
    if (this.scheduled.has(nextIndex)) return;
    // Both tracks must be fully decoded: the current one so its end time is
    // known (audioContextEndTime is final), the next one so its buffers can be
    // armed on the AudioContext clock.
    if (!cur.isFullyDecoded) return;
    if (!next.isFullyDecoded) return;
    const endTime = cur.audioContextEndTime;
    if (endTime == null) return;
    const ctx = getAudioContext();
    if (!ctx) return;
    if (endTime - ctx.currentTime > SCHEDULE_LOOKAHEAD) return;
    next.scheduleGaplessStart(endTime);
    // Only trust the scheduling if the track's end time was set (scheduleGaplessStart
    // can silently no-op when guards fail — decodeBaseTime, missing ctx, etc.).
    if (next.audioContextEndTime != null) {
      this.scheduled.add(nextIndex);
    }
  }

  private onTrackEnded(track: PlayableTrack) {
    if (this.tracks[this.index] !== track) return; // stale event from an old source
    const nextIndex = this.index + 1;
    if (nextIndex >= this.tracks.length) {
      this.callbacks.onQueueEnd?.();
      return;
    }
    const next = this.tracks[nextIndex];
    if (!next) return;
    const wasScheduled = this.scheduled.has(nextIndex);
    this.scheduled.delete(nextIndex);
    track.deactivate();
    this.index = nextIndex;
    if (!wasScheduled) {
      // Buffer wasn't ready in time — HTML5 fallback (small gap).
      void next.play();
    }
    // If scheduled, next is already playing with its progress loop running.
    this.callbacks.onTrackChange?.(nextIndex, next.info);
    this.preloadAhead();
  }

  /** Cancel any gapless-scheduled upcoming track (pause/seek/skip). Keeps the
   *  decoded buffer so the track can be re-armed later. */
  private cancelScheduledNext() {
    if (this.scheduled.size === 0) return;
    for (const i of this.scheduled) this.tracks[i]?.deactivate();
    this.scheduled.clear();
  }

  // ------------------------------------------------------------------- audio graph

  private ensureMasterGain(): GainNode | null {
    const ctx = getAudioContext();
    if (!ctx) return null;
    if (!this.masterGain) {
      const gain = ctx.createGain();
      gain.gain.value = 1;
      gain.connect(ctx.destination);
      this.masterGain = gain;
      this.tracks.forEach((t) => t.reconnect(gain));
    }
    return this.masterGain;
  }

  dispose() {
    this.disposeAll();
    this.masterGain?.disconnect();
    this.masterGain = null;
  }
}
