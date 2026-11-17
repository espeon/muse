# CURRENT

## 2026-11-16 — phase 6 and first-user admin

- First user ever to sign in becomes admin (`maki/src/api/auth/login.rs`), guarded by an advisory
  transaction lock so concurrent first sign-ins can't create two admins. Existing DB unchanged.
- New `album_alias` table + MusicBrainz release-group alias job; aliases exposed on `GET /album/{id}`
  and matched in `search_songs` alongside artist aliases.
- Per-user `users.name_display` (original/romanised/english) with `POST /api/v1/me/display`;
  muse-web Settings lets the user pick it and detail pages pick aliases via `displayName()`.
- muse-web `/admin` page: enrichment queue status + bulk retry, identity review list + dismiss,
  album match candidates + apply. Linked from Settings for admins only.

Migration `20261116120000_phase6_aliases_and_display.sql`. Verified: 78 maki tests pass,
`cargo sqlx prepare --check` clean, muse-web typecheck and build clean. No live MB call made for
the new alias endpoint.

## 2026-11-16 — Last.fm client fixes from real-run failures

Two terminally failed `artist_profile` jobs were client bugs, not provider outages:
- `src/metadata/fm.rs` interpolated the artist name raw into the query string, so "Paul McCartney
  & Wings" injected an extra parameter and Last.fm returned an error document that failed JSON
  decoding. Artist and API key are now form-encoded; non-2xx responses fail explicitly.
- Last.fm's `error` response (e.g. "artist not found" for Clara Lösel) has no `artist` field, which
  previously failed deserialization. `FmSearchResult.artist` is now optional and a missing profile
  returns `Ok(None)`. `enrich_artist_profile` uses `COALESCE` for bio/tags so a miss never erases
  existing data, and only replaces similar artists when Last.fm returned a profile.
- Deliberately did NOT split "Paul McCartney & Wings": `&` is part of the band name and the
  splitter's comment explains why it is excluded.

78 tests pass, sqlx check clean, binary rebuilt. The two failed jobs are still `failed`; requeue
them after restarting into the new binary.

## 2026-11-16 — bulk enrichment retry

Added `POST /api/v1/admin/enrichment/retry-failed` (admin auth), which requeues every terminally
failed job with `attempts = 0` and staggers `not_before` by one second per job to avoid a burst
against a recovered provider. Regenerated `.sqlx`, 76 tests pass, binary rebuilt.

During the user's first real run, 13 jobs failed with MusicBrainz 503s. They were requeued with the
same SQL the endpoint uses; the queue now shows 677 done / 13 pending / 0 failed.

## 2026-11-16 — final all-lens review

Final review with `codex/gpt-6-luna` across breaker/simplifier/operator. One verified correctness
bug: a single-file watcher event saved a one-track `Candidate` with `complete: true`, which set
the album's `scan_complete` and could let a full-album MB match run on a partial tracklist.
`Candidate.complete` is now `Option<bool>`: full scans pass `Some(bool)`, watcher events pass
`None`, and the album update uses `COALESCE(scan_complete, $2)` (new watcher-created albums default
to incomplete). `maki set-library-root` now tells the operator that a full scan is the next step.

Rejected as false: the operator's claim that `lease_secs / 3` can be zero — config validation
requires >= 30s, so the derived interval is >= 10s. Also left as-is (pre-existing, low impact):
bounded watcher channel with `unwrap()` in the callback; done-job retention; the initial
search-key backfill's non-concurrent index build (those indexes are dropped by a later migration).

Verification after the fix: 76 tests pass offline, online check clean,
`cargo sqlx prepare --check` exit 0, binary rebuilt. Still no live MusicBrainz call.

## 2026-11-16 — heavy-review fixes for phases 4–5

A heavy three-lens review found a critical deserialization bug and a set of required issues. Fixed:

- `MbTrack.disc` lacked `#[serde(default)]`; MB track JSON has no `disc`, so every release detail
  failed to deserialize. Added the default and kept medium-position injection in `get_release`.
- `song.mbid` now gets nested `recording.id`, never the release-track ID. Matching pairs tracks by
  `(disc, position)` using maps rather than ordinal zip; local zero durations no longer count as
  comparable evidence; non-finite scores are rejected.
- `apply_release_match` shared by auto-match and admin apply: disc matching uses `COALESCE(disc, 1)`,
  release-group MBIDs are preserved with `COALESCE`, and a partial application (fewer rows updated
  than release tracks) aborts the transaction.
