import { getAudioContext } from "@/player/audio-context";
import type { ResolveTrackUrl, TrackInfo, TrackState } from "./types";

export interface GaplessTrackParams {
  info: TrackInfo;
  resolveUrl: ResolveTrackUrl;
  /** Audio node tracks connect into (the Queue's master gain). Defaults to ctx.destination. */
  output?: AudioNode;
  onStateChange?: (state: TrackState) => void;
  onTimeUpdate?: (time: number) => void;
  /** Fired when the track finishes playing — for gapless-scheduled tracks, at the true end. */
  onEnded?: () => void;
  onError?: (error: unknown) => void;
}

/**
 * A single playable track with two backends:
 *
 *  - **HTML5 `<audio>`** — immediate first playback (no decode wait before
 *    sound starts).
 *  - **Web Audio `AudioBuffer`** — sample-accurate scheduling on the shared
 *    `AudioContext` clock.
 *
 * The **mid-stream crossover** (gapless.md "The Mid-Stream Crossover") moves a
 * track from HTML5 onto the AudioContext clock as soon as its buffer decodes,
 * so all gapless transitions are WebAudio → WebAudio and there is no
 * cross-clock drift.
 *
 * `AudioBufferSourceNode`s are one-shot: every play/resume/seek creates a
 * fresh source node that reuses the decoded `AudioBuffer`.
 *
 * The three scheduling bugs documented in gapless.md are baked into the
 * design of `scheduleGaplessStart()`:
 *   1. it enters `webaudio` state (so `currentTime` reads from the webaudio clock);
 *   2. it starts the progress loop;
 *   3. it sets `isPlaying` true even when the machine was already `webaudio`.
 */
export class GaplessTrack {
  readonly info: TrackInfo;

  private resolveUrl: ResolveTrackUrl;
  private output?: AudioNode;
  private onStateChange?: (state: TrackState) => void;
  private onTimeUpdate?: (time: number) => void;
  private onEnded?: () => void;
  private onError?: (error: unknown) => void;

  private state: TrackState = "idle";

  // --- HTML5 backend ---
  private audio: HTMLAudioElement | null = null;

  // --- Web Audio backend ---
  private buffer: AudioBuffer | null = null;
  private gain: GainNode | null = null;
  private source: AudioBufferSourceNode | null = null;

  // --- scheduling bookkeeping (all on the AudioContext clock once crossover happens) ---
  private isPlayingFlag = false;
  private usingWebAudio = false;
  /** ctx.currentTime the current source started at (for currentTime calc). */
  private webAudioStartedAt = 0;
  /** track-time offset the current source was started at. */
  private offsetWhenStarted = 0;
  /** frozen track-time when paused (webaudio path). */
  private pausedAtTrackTime = 0;
  /** ctx.currentTime this track was gapless-scheduled to begin at (null if started now). */
  private scheduledStartContextTime: number | null = null;
  /** AudioContext time when the current source finishes. Null when stopped/paused. */
  private _audioContextEndTime: number | null = null;

  // --- preload pipeline ---
  private url: string | null = null;
  private decoded = false;
  private decodeInFlight = false;
  private preloadStarted = false;

  // --- progress loop ---
  private rafId = 0;
  /** Generation counter: invalidates `ended` events from stopped source nodes. */
  private sourceGeneration = 0;

  constructor(params: GaplessTrackParams) {
    this.info = params.info;
    this.resolveUrl = params.resolveUrl;
    this.output = params.output;
    this.onStateChange = params.onStateChange;
    this.onTimeUpdate = params.onTimeUpdate;
    this.onEnded = params.onEnded;
    this.onError = params.onError;
  }

  get currentState(): TrackState {
    return this.state;
  }

  get isPlaying(): boolean {
    return this.isPlayingFlag;
  }

  get isDecoded(): boolean {
    return this.decoded;
  }

  get isFullyDecoded(): boolean {
    return this.decoded;
  }

  /** For GaplessTrack, buffered = full duration once decoded. */
  get bufferedTime(): number {
    return this.decoded ? this.duration : 0;
  }

  /** Effective duration: decoded buffer duration if available, else metadata. */
  get duration(): number {
    return this.buffer?.duration ?? this.info.duration;
  }

  /** ctx.currentTime this track was scheduled to start at (for Queue end-time math). */
  get scheduledStart(): number | null {
    return this.scheduledStartContextTime;
  }

  /** The AudioContext time when the current source finishes, or null. */
  get audioContextEndTime(): number | null {
    return this._audioContextEndTime;
  }

  /** Current playback position in seconds, from whichever clock is active. */
  get currentTime(): number {
    const ctx = getAudioContext();
    if (this.usingWebAudio && ctx) {
      if (this.isPlayingFlag) {
        const t = this.offsetWhenStarted + (ctx.currentTime - this.webAudioStartedAt);
        return Math.max(0, Math.min(t, this.duration));
      }
      return this.pausedAtTrackTime;
    }
    return this.audio?.currentTime ?? 0;
  }

