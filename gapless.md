# gapless.js v4 — Gapless Playback Mechanism

Extracted from the full architecture doc. This covers only how sample-accurate
gapless transitions actually work — the clock model, the mid-stream crossover,
the scheduling math, and the supporting machinery (preloading, WebAudio-clock
timing, fallback). Public API, React integration, Media Session, dev tooling,
and non-scheduling bugs are omitted.

---

## The Core Idea

Schedule the next track's `AudioBufferSourceNode.start(when)` at the exact
`AudioContext.currentTime` when the current track ends. Because all tracks share
one `AudioContext`, the clock is monotonic and the transition is sample-accurate
by construction — **but only if the current track is also playing through Web
Audio.**

If the current track is on HTML5, `audio.currentTime` and `ctx.currentTime` are
independent clocks, and any prediction across them is unreliable. They drift
independently (buffering stalls, codec padding, long-session skew), so a
cross-clock prediction is wrong some fraction of the time — this is the root
cause of the gapless overlap bug. The fix is to make sure every track is on the
AudioContext clock before it needs to be scheduled, so gapless scheduling is
always WebAudio→WebAudio.

This requires a **single shared `AudioContext`** (created lazily on the first
user gesture via `resumeAudioContext()`). `AudioContext.currentTime` must be a
common clock for scheduling to work.

---

## Why Two Backends

Each `Track` holds both an `HTMLAudioElement` and (once preloaded) a decoded
`AudioBuffer` + `GainNode` + `AudioBufferSourceNode`.

- **HTML5** gives immediate playback — no fetch/decode wait before the first
  track starts.
- **Web Audio** gives sample-accurate scheduling against the shared clock.

The trick is moving every track from HTML5 onto the AudioContext clock as soon
as its buffer is decoded, via the crossover below.

---

## The Mid-Stream Crossover (the linchpin)

The first track in a session starts on HTML5 for immediate playback. As soon as
its buffer finishes decoding, `BUFFER_READY` fires in the TrackMachine's `html5`
state and triggers a mid-stream handoff:

1. The HTML5 `<audio>` element is paused.
2. An `AudioBufferSourceNode` is started at the **exact `audio.currentTime`
   offset** the HTML5 element was at.
3. The track enters `webaudio` state with `isPlaying` preserved.

From this instant on, the track is on the AudioContext clock. Once a track is on
that clock, all subsequent gapless transitions share a single clock and are
sample-accurate. The alternative — staying in HTML5 and predicting end-time from
the AudioContext clock — is exactly the cross-clock prediction that drifts and
fails.

```
           PLAY              PLAY_WEBAUDIO
  idle ──────────► html5 ──────────────────► webaudio
    │                │  BUFFER_READY            │
    │ PRELOAD        │  (mid-stream crossover)  │ DEACTIVATE
    ▼                ▼                          │ WEBAUDIO_ENDED
  loading ◄──────  html5                        ▼
    │                                          idle
    │ BUFFER_ERROR
    ▼
  idle
```

Related transitions:

- `BUFFER_READY` in `loading` → `idle` + LOADED. A preloaded-but-not-current
  track decodes and waits for the Queue to decide when to play it.
- `DEACTIVATE` from `webaudio` → `idle` (not `loading`), since the track is being
  swapped out.

---

## Scheduling, Step by Step

1. **Preload.** When a track starts playing, the Queue calls `_preloadAhead()`
   to fetch + decode the next 2 tracks (`PRELOAD_AHEAD = 2`). Each spawns a
   fetch/decode pipeline: resolve URL → fetch → `decodeAudioData()`.

2. **Crossover.** First track crosses from HTML5 to WebAudio on `BUFFER_READY`
   (see above), putting it on the AudioContext clock.

3. **Schedule.** When the current track is within 5 seconds of its end
   (`GAPLESS_SCHEDULE_LOOKAHEAD = 5`) and the next track's buffer is decoded,
   `_tryScheduleGapless()` computes the exact end time:
   - Track that was itself gapless-scheduled:
     `current.scheduledStartContextTime + current.duration`
   - Track started via crossover or normal `play()` into webaudio:
     `ctx.currentTime + remaining`

   Both terms are on the AudioContext clock — no cross-clock prediction either
   way.

4. **Start at precise time.** `next.scheduleGaplessStart(endTime)` creates a
   source node and calls `sourceNode.start(endTime, 0)`. The Web Audio scheduler
   guarantees the node begins at exactly that sample.

5. **Handoff.** When the current track's source node fires `onended`, the Queue
   advances `currentTrackIndex`. The next track is already playing
   (pre-scheduled), so the Queue just starts its progress loop.

---

## Preloading (feeds scheduling)

- `PRELOAD_AHEAD = 2` in Queue.ts — when a track starts, the next 2 unloaded
  tracks are fetched and decoded sequentially.
- `onTrackBufferReady` cascades: when track N finishes decoding, track N+1
  starts.