- Auto-apply skips albums with tagged/admin-selected identities, requires >= 2 comparable tracks,
  and requires a configurable margin (`album_match_min_margin`, default 0.05) over the runner-up.
- Weak MB search candidates (score < 50) are skipped before their 1 req/s release lookup; stale
  candidates are deleted each run.
- Alias import is requeued when an artist gains an MBID; ambiguous aliases and duplicate artist
  names are recorded in `identity_review` and routed to Unknown Artist instead of merged; MBID
  lookup errors are propagated. Review records use stable targets and refresh `created_at`.
- Lucene query escaping added to all MB searches. Added admin endpoints to list and resolve
  identity reviews.

Verification: 76 tests pass offline, online check clean, `cargo sqlx prepare --check` exit 0.
No live MusicBrainz call was made.

## 2026-11-16 — phases 4 and 5 implemented

- **Phase 4**: new `artist_alias` table keyed by `search_key(alias)`; `artist_foc` resolves by
  MBID, exact name, then alias key, recording ambiguous aliases in `identity_review` instead of
  merging. New `artist_alias` enrichment job imports MusicBrainz aliases (`inc=aliases`) with
  locale/script/kind; `GET /artist/{id}` now includes a canonical name plus aliases.
- **Phase 5.0**: `album.release_group_mbid` and `album.mbid_source` added; existing MBIDs marked
  `legacy-unverified`; tagged MBIDs are `tag-release`, matches are `match`. CAA art now tries the
  release endpoint then the release-group endpoint.
- **Phase 5.1–5.2**: MusicBrainz release search with scores and full release tracklists; new pure
  `src/match_album.rs` scores track count, median duration delta, album title and per-track titles.
- **Phase 5.3–5.4**: `album_match_candidate` table with per-release upsert; ambiguous matches are
  stored for review, confident matches are applied transactionally (`album.mbid`,
  `release_group_mbid`, per-track `song.mbid`) without rewriting titles. Admin endpoints
  `GET/POST /admin/albums/{id}/matches` expose candidates and apply a chosen one.
- New config: `album_match_min_score` (0.78), `album_match_candidate_floor` (0.40), validated.

Verification: 74 tests pass offline (4 new scoring tests), online check clean,
`cargo sqlx prepare --check` exit 0. No live MusicBrainz call or end-to-end match was run — the
network paths are unverified beyond compile and unit tests.

## 2026-11-16 — third code-review round

Third three-lens review (different models) found 2 critical and 3 required issues in the previous
round's fixes. Fixed:

- Trust-on-first-use persisted even after the last fix: the first clean scan of a populated DB
  still *seeded* the guard from whatever root it was given. Now the guard is never inferred from
  a scan of a populated DB. A new CLI subcommand, `maki set-library-root --path <dir>`, is the
  explicit way to accept a root (also the recovery path after a legitimate root change, which
  previously dead-ended). For an empty DB the guard is still seeded automatically.
- Directory-walk and candidate-flush failures did not exclude their folders from the prune.
  Exclusions now cover walk failures, flush failures, parse failures, and the 500-track cap.
- The prune fuse counted rows that folder exclusions would have protected, so a permanently
  failing folder could refuse the whole prune. It now counts eligible rows using the same
  predicate as the delete; prefix checks require a `/` boundary so sibling folders are not
  over-protected.
- MBID handling was still racy and cleared IDs on changed sources via COALESCE. The UPDATE now
  uses `CASE WHEN $12 THEN mbid ELSE $13 END`, preserving the current DB value atomically only
  when the source is unchanged, and clearing it when the bytes changed.
- Skip/refusal detail now includes the exact `maki set-library-root` command or the old/new root.
  `record_prune` writes all three keys in one statement. Nits cleaned: redundant startup reclaim,
  duplicated concurrency clamp, divergent interval floors, unjustified timeout/lease coupling,
  and a comment documenting the accepted lost-lease window.

Verification: 70 tests pass offline, online check clean, `cargo sqlx prepare --check` exit 0,
binary builds and `maki --help` lists `set-library-root`. No live scan was run this round.

## 2026-11-16 — second code-review round

A second three-lens review found new issues in the first round's fixes. Addressed:

- **Trust-on-first-use hole in the prune guard**: with no `index_state.library_root` row, any root
  matched. A scan of a subtree on a populated DB could prune up to `max_prune_fraction` of the
  library. Now a populated DB with no guard refuses the prune, seeds the guard from the clean
  scan, and only prunes on later scans. Root-guard read errors also fail closed.
- `files_seen` counted non-audio files; a subtree with only a README could pass the guard. Now
  `audio_files_seen` is tracked separately.
- Stale pruning was a library-wide kill switch: one failing file blocked it forever and prevented
  guard seeding. Failed folders are now collected and excluded from the prune instead.
- MBID preservation had a read-then-update race; the UPDATE uses `COALESCE($12, mbid)`, and a
  changed source file clears the old MBID so enrichment re-runs.
- Added configurable `enrichment_job_timeout_secs` (default 900), a single reclaim task instead of
  one per slot, lost-lease logging, and cancellation of a handler when its lease is lost.
- Config values are now range-validated (`max_prune_fraction` in (0,1], lease >= 30s).
- Prune decisions are persisted to `index_state` (`last_prune_result/detail/at`) and the refusal
  message names the config key.

Verification: 70 tests pass offline, online check clean, `cargo sqlx prepare --check` exit 0.
Live smoke: a populated DB with the guard row removed logged "library root is not established on
a populated DB; skipping prune", kept both songs, and seeded the guard. Test rows and state were
cleaned up (`song`, `enrichment_job`, `index_state` all empty).

## 2026-11-16 — code-review fixes applied

Three-lens review (breaker/simplifier/operator, different models) found two critical data-loss
paths and a set of required issues. All were addressed:

- Pruning now requires a non-empty scan and a persisted `index_state.library_root` match; a
  prune exceeding `max_prune_fraction` (default 0.5) of the library is refused; tracks skipped
  by the 500-track cap count as failures so they cannot become stale-prune candidates. Added
  three migrations: `20261116100000_prune_guard.sql`, `20261116101000_job_leases.sql`,
  `20261116102000_drop_search_indexes.sql`, `20261116103000_drop_unused_columns.sql`.
- `delete_songs_under` uses `starts_with` (no LIKE wildcard bug) and runs orphan cleanup once;
  watcher directory removals now use the event's `RemoveKind` instead of `is_dir()`.
- `save_candidate` is one transaction across all helpers; name+artist album reuse is limited to
  legacy rows with `edition_key IS NULL`; enriched song MBIDs survive tagless rescans.
- Provider 5xx/429 now fails an enrichment job instead of being recorded as "no match";
  Last.fm failures no longer overwrite a bio with a placeholder; album-art failures retry.
- Jobs have claim tokens, leased ownership with heartbeat renewal, periodic reclaim, supervised
  worker slots with panic logging, an admin reset that can release `running` jobs, and
  invalidation when input metadata changes.
- Folder hints now derive from the resolved album folder, so bare `Disc N` subfolders yield the
  parent album name; disc precedence is single-source.
- Search-key staleness is logged at startup; reindex is batched (500/page); the three unusable
  btree indexes were dropped.
- Nits: removed write-only `metadata_source` columns, the 25565 sentinel, dead `song_artist`
  join, the 20-arm picture-type trait, dead binding, double sanitization, and duplication in
  config defaults (`Config::default()` now used by `cmd_tags`).

Verification: `cargo check --all-targets` clean online and offline; 71 tests pass offline;
`cargo sqlx prepare --check` exit 0. Live smoke test with two bare `Disc N` folders produced one
album named `Album` with discs 1 and 2; a following scan of an empty, different root logged
"scan found no audio files" and left both songs intact. All test rows, jobs, and the guard row
were removed; `song`, `enrichment_job`, and `index_state` are empty again.

## 2026-11-16 — phases 1–3 implemented (resilient scan, album folders, unicode search)

### What landed

- **Phase 1.** `metadata/fallback.rs` derives track number/title/disc from file paths and
  artist/album/year from folder names; no transliteration. `metadata/mod.rs::apply_fallbacks`
  fills only empty tag fields and records `MetadataSource::{Tag,Folder}`. FLAC files with no
  VORBIS_COMMENT no longer fail. `index/db.rs` refuses empty identity fields.
- **Real bug found by smoke test:** `id3::partial_tag_ok` does **not** turn `NoTag` into an empty
  tag, so every untagged WAV/AIFF/MP3 was still rejected. Added
  `metadata::read_id3_or_empty` and used it in all three readers.