  // --------------------------------------------------------------- preload

  /**
   * Fetch + decode this track's AudioBuffer. Idempotent. On success, if the
   * track is currently playing on HTML5, performs the mid-stream crossover.
   * Silently no-ops (defers decode) if no AudioContext exists yet.
   */
  async preload(): Promise<void> {
    if (this.preloadStarted) return;
    this.preloadStarted = true;
    const ctx = getAudioContext();
    if (!ctx) {
      // No context yet (no user gesture). Prime the URL; decode when context exists.
      void this.ensureUrl().catch((e) => this.onError?.(e));
      return;
    }
    await this.decode(ctx);
  }

  private async ensureUrl(): Promise<string> {
    if (this.url) return this.url;
    this.url = await this.resolveUrl(this.info.id);
    return this.url;
  }

  private async decode(ctx: AudioContext): Promise<void> {
    if (this.decoded || this.decodeInFlight) return;
    this.decodeInFlight = true;
    try {
      this.setState("loading");
      const url = await this.ensureUrl();
      const res = await fetch(url);
      if (!res.ok) throw new Error(`track fetch failed: ${res.status}`);
      const arrayBuffer = await res.arrayBuffer();
      const buffer = await ctx.decodeAudioData(arrayBuffer);
      this.buffer = buffer;
      this.decoded = true;
      this.handleBufferReady(ctx);
    } catch (err) {
      this.preloadStarted = false; // allow a later retry
      this.onError?.(err);
      this.setState("idle");
    } finally {
      this.decodeInFlight = false;
    }
  }

  /** Called when the AudioBuffer finishes decoding (BUFFER_READY in gapless.md). */
  private handleBufferReady(ctx: AudioContext) {
    if (this.state === "loading") {
      // Preloaded but not current — wait for the Queue to play it.
      this.setState("idle");
    }
    // If currently playing on HTML5, crossover onto the AudioContext clock.
    if (this.isPlayingFlag && !this.usingWebAudio && this.audio) {
      this.crossover(ctx, this.audio.currentTime);
    }
  }

  // --------------------------------------------------------------- playback

  /** Start (or resume) playback. Begins on HTML5 for immediacy if the buffer isn't ready. */
  async play(): Promise<void> {
    const ctx = getAudioContext();
    // Decoded buffer available → go straight to Web Audio.
    if (this.decoded && ctx) {
      this.startWebAudio(ctx, this.pausedAtTrackTime);
      return;
    }
    // Otherwise start on HTML5 now and crossover once the buffer is ready.
    await this.startHtml5();
    if (!this.preloadStarted) void this.preload();
    else if (ctx) void this.decode(ctx);
  }

  /** Resolve the URL if needed, then HTML5-play from the current position. */
  private async startHtml5(): Promise<void> {
    const url = await this.ensureUrl();
    if (!this.audio) {
      const audio = new Audio();
      audio.preload = "auto";
      audio.crossOrigin = "anonymous";
      audio.src = url;
      audio.addEventListener("ended", () => this.onEnded?.());
      this.audio = audio;
    }
    this.audio.currentTime = this.pausedAtTrackTime;
    this.usingWebAudio = false;
    this.setState("html5");
    this.isPlayingFlag = true;
    try {
      await this.audio.play();
      this.startProgressLoop();
    } catch (err) {
      this.isPlayingFlag = false;
      this.onError?.(err);
    }
  }

  /**
   * Mid-stream crossover: move a track playing on HTML5 onto the AudioContext
   * clock at the exact offset it was at. (gapless.md linchpin.)
   */
  private crossover(ctx: AudioContext, atOffset: number) {
    this.audio?.pause();
    this.startWebAudio(ctx, atOffset);
  }

  /** Start (or restart) a Web Audio source node at the given track-time offset, now. */
  private startWebAudio(ctx: AudioContext, offset: number) {
    if (!this.buffer) return;
    this.stopSource();
    const gen = this.sourceGeneration;
    const gain = this.ensureGain(ctx);
    const source = ctx.createBufferSource();
    source.buffer = this.buffer;
    source.connect(gain);
    source.addEventListener("ended", () => {
      if (gen === this.sourceGeneration) this.onEnded?.();
    });
    source.start(0, offset);
    this.source = source;
    this.usingWebAudio = true;
    this.isPlayingFlag = true;
    this.offsetWhenStarted = offset;
    this.webAudioStartedAt = ctx.currentTime;
    this.pausedAtTrackTime = offset;
    this.scheduledStartContextTime = null;
    this._audioContextEndTime = ctx.currentTime + (this.buffer!.duration - offset);
    this.setState("webaudio");
    this.startProgressLoop();
  }

