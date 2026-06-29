import Hls from "hls.js";
import { getAudioContext } from "@/player/audio-context";
import type { TrackInfo, TrackState } from "./types";

export interface HlsTrackParams {
  info: TrackInfo;
  resolveHlsUrl: (id: number) => Promise<string>;
  /** Audio node to connect into (the Queue's master gain). */
  output?: AudioNode;
  onStateChange?: (state: TrackState) => void;
  onTimeUpdate?: (time: number) => void;
  onEnded?: () => void;
  onError?: (error: unknown) => void;
  /** Optional fixed quality level name. If null, uses ABR (auto). */
  profileName?: string | null;
}

/**
 * HLS-powered track with segment-level WebAudio decode.
 *
 * Uses hls.js to fetch + demux HLS segments (master.m3u8 → variant → fMP4
 * segments). Instead of feeding segments to a MediaSource SourceBuffer,
 * we intercept each segment, combine it with the init segment, and
 * `decodeAudioData` it into an `AudioBuffer`. These buffers are chained
 * on the WebAudio clock for sample-accurate gapless playback.
 *
 * ABR: hls.js handles level selection. When profileName is null, hls.js
 * auto-selects based on bandwidth estimation. When set, we force the level.
 *
 * The key events:
 *  - FRAG_PARSING_INIT_SEGMENT: capture the fMP4 init box (moov)
 *  - FRAG_LOADED: the fragment's data (moof+mdat), prepend init, decode
 */
export class HlsTrack {
  readonly info: TrackInfo;

  private resolveHlsUrl: (id: number) => Promise<string>;
  private output?: AudioNode;
  private onStateChange?: (state: TrackState) => void;
  private onTimeUpdate?: (time: number) => void;
  private onEnded?: () => void;
  private onError?: (error: unknown) => void;
  private profileName?: string | null;

  private state: TrackState = "idle";

  // hls.js instance
  private hls: Hls | null = null;
  /** Hidden, muted <audio> element — drives hls.js's StreamController.
   *  hls.js needs a media element's currentTime to advance to load the
   *  next fragment. We play this silently; actual audio comes from WebAudio. */
  private mediaEl: HTMLAudioElement | null = null;
  /** Sync timer: pushes WebAudio currentTime back to the media element
   *  so hls.js loads fragments ahead of our playback position. */
  private mediaSyncTimer: ReturnType<typeof setInterval> | null = null;

  // --- segment decode pipeline ---
  private initSegment: Uint8Array | null = null;
  private decodedBuffers: AudioBuffer[] = [];
  private bufferDurations: number[] = []; // cumulative durations for seek math
  private totalDecodedDuration = 0;
  /** Fragment sequence numbers we've already decoded — prevents duplicates
   *  from hls.js firing BUFFER_APPENDING for multiple buffer types (audio,
   *  audiovideo, etc.) on the same fragment. */
  private decodedFragmentSNs = new Set<number | string>();

  // --- WebAudio playback ---
  private gain: GainNode | null = null;
  /** ALL active source nodes — we track every one so stopSource can clean
   *  them all up. Seeking creates a new set; the old set must be stopped. */
  private activeSources: AudioBufferSourceNode[] = [];
  private isPlayingFlag = false;
  private usingWebAudio = false;
  private webAudioStartedAt = 0;
  private offsetWhenStarted = 0;
  private pausedAtTrackTime = 0;
  private scheduledStartTime: number | null = null;
  private sourceGeneration = 0;

  // --- scheduling ---
  private rafId = 0;

  // --- url + preload ---
  private url: string | null = null;
  private preloadStarted = false;
  private fullyDecoded = false;

  constructor(params: HlsTrackParams) {
    this.info = params.info;
    this.resolveHlsUrl = params.resolveHlsUrl;
    this.output = params.output;
    this.onStateChange = params.onStateChange;
    this.onTimeUpdate = params.onTimeUpdate;
    this.onEnded = params.onEnded;
    this.onError = params.onError;
    this.profileName = params.profileName;
  }

  get currentState(): TrackState {
    return this.state;
  }

  get isPlaying(): boolean {
    return this.isPlayingFlag;
  }

  get isDecoded(): boolean {
    return this.decodedBuffers.length > 0;
  }

  get isFullyDecoded(): boolean {
    return this.fullyDecoded;
  }

  get duration(): number {
    return this.totalDecodedDuration || this.info.duration;
  }