- The progress loop is a second trigger at `GAPLESS_SCHEDULE_LOOKAHEAD = 5`
  seconds before track end, ensuring gapless scheduling happens even if
  preloading finished long ago.
- Preloading begins automatically within the last 25 seconds of the current
  track. If the next buffer isn't decoded before the current track ends,
  playback falls back to HTML5 and there's a small gap.

---

## WebAudio-Clock Timing

Because gapless tracks live on the AudioContext clock, `currentTime` must be read
from it:

```
WebAudio playing:  ctx.currentTime - webAudioStartedAt
WebAudio paused:   pausedAtTrackTime  (frozen)
HTML5:             audio.currentTime
```

### Pause / Resume on the WebAudio path

`AudioBufferSourceNode` is one-shot — it cannot be paused and restarted. Every
play/resume/seek creates a new node; the decoded `AudioBuffer` is retained and
reused.

On pause:
- Record `pausedAtTrackTime = currentTime`
- `sourceNode.stop()` + `disconnect()`
- Set `_webAudioPaused = true`

On resume:
- Create a fresh `AudioBufferSourceNode`
- `sourceNode.start(0, pausedAtTrackTime)`
- `webAudioStartedAt = ctx.currentTime - pausedAtTrackTime`

---

## Fallback

If Web Audio is unavailable (`webAudioIsDisabled: true`), decoding fails, or the
buffer isn't ready in time, playback falls back to HTML5 Audio. There will be a
small gap between tracks in this case. All WebAudio paths guard
`if (!this.ctx) return` and silently fall back to HTML5 until the context exists.

---

## Gapless-Scheduling Bug History

These three bugs are specifically about getting a gapless-scheduled track to
actually play *and* report progress. All are covered by regression tests.

### `scheduleGaplessStart` sent the wrong machine event (`'PLAY'` instead of `'PLAY_WEBAUDIO'`)

**Symptom:** After a gapless transition, audio played (the WebAudio source node
ran) but `Track.currentTime` returned 0 and never advanced — progress bar frozen.

**Root cause:** `'PLAY'` transitions the machine to `html5` state. In `html5`,
`_isUsingWebAudio` is false, so `currentTime` read from `audio.currentTime` — the
HTML5 element, which was still at 0 because it was never used for this track.

**Fix:** `scheduleGaplessStart` sends `'PLAY_WEBAUDIO'`.

### `scheduleGaplessStart` never started the progress loop

**Symptom:** Companion to the above. Even with the machine state fixed,
`onProgress` was never called for gapless-started tracks — "Now Playing" title
and timestamp stayed frozen on the previous track.

**Root cause:** `scheduleGaplessStart` armed the node and updated machine state
but never called `_startProgressLoop()`. `Queue.onTrackEnded`'s gapless branch
(`_scheduledIndices.has(cur.index)`) called `cur.play()` for non-scheduled tracks
but had no equivalent for scheduled ones.

**Fix:** `Queue.onTrackEnded`'s gapless branch calls `cur.startProgressLoop()`
(renamed from the private `_startProgressLoop` so the Queue can call it). The
loop then drives `onProgress` as normal.

### `PLAY_WEBAUDIO` silently dropped in `webaudio` state → `isPlaying` never true

**Symptom:** When the next track's buffer had already finished decoding (fast
network / short track), `onProgress` still never fired even with the progress
loop fix. The loop started, then exited immediately.

**Root cause:** The progress loop exits when `!this.isPlaying`, where `isPlaying`
is machine context. Sequence:
1. Track preloads: `idle` → `loading` (via `PRELOAD`)
2. Decode completes: `loading` → `webaudio` (via `BUFFER_READY`) — `isPlaying`
   stays `false` (decoded but not yet audible)
3. `scheduleGaplessStart` sends `'PLAY_WEBAUDIO'` — but the machine is already in
   `webaudio` and had **no `PLAY_WEBAUDIO` handler**. Event dropped. `isPlaying`
   stays `false`.
4. `startProgressLoop` checks `!this.isPlaying` → `true` → exits immediately.

**Fix:** Add a `PLAY_WEBAUDIO` handler in `webaudio` that sets `isPlaying: true`,
representing the moment `scheduleGaplessStart` arms the node and it will
imminently produce sound.

---

## Constraints That Bound Gapless

- **Single AudioContext.** Required so `AudioContext.currentTime` is a common
  clock for scheduling.
- **CORS required for WebAudio.** `fetch()` + `decodeAudioData` needs CORS
  headers from the audio server. Without them, tracks silently fall back to
  HTML5 — no gapless.
- **One-shot source nodes.** `AudioBufferSourceNode` can't be restarted; every
  play/resume/seek makes a new node. The PCM `AudioBuffer` is reused.
- **Buffer must be decoded before the current track ends.** Otherwise the queue
  falls back to HTML5 `audio.play()` for the next track and there's a gap.
  Preloading starts within the last 25 seconds.
