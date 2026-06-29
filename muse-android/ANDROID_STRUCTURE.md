here's a plan to port muse to android. i'll keep it parallel to the ios app's structure so the two stay in sync as the api evolves.

## stack decisions

| concern | ios (today) | android (proposed) | why |
|---|---|---|---|
| language | swift | kotlin | first-class on android, coroutines map cleanly to swift concurrency |
| ui | swiftui | jetpack compose | declarative parallel, compose 1.7+ has strong perf |
| min/target sdk | ios 26 | minSdk 28 (a9), targetSdk 35 (a15) | minSdk 28 unlocks predictive back, modern crypto, drops most legacy cruft. raise to 31 if we want splashscreen api without compat |
| audio | AVQueuePlayer | media3 exoplayer + mediasessionservice | hls/dash native, gapless via setMediaItems + preload manager, full bluetooth/auto/wear story |
| auth | ASWebAuthenticationSession + keychain | appauth-android (chrome custom tabs) + encryptedsharedpreferences/datastore | mirrors the oidc pkce flow already on the server |
| http | URLSession + Codable | ktor client + kotlinx.serialization | simpler than retrofit for this shape; one decoder config matches `convertFromSnakeCase` |
| images | custom ImageCache | coil 3 | compose-native, disk + memory cache, handles webp |
| lyrics | custom LRCParser | port LRCParser line-for-line to kotlin | tiny, no dep needed |
| artwork shader | Shaders.metal | AGSL via `RuntimeShader` (a13+) | direct equivalent. fallback to a gradient on older devices |
| nav | swiftui navigation | navigation-compose (type-safe routes) | |
| di | env values | koin | lighter than hilt for a single-module app |

## module layout (single `:app` module to start)

```
app/src/main/java/vg/nat/muse/
  MuseApplication.kt
  MainActivity.kt
  net/
    ApiClient.kt          ← mirrors APIClient.swift
    AuthManager.kt        ← mirrors AuthManager.swift
    UmiClient.kt
    Models.kt             ← Track, Album, Artist, Playlist, HomeRow, HLSProfile
  player/
    PlayerEngine.kt       ← wraps ExoPlayer + MediaSession
    PlaybackService.kt    ← MediaSessionService subclass
    HlsQualityController.kt
  lyrics/
    LrcParser.kt
    SyncedLyricsScreen.kt
    TranslationService.kt
  ui/
    theme/Theme.kt
    components/ (ArtworkImage, MarqueeText, AlbumCard, TrackRow, DynamicArtworkBackground)
    home/HomeScreen.kt
    library/{Albums,Artists,Playlists,Library}Screen.kt
    detail/{Album,Artist,Playlist}DetailScreen.kt
    search/SearchScreen.kt
    player/{MiniPlayer,FullPlayer,PlayerControlBar,NowPlayingSheet}.kt
    settings/SettingsScreen.kt
    auth/LoginScreen.kt
    RootScaffold.kt
```

## phased build

**phase 0 — scaffolding (½ day)**
- new project, agp 8.7+, kotlin 2.0, compose bom, jvmTarget 17
- gradle version catalog (`libs.versions.toml`) with ktor, media3, coil, appauth, koin, kotlinx-serialization
- ci: github actions workflow mirroring the existing one for builds

**phase 1 — networking + auth (1–2 days)**
- port `Models.swift` files to `Models.kt` data classes with `@Serializable`
- `ApiClient` with the same surface (`fetchHome`, `fetchAlbums`, `batchSignTracks`, etc.)
- json config: `decodeEnumsCaseInsensitive`, snake_case via `JsonNamingStrategy.SnakeCase`
- date handling matches the ios custom decoder (try double, then iso8601 with/without fractional seconds)
- `AuthManager` with appauth-android, register `muse://` deep link in manifest, store tokens in `EncryptedSharedPreferences`
- 401 retry-on-refresh interceptor

**phase 2 — playback (2–3 days)**
- `PlaybackService : MediaSessionService` with foreground notification
- `PlayerEngine` wraps an `ExoPlayer` instance, exposes a `StateFlow<PlayerState>`
- gapless: build the queue as `MediaItem`s, set with `player.setMediaItems(items, startIndex, startPositionMs)`. media3's preload manager handles pre-buffering ahead
- batch-sign upcoming `preloadCount = 3` tracks, populate a signed-url map keyed by track id, same as ios
- hls: exoplayer detects hls automatically by extension/mime; for bitrate selection use `DefaultTrackSelector.Parameters.Builder().setMaxAudioBitrate(peakBitRate)`
- hls profile detection: read `Format.bitrate` from `AnalyticsListener.onDownstreamFormatChanged`, match to nearest `HLSProfile` (port the same algorithm)
- bluetooth/lockscreen/android auto: `MediaSession` plus a custom `MediaSession.Callback` for next/prev → `apiClient.setPlaying`
- audio focus, becoming-noisy: media3 handles both via `setAudioAttributes(.., handleAudioFocus = true)` and the built-in `BecomingNoisyReceiver`

**phase 3 — core ui (3–5 days)**
- root scaffold with bottom nav: home, library, search, settings
- home, library tabs, detail screens, search — straight port of the swiftui views
- mini player + full player sheet (modal bottom sheet from material3, or a custom `Scaffold` with `AnchoredDraggableState` for the swipe-up gesture parity)
- `ArtworkImage` via coil, `MarqueeText` via compose's `basicMarquee()` modifier (built-in since compose 1.6)
- `AlbumCard`, `TrackRow` — straightforward compose

**phase 4 — lyrics + animated background (1–2 days)**
- port `LRCParser` and `RichSyncLyricsView` (jlf)
- `DynamicArtworkBackground` via `RuntimeShader` (agsl). port the metal shader; agsl syntax is glsl-like, the metal one likely needs minor adaptation. on api < 33, fall back to a `Brush.radialGradient` from extracted palette colors (palette-ktx)

**phase 5 — polish (1–2 days)**
- predictive back gesture for the full player sheet
- material you dynamic color on a12+
- crashlytics or sentry if you want it
- proguard rules for kotlinx-serialization and media3
- play store listing assets (only if you plan to publish; otherwise apk via github releases is fine)

## open questions before i start

1. **distribution.** play store, or self-hosted apk/f-droid? affects signing setup and whether we need play services at all (we don't need any for the current feature set, which is nice)
2. **min api.** 28 is my recommendation, but if you have a specific device in mind we can go lower. dropping below 26 would mean fighting java.time and a bunch else
3. **shared models.** want me to keep ios and android model classes hand-synced, or set up a kotlin multiplatform `:shared` module with the model/api layer that both consume? kmp adds complexity but eliminates a real drift risk as the api evolves
4. **wear os / android auto / cast.** in scope for v1 or later?
5. **eli (remote control ws).** still out of scope on android too, right?

if you want, i can pick reasonable defaults (play store optional, minSdk 28, no kmp for v1, no wear/auto/cast in v1, eli out of scope) and start phase 0 + 1.