  /** How much audio has been decoded so far (seconds). */
  get bufferedTime(): number {
    return this.totalDecodedDuration;
  }

  get scheduledStart(): number | null {
    return this.scheduledStartTime;
  }

  /** Current playback position in seconds. */
  get currentTime(): number {
    const ctx = getAudioContext();
    if (this.usingWebAudio && ctx) {
      if (this.isPlayingFlag) {
        const t =
          this.offsetWhenStarted + (ctx.currentTime - this.webAudioStartedAt);
        return Math.max(0, Math.min(t, this.duration));
      }
      return this.pausedAtTrackTime;
    }
    return this.pausedAtTrackTime;
  }

  // --------------------------------------------------------------- preload

  async preload(): Promise<void> {
    if (this.preloadStarted) return;
    this.preloadStarted = true;
    const ctx = getAudioContext();
    if (!ctx) {
      void this.ensureUrl().catch((e) => this.onError?.(e));
      return;
    }
    await this.loadManifest(ctx);
  }

  private async ensureUrl(): Promise<string> {
    if (this.url) return this.url;
    this.url = await this.resolveHlsUrl(this.info.id);
    return this.url;
  }

  private async loadManifest(_ctx: AudioContext): Promise<void> {
    this.setState("loading");
    try {
      const url = await this.ensureUrl();
      if (!Hls.isSupported()) {
        throw new Error("hls.js is not supported in this browser");
      }
      const hls = new Hls({
        enableWorker: true,
        lowLatencyMode: false,
        startFragPrefetch: true,
        // Load ahead aggressively — we need segments decoded before they play
        maxBufferLength: 60,
        maxMaxBufferLength: 120,
      });
      this.hls = hls;

      // Create a hidden, muted audio element to drive hls.js's loading loop.
      // hls.js's StreamController reads media.currentTime to decide which
      // fragment to load next. Without a media element, it stalls after
      // the first fragment.
      const mediaEl = new Audio();
      mediaEl.muted = true;
      mediaEl.volume = 0; // belt-and-suspenders: ensure no audio from the element
      mediaEl.preload = "auto";
      this.mediaEl = mediaEl;

      hls.attachMedia(mediaEl);

      hls.on(Hls.Events.MANIFEST_PARSED, (_event, data) => {
        // Set quality level if specified
        if (this.profileName) {
          const levelIdx = data.levels.findIndex(
            (l) => l.name === this.profileName || l.attrs?.NAME === this.profileName,
          );
          if (levelIdx >= 0) {
            hls.currentLevel = levelIdx;
          } else {
            hls.currentLevel = hls.levels.length - 1; // highest
          }
        } else {
          hls.currentLevel = -1; // auto (ABR)
        }
        // Start loading fragments
        hls.startLoad(0);
        // Play the muted media element so hls.js's StreamController advances
        // through fragments. We intercept the audio via BUFFER_APPENDING.
        void this.mediaEl?.play().catch(() => {});
        // Sync the media element's currentTime ahead of WebAudio playback
        // so hls.js loads fragments before we need them.
        this.startMediaSync();
      });

      // Intercept demuxed segment data. hls.js fires BUFFER_APPENDING once
      // per buffer type for each fragment — we deduplicate by fragment SN.
      hls.on(Hls.Events.BUFFER_APPENDING, async (_event, data) => {
        if (data.type !== "audio" && data.type !== "audiovideo") return;

        const chunk = data.data as Uint8Array;
        const fragSN = data.frag?.sn;

        // Init segment: no "moof" box. Capture it and return.
        if (!this.containsBox(chunk, "moof")) {
          this.initSegment = chunk;
          return;
        }

        // Deduplicate: hls.js fires BUFFER_APPENDING once per buffer type
        // (audio, audiovideo, etc.) for the same fragment. Only decode the
        // first one we see for each sequence number.
        if (fragSN != null && this.decodedFragmentSNs.has(fragSN)) return;
        if (fragSN != null) this.decodedFragmentSNs.add(fragSN);

        // It's a fragment — combine with init segment and decode
        if (!this.initSegment) return;

        try {
          const combined = this.combineInitAndFragment(this.initSegment, chunk);
          const ctx = getAudioContext();
          if (!ctx) return;

          const audioBuffer = await ctx.decodeAudioData(combined);
          const wasPlaying = this.isPlayingFlag;
          const alreadyWebAudio = this.usingWebAudio;
          this.decodedBuffers.push(audioBuffer);
          this.totalDecodedDuration += audioBuffer.duration;
          this.bufferDurations.push(this.totalDecodedDuration);

          if (wasPlaying && !alreadyWebAudio && this.decodedBuffers.length >= 1) {
            // Crossover: first buffer decoded, start WebAudio now.
            this.startWebAudioFromBuffers(ctx, this.pausedAtTrackTime);
          } else if (wasPlaying && alreadyWebAudio) {
            // Already playing on WebAudio — schedule the new buffer to play
            // right after the last scheduled one, so it chains seamlessly.
            this.scheduleAppendedBuffer(ctx, audioBuffer);
          }
        } catch (e) {
          // decodeAudioData may fail for some segment types — skip silently
          // but report for debugging
          this.onError?.(e);
        }
      });

      hls.on(Hls.Events.BUFFER_EOS, () => {
        this.fullyDecoded = true;
        this.hls?.destroy();
        this.hls = null;
        if (this.state === "loading") {
          this.setState("idle");
        }
      });

      hls.on(Hls.Events.ERROR, (_event, data) => {
        if (data.fatal) {
          this.onError?.(new Error(`HLS fatal: ${data.type} — ${data.details}`));
          this.setState("idle");
        }
      });

      hls.loadSource(url);
    } catch (err) {
      this.preloadStarted = false;
      this.onError?.(err);
      this.setState("idle");
    }
  }

