# muse android — port plan & status

Port of the muse iOS app to Android (Kotlin + Jetpack Compose), kept structurally
parallel to the iOS app so the two stay in sync as the maki API evolves.

## stack

| concern | choice |
|---|---|
| language / ui | Kotlin 2.0.21, Jetpack Compose (BOM), compileSdk 36 / minSdk 28 |
| audio | media3 ExoPlayer + MediaSessionService (HLS via media3-exoplayer-hls) |
| auth | chrome custom tabs + `muse://` deep link; EncryptedSharedPreferences |
| http | ktor client + kotlinx.serialization (snake_case + flex date decoder) |
| images | coil3 |
| lyrics | custom (JLF model, karaoke via offscreen DstIn gradient) |
| artwork bg | AGSL RuntimeShader (palette-driven), gradient fallback < API 33 |
| nav | navigation-compose |
| di | none — CompositionLocals (KMP deferred) |

## what's done

### phase 0 — scaffolding ✅
Gradle project, version catalog, manifest, theme, CI workflow mirroring the
ios/maki ones. `5af913b`

### phase 1 — networking + auth ✅
Full model layer ported field-for-field; `ApiClient` with the entire iOS surface
and 401→refresh→retry; `AuthManager` (custom-tabs login, `muse://` redirect,
encrypted tokens, exact `Bearer authjs.session-token:` header). `5af913b`

### phase 2 — playback ✅
`PlayerEngine` (ExoPlayer wrapper, batch-sign preload window of 3, HLS quality
control + active-profile detection, audio focus + becoming-noisy); `PlaybackService`
foreground/notification driven by a persistent `MediaController` connection;
gapless via `setMediaItems`, native `previous`. `428fbcc`, `3e54c6e`, `bfbdcbc`

### phase 3 — core ui ✅
Bottom nav (home, library albums/artists/playlists, search, settings); album /
artist / playlist detail; login; settings (server url, logout, adaptive streaming
toggle, quality picker); mini player + full player. `2680446`, `d7a9f88`

### phase 4 — lyrics + animated background ✅
AGSL tonal background (palette-driven drifting blobs + darkened palette, gradient
fallback); full JLF model + `UmiClient`; karaoke synced lyrics:
per-syllable masking for rich sync (plain lines unswept), space-aware tokenization
against source text, soft gradient sweep edge, refresh-rate per-frame draw,
edge fade, forced-dark, line anchored near top. `25177f3`, `cf23df1`, …, `717f5dc`

### phase 5 — polish (partial) 🟡
Done: anchored-draggable full-bleed player sheet, predictive back driving the
sheet, marquee title/artist, forced-dark player, primary transport controls,
controls pinned to bottom (no shift on lyrics toggle), larger animated artwork
(crossfade on track change), tuned padding, lyric scroll target.

## known issues
- lyric auto-scroll on line change can still look snappy — was partly a stale-build
  artifact; verify after a clean install, revisit if it persists.
- foreground/notification + HLS runtime-verified; HLS lossless auto-scalability is
  a known parity gap vs iOS (manual quality works).

## remaining work

### next — search
Current search is song-only and plays the result as a one-track queue. Improve:
- [ ] album / artist results (sectioned, or unified results screen)
- [ ] recent searches, persisted, tap-to-repeat
- [ ] result → album/artist detail navigation
- [ ] better empty / loading / error states

### orientation / landscape
- [ ] no orientation handling yet; player + detail screens assume portrait. add
      landscape layouts (esp. full player / now-playing).

### release build / distribution
- [ ] enable release variant + proguard/minification (rules stubbed)
- [ ] signing: keystore decision (debug-signed release for now, or real keystore
      via `signingConfigs` from env/local.properties)
- [ ] distribution: self-hosted APK via GitHub releases (play store optional)

### misc polish
- [ ] splash screen (android 12+ SplashScreen API)
- [ ] queue sheet styling
- [ ] mini-player title marquee
- [ ] pull-to-refresh on home/library

### deferred
- [ ] translation via ML Kit (iOS uses Apple Translation; no direct equivalent)
- [ ] wear os / android auto / cast (out for v1; auto is cheap via MediaSession)
- [ ] eli remote-control ws (out of scope)
- [ ] KMP shared models (`net/` kept pure so extraction is a refactor, not a rewrite)

## architecture decisions (record)
- **auth**: chrome custom tabs + `muse://` intent (not AppAuth) — server-mediated,
  client never does PKCE.
- **foreground service**: persistent `MediaController` connection drives media3's
  notification/foreground lifecycle (not `startForegroundService` from `play()` —
  that caused `ForegroundServiceDidNotStartInTimeException`).
- **player sheet**: hand-rolled `Animatable` + `detectVerticalDragGestures` (not
  foundation `AnchoredDraggableState` — it has no clean programmatic "animate open").
- **background**: platform `android.graphics.RuntimeShader` + `nativeCanvas`
  (the Compose `RuntimeShader` wrapper didn't resolve at our compose version).
- **karaoke**: offscreen `DstIn` gradient mask (not a metal shader); per-syllable
  via `FlowRow` tokens.
- **lyrics spacing**: tokenize against source text so it works for both word- and
  syllable-level providers.
- **player theming**: forced `darkColorScheme()` inside the full player
  (white-on-dark artwork background).
- **models**: pure data classes + kotlinx.serialization (KMP-extractable).
