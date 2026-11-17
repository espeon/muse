# AGENTS.md

This file provides guidance to Codex (Codex.ai/code) when working with code in this repository.

## Services

| Service | Lang | Port | Role |
|---------|------|------|------|
| **maki** | Rust/Axum | 3033 | Music server — file scanning, metadata, transcoding, API, embedded web UI |
| **nozomi** | Next.js 15 | 3031 | Web frontend (being replaced by a SolidJS frontend) |
| **muse** (iOS) | Swift/SwiftUI | — | Native iOS app targeting iOS 26 |
| **umi** | external | 3032 | Optional lyrics service (Apple Music API); frontend fails silently without it |
| **eli** | Rust | 3000 | Remote control WebSocket server (out of scope for current work) |

## Development commands

### Maki (Rust)
```sh
cd maki
cargo build
cargo run
# Env vars required: MOUNT, DATABASE_URL, OIDC_CLIENT_ID, OIDC_CLIENT_SECRET,
#                    OIDC_AUTHORIZE, OIDC_ISSUER, OIDC_TOKEN, OIDC_REDIRECT
DRY_RUN=1 cargo run   # scan files without serving
NO_SCAN=1 cargo run   # serve without scanning
```

### Nozomi (Next.js)
```sh
cd nozomi
bun install
bun dev          # runs on port 3031 with turbo
bun build
bun lint
```

### Full stack (dev)
```sh
docker compose up
```

### Build and push images
```sh
./build.sh all            # builds nozomi + maki (+ umi if present)
./build.sh nozomi,maki    # specific services
./build.sh all v1.2.3     # with tag
```

### iOS app
Open `muse/muse.xcodeproj` in Xcode. Target is iOS 26.

## Maki architecture

### Embedded web UI (muse-web)

The muse-web frontend (React/Vite SPA) is embedded directly into the maki binary at compile time via rust-embed. The built muse-web/dist/ directory is included as binary assets and served by maki itself.

- src/web.rs — RustEmbed struct + spa_fallback() / static_handler() handlers
- build.rs — automatically runs pnpm build in muse-web/ if dist/ is missing (set MAKI_SKIP_WEB_BUILD=1 to skip)
- The SPA API calls are same-origin (MAKI_URL defaults to empty when no VITE_MAKI_URL env is set)
- Client-side routing works via a fallback() handler that serves index.html for all non-API routes

Maki is structured as:
- `src/main.rs` — startup, OIDC auth setup, Axum router, spawns indexer thread
- `src/api/` — route handlers: `album.rs`, `artist.rs`, `song.rs`, `playlist.rs`, `home.rs`, `serve.rs` (transcoding), `sign.rs`, `index.rs`
- `src/api/auth/` — OIDC PKCE flow
- `src/api/middleware/` — JWT auth extractor
- `src/index/` — file watcher (notify) + initial directory scan via jwalk
- `src/metadata/` — audio metadata parsing for MP3/FLAC/WAV/AIFF, MusicBrainz + Spotify clients, Last.fm scrobbling
- `src/db.rs` — PostgreSQL pool via sqlx
- `src/config.rs` — JSON config at `config/config.maki.json`

**Database/scraping are in flux** — migrations live in `maki/migrations/`; metadata clients (`musicbrainz.rs`, `spotify.rs`, `fm.rs`) are subject to change.

## Nozomi architecture

- `src/app/` — Next.js App Router pages
- `src/stores/` — Zustand state (player, queue, etc.)
- `src/components/` — UI components
- `src/helpers/` — API client utilities
- Dual-player gapless playback via two ReactPlayer instances
- Auth via next-auth v5 with pg adapter

**Nozomi is being replaced** by a new SolidJS frontend. Avoid large investments here.

## iOS app architecture (`muse/muse/`)

- `Networking/APIClient.swift` — Maki REST API calls
- `Networking/UmiClient.swift` — Lyrics service calls (optional)
- `Networking/AuthManager.swift` — OIDC auth + token management
- `Player/PlayerEngine.swift` — AVFoundation playback engine
- `Player/NowPlayingUpdater.swift` — Now Playing / lock screen integration
- `Models/` — Track, Album, Artist, Playlist, HomeModels (match Maki JSON shapes)
- `Views/` — SwiftUI views organized by feature (Home, Library, Detail, Player, Search, Auth, Settings)
- `Lyrics/` — LRC parser and synced lyrics view

**Pending:** Add `muse://` URL scheme in Xcode (Target → Info → URL Types) for deep link auth callback.

## Deployment

Production: `docker compose -f docker-compose.prod.yaml up -d` (requires `.env` from `.env.example`)

Images published to `ghcr.io/espeon/muse/<service>:<tag>`.