  // --------------------------------------------------------------- playback

  async play(): Promise<void> {
    const ctx = getAudioContext();
    if (this.decodedBuffers.length > 0 && ctx) {
      this.startWebAudioFromBuffers(ctx, this.pausedAtTrackTime);
      return;
    }
    // No buffers yet — wait for decode. Mark as playing so the BUFFER_APPENDING
    // handler triggers the crossover when the first buffer is ready.
    this.isPlayingFlag = true;
    if (!this.preloadStarted) void this.preload();
  }

  /** Start (or resume) WebAudio playback from the decoded buffer chain. */
  private startWebAudioFromBuffers(ctx: AudioContext, offset: number) {
    this.stopSource();
    const gen = this.sourceGeneration;
    const gain = this.ensureGain(ctx);

    // Find which buffer to start from based on the offset
    const { bufferIndex, bufferOffset } = this.findBufferForOffset(offset);

    // Schedule the chain of buffers
    const startCtxTime = ctx.currentTime;

    for (let i = bufferIndex; i < this.decodedBuffers.length; i++) {
      const buf = this.decodedBuffers[i];
      const source = ctx.createBufferSource();
      source.buffer = buf;
      source.connect(gain);
      this.activeSources.push(source);

      const startOffset = i === bufferIndex ? bufferOffset : 0;

      // Schedule sequentially
      const scheduleAt = i === bufferIndex ? startCtxTime : this.lastScheduledEnd ?? startCtxTime;

      source.start(scheduleAt, startOffset);
      this.lastScheduledEnd = scheduleAt + (buf.duration - startOffset);

      // When the last scheduled buffer ends, fire onEnded
      if (i === this.decodedBuffers.length - 1) {
        source.addEventListener("ended", () => {
          if (gen === this.sourceGeneration) {
            this.onEnded?.();
          }
        });
      }
    }

    this.usingWebAudio = true;
    this.isPlayingFlag = true;
    this.offsetWhenStarted = offset;
    this.webAudioStartedAt = startCtxTime;
    this.pausedAtTrackTime = offset;
    this.scheduledStartTime = null;
    this.setState("webaudio");
    this.startProgressLoop();
  }

  private lastScheduledEnd = 0;

  /** Keep the muted media element's currentTime ahead of the WebAudio
   *  playback position so hls.js loads future fragments proactively.
   *
   *  The key constraint: never jump the media element so far ahead that
   *  hls.js thinks it's a seek and flushes its buffer. Only nudge it when
   *  it's clearly stuck (not advancing while WebAudio is playing). */
  private startMediaSync() {
    this.stopMediaSync();
    let lastMediaTime = 0;
    let stuckCount = 0;
    this.mediaSyncTimer = setInterval(() => {
      if (!this.mediaEl || this.fullyDecoded) return;
      if (!this.isPlayingFlag) return; // no need to sync when paused

      const webAudioTime = this.currentTime;
      const mediaTime = this.mediaEl.currentTime;

      // Detect if the media element is stuck (not advancing between ticks).
      if (Math.abs(mediaTime - lastMediaTime) < 0.1) {
        stuckCount++;
      } else {
        stuckCount = 0;
      }
      lastMediaTime = mediaTime;

      // Only nudge if stuck for 2+ ticks AND behind WebAudio by more than
      // 5 seconds. A small nudge (2s ahead) is enough to trigger loading
      // without hls.js interpreting it as a seek.
      if (stuckCount >= 2 && mediaTime < webAudioTime - 5) {
        this.mediaEl.currentTime = webAudioTime + 2;
        stuckCount = 0;
      }
    }, 2000);
  }