  /**
   * Arm this track to begin at a precise AudioContext time (sample-accurate).
   * Used by the Queue for gapless transitions. All three gapless.md scheduling
   * bugs are handled here (see class docs).
   */
  scheduleGaplessStart(endTime: number) {
    const ctx = getAudioContext();
    if (!ctx || !this.buffer) return; // can't schedule — Queue falls back to HTML5
    this.stopSource();
    const gen = this.sourceGeneration;
    const gain = this.ensureGain(ctx);
    const source = ctx.createBufferSource();
    source.buffer = this.buffer;
    source.connect(gain);
    source.addEventListener("ended", () => {
      if (gen === this.sourceGeneration) this.onEnded?.();
    });
    source.start(endTime, 0);
    this.source = source;
    this.usingWebAudio = true;
    this.isPlayingFlag = true; // bug fix #3
    this.scheduledStartContextTime = endTime;
    this.webAudioStartedAt = endTime;
    this.offsetWhenStarted = 0;
    this.pausedAtTrackTime = 0;
    this._audioContextEndTime = endTime + this.buffer!.duration;
    this.setState("webaudio"); // bug fix #1
    this.startProgressLoop(); // bug fix #2
  }

  pause() {
    if (this.usingWebAudio) {
      this.pausedAtTrackTime = this.currentTime;
      this.stopSource(); // one-shot node can't pause; recreated on resume
      this.isPlayingFlag = false;
    } else if (this.audio) {
      this.audio.pause();
      this.pausedAtTrackTime = this.audio.currentTime;
      this.isPlayingFlag = false;
    }
    this.stopProgressLoop();
  }

  async resume() {
    const ctx = getAudioContext();
    if (this.usingWebAudio && ctx && this.buffer) {
      this.startWebAudio(ctx, this.pausedAtTrackTime);
    } else if (this.audio) {
      this.isPlayingFlag = true;
      try {
        await this.audio.play();
        this.startProgressLoop();
      } catch (err) {
        this.isPlayingFlag = false;
        this.onError?.(err);
      }
    }
  }

  seek(time: number) {
    const clamped = Math.max(0, Math.min(time, this.duration));
    this.pausedAtTrackTime = clamped;
    const ctx = getAudioContext();
    if (this.usingWebAudio && ctx && this.buffer && this.isPlayingFlag) {
      this.startWebAudio(ctx, clamped);
    } else if (this.audio) {
      this.audio.currentTime = clamped;
    }
    this.onTimeUpdate?.(clamped);
  }

  /** Per-track gain (0..1). The Queue's master gain controls overall volume. */
  setVolume(volume: number) {
    if (this.gain) this.gain.gain.value = volume;
  }

  /** Re-route this track's gain into a different output node — used when the
   *  Queue's master gain is created after this track was constructed. */
  reconnect(output: AudioNode) {
    this.output = output;
    this.gain?.disconnect();
    this.gain?.connect(output);
  }

  // --------------------------------------------------------------- lifecycle

  /** Stop and reset to idle, keeping the decoded buffer (used when swapped out). */
  deactivate() {
    this.stopProgressLoop();
    this.stopSource();
    if (this.audio) {
      this.audio.pause();
      this.audio.removeAttribute("src");
      this.audio.load();
      this.audio = null;
    }
    this.usingWebAudio = false;
    this.isPlayingFlag = false;
    this.pausedAtTrackTime = 0;
    this.setState("idle");
  }

  /** Fully release resources. */
  dispose() {
    this.deactivate();
    this.buffer = null;
    this.decoded = false;
    this.preloadStarted = false;
    this.url = null;
    this.gain?.disconnect();
    this.gain = null;
  }

  // --------------------------------------------------------------- internals

  private ensureGain(ctx: AudioContext): GainNode {
    if (!this.gain) {
      const gain = ctx.createGain();
      gain.gain.value = 1;
      gain.connect(this.output ?? ctx.destination);
      this.gain = gain;
    }
    return this.gain;
  }

  private stopSource() {
    if (this.source) {
      try {
        this.source.stop();
      } catch {
        /* already stopped */
      }
      this.source.disconnect();
      this.source = null;
    }
    // Invalidate any in-flight `ended` event from the node we just stopped,
    // so pause/seek/crossover don't advance the queue.
    this.sourceGeneration++;
    this._audioContextEndTime = null;
  }

  private startProgressLoop() {
    this.stopProgressLoop();
    const tick = () => {
      if (!this.isPlayingFlag) return;
      this.onTimeUpdate?.(this.currentTime);
      this.rafId = requestAnimationFrame(tick);
    };
    this.rafId = requestAnimationFrame(tick);
  }

  private stopProgressLoop() {
    if (this.rafId) cancelAnimationFrame(this.rafId);
    this.rafId = 0;
  }

  private setState(next: TrackState) {
    if (this.state === next) return;
    this.state = next;
    this.onStateChange?.(next);
  }
}
