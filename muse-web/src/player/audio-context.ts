/**
 * Shared AudioContext singleton — the single monotonic clock that makes
 * gapless scheduling sample-accurate.
 *
 * Per gapless.md ("The Core Idea"): every track must be on the AudioContext
 * clock before it needs to be scheduled, so gapless transitions are always
 * WebAudio → WebAudio and there is no cross-clock drift between
 * `audio.currentTime` and `ctx.currentTime`.
 *
 * Browsers suspend AudioContext until a user gesture, so this module exposes
 * `resumeAudioContext()` to be called from a click/tap/keydown before the
 * first playback. Until then `getAudioContext()` returns null and the engine
 * falls back to HTML5 audio (small gap, but never silent failure).
 */

type AudioContextCtor = typeof AudioContext;

function resolveCtor(): AudioContextCtor | null {
  if (typeof window === "undefined") return null;
  return window.AudioContext ?? (window as unknown as {
    webkitAudioContext?: AudioContextCtor;
  }).webkitAudioContext ?? null;
}

let ctx: AudioContext | null = null;

/** The shared context, or null if not yet created. */
export function getAudioContext(): AudioContext | null {
  return ctx;
}

/** True once the context exists and is running (gapless scheduling enabled). */
export function isAudioContextReady(): boolean {
  return ctx != null && ctx.state === "running";
}

/**
 * Create (lazily) and resume the shared AudioContext. Must be called inside
 * a user gesture to satisfy autoplay policy. Idempotent.
 */
export function resumeAudioContext(): AudioContext | null {
  if (ctx == null) {
    const Ctor = resolveCtor();
    if (!Ctor) return null;
    ctx = new Ctor();
  }
  if (ctx.state === "suspended") {
    void ctx.resume().catch(() => {
      /* will retry on next gesture */
    });
  }
  return ctx;
}