  private stopMediaSync() {
    if (this.mediaSyncTimer) {
      clearInterval(this.mediaSyncTimer);
      this.mediaSyncTimer = null;
    }
  }

  /** Schedule a newly-decoded buffer to play seamlessly after the last
   *  scheduled one. Called when a new segment finishes decoding during
   *  active WebAudio playback. */
  private scheduleAppendedBuffer(ctx: AudioContext, buffer: AudioBuffer) {
    const gain = this.ensureGain(ctx);
    const source = ctx.createBufferSource();
    source.buffer = buffer;
    source.connect(gain);
    this.activeSources.push(source);

    // Schedule right after the last buffer's end. If we've already passed
    // that time (decode was slow), start immediately.
    const startTime = Math.max(ctx.currentTime, this.lastScheduledEnd);
    source.start(startTime, 0);
    this.lastScheduledEnd = startTime + buffer.duration;

    // Invalidate old ended handlers and wire this as the new last source.
    const gen = ++this.sourceGeneration;
    source.addEventListener("ended", () => {
      if (gen === this.sourceGeneration) {
        this.onEnded?.();
      }
    });
  }

  private findBufferForOffset(offset: number): {
    bufferIndex: number;
    bufferOffset: number;
  } {
    for (let i = 0; i < this.bufferDurations.length; i++) {
      if (this.bufferDurations[i] > offset) {
        const start = i > 0 ? this.bufferDurations[i - 1] : 0;
        return { bufferIndex: i, bufferOffset: offset - start };
      }
    }
    // Offset is at or past the end — start from the last buffer
    const lastIdx = this.bufferDurations.length - 1;
    const lastStart = lastIdx > 0 ? this.bufferDurations[lastIdx - 1] : 0;
    return {
      bufferIndex: Math.max(0, lastIdx),
      bufferOffset: Math.max(0, offset - lastStart),
    };
  }

  pause() {
    if (this.usingWebAudio) {
      this.pausedAtTrackTime = this.currentTime;
      this.stopSource();
      this.isPlayingFlag = false;
    } else {
      this.isPlayingFlag = false;
    }
    this.mediaEl?.pause();
    this.stopProgressLoop();
  }

  async resume() {
    const ctx = getAudioContext();
    if (this.decodedBuffers.length > 0 && ctx) {
      this.startWebAudioFromBuffers(ctx, this.pausedAtTrackTime);
    } else {
      this.isPlayingFlag = true;
    }
    // Resume the muted media element to keep hls.js loading
    void this.mediaEl?.play().catch(() => {});
  }

  seek(time: number) {
    const clamped = Math.max(0, Math.min(time, this.duration));
    this.pausedAtTrackTime = clamped;
    const ctx = getAudioContext();

    // Case 1: seek target is within already-decoded buffers — just restart
    // WebAudio from the right offset.
    if (clamped <= this.totalDecodedDuration && this.decodedBuffers.length > 0) {
      if (this.usingWebAudio && ctx && this.isPlayingFlag) {
        this.startWebAudioFromBuffers(ctx, clamped);
      }
      this.onTimeUpdate?.(clamped);
      return;
    }

    // Case 2: seek target is past what we've decoded — tell hls.js to load
    // from the new position, clear stale buffers, wait for new segments.
    this.stopSource();
    this.decodedBuffers = [];
    this.bufferDurations = [];
    this.totalDecodedDuration = 0;
    this.initSegment = null;
    this.fullyDecoded = false;
    this.usingWebAudio = false;
    this.lastScheduledEnd = 0;
    this.decodedFragmentSNs.clear();

    // Tell hls.js to start loading from the seek position.
    if (this.hls) {
      // Flush hls.js's internal buffer state so it doesn't re-append old segments.
      this.hls.startLoad(clamped);
    }
    // Jump the media element to the seek position so hls.js's StreamController
    // loads fragments starting from there.
    if (this.mediaEl) {
      this.mediaEl.currentTime = clamped;
      if (this.isPlayingFlag) {
        void this.mediaEl.play().catch(() => {});
      }
    }

    // If playing, the BUFFER_APPENDING handler will trigger the crossover
    // once the first segment at the new position is decoded.
    // If paused, just update the position display.
    this.onTimeUpdate?.(clamped);
  }