- **Phase 2.** `index/folder.rs` resolves album folders (incl. `Disc 1` / `Album (Disc 2)`
  collapsing, loose root files), `index/mod.rs` reads a whole folder before writing it, caps a
  candidate at 500 tracks, marks failed folders `scan_complete = false`, and the watcher now
  handles all event paths, directory events, and files still being copied (10 × 200 ms settle).
  `db::save_candidate` resolves the album artist once and inserts one album per folder.
  Migration backfills `edition_key` per song folder; albums spanning multiple folders are
  marked incomplete rather than guessed.
- **Phase 3.** `helpers/normalize.rs` (`search_key`, `search_query`), `search_key` columns and
  version on artist/album/song, write paths set version 1, SQL backfill sets version 0,
  `maki reindex-search` recomputes exact keys idempotently. Search endpoint matches on
  `search_key ILIKE … ESCAPE '\'`; response shape unchanged.
- Stale pruning now also requires the scan root to equal `MOUNT`, closing the hole that caused
  the earlier deletion (plan risk #7).

### Verification performed

- `cargo check --all-targets` clean online and offline; `cargo test` **71 passed** in both modes;
  `cargo sqlx prepare --check -- --all-targets` exit 0.
- Live two-folder scan of four untagged WAVs with Japanese/Chinese filenames: 2 albums, 4 songs,
  `metadata_source='folder'`, correct track numbers, `edition_key` per folder, all
  `scan_complete=true`, and normalized search keys. Queried
  `search_key ILIKE '%badもーど%'` and got the expected row. Test rows/jobs and temp dir removed
  afterwards; `song` and `enrichment_job` are empty again.
- No HTTP-level search regression test was added (the unit tests cover `search_key` and I
  verified the `ESCAPE '\'` SQL directly). Flagging that as a gap, not claiming it was tested.

### Still open

- No `album_match` job / `album_match_candidate` table yet (phase 5).
- Search-key backfill for pre-existing rows is approximate (version 0); run
  `maki reindex-search` if you keep rows that were not re-scanned.
- The real library still needs the full rescan you approved; do that after phase 4/5 if you
  want to test the whole pipeline on the 898-file workload.

## 2026-11-16 — phase 0 implemented, plus a dev-DB incident

### Phase 0 done (see `maki/PLAN.md` §3)

- New migration `20261116000000_enrichment_jobs.sql`: `enrichment_job(kind, target, state,
  attempts, not_before, last_error, ...)`, `UNIQUE (kind, target)`, pending index.
- `src/enrich.rs`: durable queue with atomic `FOR UPDATE SKIP LOCKED` claim, success/retry/failed
  transitions, exponential backoff, lease-based recovery of `running` jobs after a crash,
  dedupe on re-scan. Four job kinds implemented: `artist_mbid`, `artist_profile`, `album_art`,
  `track_mbid`. No `album_match` job yet (phase 5 owns it).
- `index/db.rs`: `add_song`/`artist_foc`/`album_foc`/`song_foc` are now DB-only. Every network
  call moved into four `enrich_*` handlers at the bottom of the file. The misleading comment
  about "release-group lookup" was removed.
- `config.rs`: `enrichment_enabled`, `enrichment_concurrency`, `enrichment_max_attempts`,
  `enrichment_lease_secs` (all with serde defaults, so existing config files still load).
  `main.rs` spawns the worker unless serving is disabled.
- `api/admin.rs`: `GET /api/v1/admin/enrichment` and `POST /api/v1/admin/enrichment/{id}/retry`,
  behind the existing `AdminUser` extractor (which is where the 403 comes from; no separate
  auth test was written — be accurate about that).
- `index/mod.rs`: stale prune now skipped if any file in the walk failed. Added pure
  `should_prune(failures)` + test.
- Also fixed a latent online-build failure at `api/index.rs:40`: with sqlx 0.9, `query_as!` over
  two `LEFT JOIN`s inferred `Option<String>` for the non-Option `IndexSong` fields. Both joins
  are now INNER (both FKs are NOT NULL), so the API JSON shape is unchanged.
- Verification: `cargo check --all-targets` clean online and offline; `cargo test` 52 passed in
  both modes; `cargo sqlx prepare --check -- --all-targets` exit 0. A live scan of one FLAC
  imported the song and queued exactly `album_art`, `artist_mbid`, `artist_profile`; no HTTP
  calls appeared in the log.

### INCIDENT — the smoke test deleted 853 song rows from the dev DB

I started the built binary with `MOUNT` pointed at my temporary one-file fixture instead of the
real library. The existing `delete_stale_songs` sweep treats "not stamped during this scan" as
"removed from disk", so a successful scan of a *partial* tree deleted **853 rows from `song`**
in container `muse-maki-db-1`. The deletes of `playlist_item` links and `favorites` both affected
0 rows, so no playlist or favourite data was lost. Play counts / `last_play` on those 853 rows
are gone. The underlying 898 audio files are still on disk at `/Users/natalie/Music/Music`.

I removed my test song, its album/artist and its three queued jobs; `song` and `enrichment_job`
are now both empty. I did **not** re-scan the real library, because that recreates song rows with
new ids and still cannot restore play counts. That needs your decision.

Also observed in the log: `cleanup_orphans` fails with an `album_genre_album_fkey` violation
after a prune (pre-existing bug, not introduced by phase 0).

The phase 0.4 fix only guards *failed files*; it does not guard a successful scan with the wrong
`MOUNT`. `delete_stale_songs` fundamentally assumes MOUNT is the entire library. This is now
recorded as a plan risk.

## 2026-11-16 — maki builds again; plan for multilingual album-aware ingestion

### Plan

`maki/PLAN.md` — plan only, nothing from it implemented yet. Album-first identification with
multilingual entity matching, revised against the actual code. Three changes from the draft
plan the user supplied:

1. No release-group/release/recording entity split. `album` already holds a MusicBrainz
   *release*; editions get grouped by one new column, `album.release_group_mbid`.
2. No local transliteration subsystem. 周杰倫 → Jay Chou is an alias, not a reading; kana
   romanisation needs segmentation; pinyin needs a dictionary. Use MusicBrainz aliases
   (`inc=aliases`, already rate-limited) plus the two exact local transforms (NFKC and the
   katakana↔hiragana codepoint offset) for search keys.
3. The P0 is not album matching, it is that `add_song` performs HTTP inside the sequential
   scan loop (MusicBrainz at 1 req/s, Last.fm, Deezer, TheAudioDB, Spotify, cover art, plus a
   per-song MB search). Album-first matching needs a two-pass scan, so extraction has to be
   split from enrichment first.

Also cut: a `raw_file` staging table (pass 1 in `album`/`song` keyed by `album.edition_key`
already is one), and `unaccent()` in the search path (a Rust `search_key` replaces it and
handles full-width forms it cannot).

### Plan review (2026-11-16)

`plan-reviewer` reviewed PLAN.md; §8 of the plan records what changed. Headline: its P0 was
wrong (`get_release_group_info` *is* a release lookup — the name says what it returns, and a
misleading comment at `db.rs:325` caused the misreading), but investigating it surfaced a real
P0: `album.mbid` holds a **release** MBID for tagged files and a **release-group** MBID for
search-matched albums, and cover art always queries the release endpoint. That is now phase 5.0
with an `album.mbid_source` column. Twelve P1/P2 findings were accepted: no migration policy for
pre-existing/merged-edition albums, an unsafe stale-prune under partial scans (pre-existing bug),
`analysis.rs` is not a queue so the job lifecycle is now specified, candidate completeness,
watcher directory/multi-path/half-copy events, alias key + collision contract, candidate
lifecycle, unscheduled admin surface, `album_key` grouping, search reindex strategy, enumerated
name-write paths, and the search API contract as acceptance criteria.

### Build fix (pre-existing breakage, not part of the plan)

`cargo check --all-targets` and `cargo test` now pass, both against a live DB and with
`SQLX_OFFLINE=true`. 50 tests pass in both modes.

- `src/api/song.rs` lastfm `set_now_playing` / `scrobble`: sqlx 0.9 infers projected columns of
  a `LEFT JOIN` query **inconsistently** — plain SQL gives a mix (`song.name` and
  `artist.name` as `Option<String>`, but `album.name` as `String`). Confirmed against
  `information_schema` that `song.name`, `song.duration`, `album.name`, `artist.name` are all
  `NOT NULL`, so none of this is real nullability. Wrapping all four in `COALESCE` makes them
  uniformly `Option`, which is the only shape that can be handled uniformly at the call sites;
  the `COALESCE` is an inference workaround, not a NULL guard, and the comment in the file says
  so. Call sites use `.as_deref().unwrap_or("")` / `.unwrap_or_default() as u32`.
- Regenerated `maki/.sqlx` with sqlx-cli **0.9.0** (the installed global CLI was 0.8.6, which
  produces cache data sqlx 0.9 rejects). 58 modified + 18 added + 3 orphaned entries deleted.
  The global CLI is now **0.9.0**, so plain `cargo sqlx prepare` is correct from here on;
  confirmed with `cargo sqlx prepare --check -- --all-targets` (exit 0).

### Verification performed

- `cargo check --all-targets`: clean, online and offline. One pre-existing warning
  (`AdminUser::payload` unused).
- `cargo test`: 50 passed, online and offline.
- `git diff maki/src/api/song.rs`: only the two lastfm call sites (4 lines each), the two
  `SELECT` blocks, and two explanatory comments changed by me, on top of the user's
  in-progress `Host` extractor move in the same file (left untouched). CRLF preserved
  (859/859 lines).

## 2026-07-19 — general dependency update

Scope agreed with user: **maki (Rust) + muse-web (pnpm)**, bumping all the way to
latest majors. nozomi, umi, eli, maki-analyzer deliberately left alone.

### Baseline (before any changes)

- **maki**: compiles cleanly. `cargo check --all-targets` succeeds, 3 pre-existing
  dead-code warnings (`AdminUser::payload`, `HmacMessage::decode`, `verify_hmac`).
- **muse-web**: `vite build` succeeds. `tsc --noEmit` fails with 2 *pre-existing*
  errors:
  - `src/views/AuthCallback.tsx:16` — `class` prop on `<svg>` (React types want `className`)
  - `vite.config.ts:1` — `Cannot find module 'node:url'` (no `@types/node` dep)

### Environment blockers found (not dep problems)

1. Global git config has `url.ssh://git@github.com/.insteadof https://github.com/`
   and ssh-agent has no identities, so cargo could not fetch the
   `[patch.crates-io] darwin-libproc` git dep. Workaround for local builds only:
   `GIT_CONFIG_GLOBAL=/dev/null CARGO_NET_GIT_FETCH_WITH_CLI=true cargo ...`
   (user's `~/.gitconfig` untouched).
2. `maki/.sqlx` offline cache is **stale** — 10 queries have no cached data, so
   `SQLX_OFFLINE=true` does not build. Baseline check therefore uses a live DB.
3. To build maki locally you need Postgres at `localhost:5438` (per `.env`) with
   all `maki/migrations/*.sql` applied. Used a throwaway container
   `muse-depcheck-db` (postgres:17) and applied migrations in filename order.
   Required env for checks: `set -a && . ../.env && set +a`, and `unset SQLX_OFFLINE`.

### Progress

**muse-web** — done, verified.
- `pnpm up --latest`, all deps to latest (react 19.3.1, vite 8.3.7, typescript 7.0.2,
  @vitejs/plugin-react 6.3.0, lucide-react 1.70.0, @base-ui/react 1.9.0, ...).
- Removed `baseUrl` from `tsconfig.json` (TS 7 rejects it).
- Added `@types/node` (dev) — `vite.config.ts` imports `node:url` and typecheck never
  passed without it.
- Fixed `src/views/AuthCallback.tsx`: `class`/`stop-color` are not valid React props, so
  the icon was rendering unstyled. Now `className`/`stopColor`.
- `pnpm typecheck` clean (baseline had 2 errors), `pnpm build` OK (vite 8).

**maki** — waves 1 and 2 done, compiles, 49 tests pass.
- Wave 1 (all non-axum, non-sqlx deps) bumped to latest.
- Held back deliberately:
  - `sha3` 0.10.8 — 0.12 removed `Shake128` (XOF moved out of the crate); code needs it.
  - `md-5` 0.10.5 — 0.11 (digest 0.11) dropped `LowerHex` on digest output, which
    `format!("{:x}", ..)` relies on. Bumping would also add a second `digest` major.
- Wave 2: axum 0.7→0.8, axum-macros 0.5, tower-http 0.7, tower 0.5, utoipa 4→6.
  - `axum::async_trait` removed → dropped `#[async_trait]` from the 4 `FromRequestParts`
    impls (axum 0.8 uses native async fns).
  - `Message::Text(String)` → `Message::Text(Utf8Bytes)` in `remote/ws.rs`.
  - Route paths rewritten to axum 0.8 syntax: `:id`→`{id}`, `*path`→`{*path}`
    (22 paths in `api/mod.rs` and `main.rs`). This is a *runtime-only* failure mode,
    compile does not catch it.
  - `axum::extract::Host` no longer exists and axum-extra has no replacement, so added
    `src/api/middleware/host.rs`: a port of axum 0.7's extractor preserving the
    `Forwarded` → `X-Forwarded-Host` → `Host` → URI precedence. 5 unit tests.
- Note: `src/api/index.rs` and `src/api/song.rs` are CRLF; an edit normalised them to LF
  and inflated the diff, which was reverted. Keep CRLF on those two files.

### maki wave 3-4 (sqlx 0.9, reqwest 0.13, oauth2 5.0) — done

- **sqlx 0.7 → 0.9.** Features `runtime-tokio-rustls` → `runtime-tokio` +
  `tls-rustls-ring-webpki`.
  - sqlx 0.9 added the `SqlSafeStr` guard, which rejects dynamic SQL strings. It caught
    a **real SQL injection** in `api/index.rs::search_songs`: `slug`, `sortby`, and `dir`
    were interpolated straight into the query text. Fixed by binding `slug` as `$1` and
    whitelisting `dir` to asc/desc (`sortby` was already a closed enum). The remaining
    `format!` is wrapped in `AssertSqlSafe` with a comment explaining the audit.
  - sqlx 0.9 also tightened nullability inference for `LEFT JOIN` columns, so
    `album.name`/`artist.name` in `set_playing`/`scrobble_song` are now `Option<String>`.
    Passed through with `as_deref().unwrap_or("")` (Last.fm treats empty as absent).
  - `api/analysis.rs` `pending_songs` builds SQL via `format!` with a static fragment
    chosen by a match, so it got `AssertSqlSafe` too.
  - Checked the rewritten search SQL with `PREPARE`/`EXECUTE` against Postgres.
- **reqwest 0.11 → 0.13.** `rustls-tls` → `rustls`; `query`/`form` are now separate
  features and both were needed. Dropped the unused `blocking` feature.
- **oauth2 4.4 → 5.0.** Bigger rework:
  - `BasicClient::new(id, secret, auth, token)` → builder (`Client::new(id)` +
    `set_client_secret`/`set_auth_uri`/`set_token_uri`/`set_redirect_uri`).
  - oauth2 5 tracks configured endpoints in the type, so `providers/mod.rs` now has an
    `OidcClient` alias that has to be spelled out in the `AuthProvider` trait.
  - `async_http_client` is gone. oauth2 is built with `default-features = false` and
    `src/api/auth/oauth_http.rs` implements `AsyncHttpClient` over our own reqwest 0.13
    client. Its bundled `reqwest` feature pins reqwest **0.12**, which would otherwise
    compile a second HTTP stack; `cargo tree -i reqwest` now shows a single 0.13.5.

### Verification actually performed

- `cargo build` + `cargo test`: 50 tests pass (49 pre-existing + 1 new route-table test).
  Same 3 pre-existing dead-code warnings, no new ones.
- Added `routes()` in `main.rs`, split out of `serve()`, with a test that builds it.
  Confirmed this test has teeth: axum 0.8 panics on `:id` with "Path segments must not
  start with `:`", so a missed route rewrite fails the suite.
- Ran the real binary against a fresh Postgres (migrations applied by maki itself).
  Every endpoint I touched responded correctly, including
  `/api/v1/track/{id}/similar`, nested `/api/v1/track/{id}/hls/{profile}`, and
  `/api/v1/search/{slug}`. SQL-injection payloads in `slug` and `dir` return normally
  and leave the `song` table intact.
- `rustfmt --check` clean on every file I touched that was clean beforehand; the repo as
  a whole is not fmt-clean, so no wholesale reformat was done. `index.rs` and `song.rs`
  are CRLF and were kept CRLF (rustfmt normalises on write; had to restore).
- muse-web: `pnpm typecheck` clean, `pnpm build` OK.

### Known leftovers (not touched, outside scope)

- `maki/.sqlx` offline query cache is stale (10 queries missing), so `SQLX_OFFLINE=true`
  builds fail. Needs `cargo sqlx prepare` with a live DB.
- maki `Cargo.toml` still declares `edition = "2018"`.
- Building maki locally needs the git-config workaround for the `darwin-libproc` patch.