  setVolume(volume: number) {
    if (this.gain) this.gain.gain.value = volume;
  }

  reconnect(output: AudioNode) {
    this.output = output;
    this.gain?.disconnect();
    this.gain?.connect(output);
  }

  // --------------------------------------------------------------- lifecycle

  deactivate() {
    this.stopProgressLoop();
    this.stopMediaSync();
    this.stopSource();
    this.usingWebAudio = false;
    this.isPlayingFlag = false;
    this.pausedAtTrackTime = 0;
    if (this.mediaEl) {
      this.mediaEl.pause();
      this.mediaEl.removeAttribute("src");
      this.mediaEl.load();
      this.mediaEl = null;
    }
    this.hls?.destroy();
    this.hls = null;
    this.setState("idle");
  }

  /** Arm this track to begin at a precise AudioContext time (gapless scheduling).
   *  For HLS tracks, this requires full decode to be complete. */
  scheduleGaplessStart(endTime: number) {
    const ctx = getAudioContext();
    if (!ctx || this.decodedBuffers.length === 0) return; // can't schedule — Queue falls back
    this.stopSource();
    const gen = this.sourceGeneration;
    const gain = this.ensureGain(ctx);

    this.scheduledStartTime = endTime;
    this.usingWebAudio = true;
    this.isPlayingFlag = true;
    this.offsetWhenStarted = 0;
    this.webAudioStartedAt = endTime;
    this.pausedAtTrackTime = 0;

    // Schedule all decoded buffers sequentially starting at endTime
    let scheduleAt = endTime;
    for (let i = 0; i < this.decodedBuffers.length; i++) {
      const buf = this.decodedBuffers[i];
      const source = ctx.createBufferSource();
      source.buffer = buf;
      source.connect(gain);
      this.activeSources.push(source);
      source.start(scheduleAt, 0);
      this.lastScheduledEnd = scheduleAt + buf.duration;

      if (i === this.decodedBuffers.length - 1) {
        source.addEventListener("ended", () => {
          if (gen === this.sourceGeneration) this.onEnded?.();
        });
      }
      scheduleAt += buf.duration;
    }

    this.setState("webaudio");
    this.startProgressLoop();
  }

  dispose() {
    this.deactivate();
    this.decodedBuffers = [];
    this.bufferDurations = [];
    this.totalDecodedDuration = 0;
    this.initSegment = null;
    this.preloadStarted = false;
    this.url = null;
    this.fullyDecoded = false;
    this.decodedFragmentSNs.clear();
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
    for (const src of this.activeSources) {
      try {
        src.stop();
      } catch {
        /* already stopped */
      }
      src.disconnect();
    }
    this.activeSources = [];
    this.sourceGeneration++;
    this.lastScheduledEnd = 0;
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

  // --------------------------------------------------------------- fMP4 utilities

  /** Check if a Uint8Array contains a specific ISOBMFF box type (4-char). */
  private containsBox(data: Uint8Array, boxType: string): boolean {
    const target = new Uint8Array(boxType.length);
    for (let i = 0; i < boxType.length; i++) {
      target[i] = boxType.charCodeAt(i);
    }
    // ISOBMFF boxes: 4 bytes size + 4 bytes type. Scan for the type.
    for (let i = 4; i <= data.length - 4; i++) {
      if (
        data[i] === target[0] &&
        data[i + 1] === target[1] &&
        data[i + 2] === target[2] &&
        data[i + 3] === target[3]
      ) {
        return true;
      }
    }
    return false;
  }

  /** Concatenate the init segment (moov) with a fragment (moof+mdat)
   *  to form a complete, decodable fMP4 file. */
  private combineInitAndFragment(
    init: Uint8Array,
    fragment: Uint8Array,
  ): ArrayBuffer {
    const combined = new Uint8Array(init.length + fragment.length);
    combined.set(init, 0);
    combined.set(fragment, init.length);
    return combined.buffer;
  }
}
