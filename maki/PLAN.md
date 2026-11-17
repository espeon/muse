# maki: multilingual, album-aware ingestion

Status: **plan, nothing implemented.** Scope: `maki` (Rust) only. No muse-web / iOS changes
in phases 0–5.

Derived from the draft plan (album-first identification, multilingual entity matching),
revised against what is actually in the code today. Where I disagree with the draft, the
reason is in §1.

---

## 0. What I verified in the code

Facts below are read from the working tree, not assumed.

| # | Finding | Where |
|---|---------|-------|
| 1 | **The working tree did not compile** when this plan was written; fixed 2026-11-16 (§3 Phase 0.0). 4 type errors at `src/api/song.rs` plus 10 queries with no offline cache data. | `src/api/song.rs`; `SQLX_OFFLINE=true cargo check` |
| 2 | Scanning is a flat sequential walk. `scan_file` per file, awaited, no folder concept anywhere. | `src/index/mod.rs:48-53` |
| 3 | Tag parse → `add_song` immediately. No grouping, no scan session. | `src/metadata/mod.rs:89-131` |
| 4 | **Network I/O happens inside the scan loop.** Per new artist: MusicBrainz artist search (global 1 req/s governor), Last.fm, Deezer, TheAudioDB, Spotify, then an image download. Per album: MB release-group search + Cover Art Archive + Deezer cover. Per song without a track MBID: another MB search. All `await`ed inline, sequentially, for the whole library. | `src/index/db.rs:147-156`, `:186-236`, `:325-352`, `:383-410`, `:605-620`; governor at `src/metadata/musicbrainz.rs:22` |
| 5 | Album identity is `(name, artist)` exact string match, or `mbid`. Slug is `md5(name|artist[|mbid])`, with a `_2`, `_3` suffix walk on collision. Two editions of the same album with the same displayed name therefore collapse. | `src/index/db.rs:318-330`, `:440-478` |
| 6 | Artist identity is `mbid` or exact `name`. Slug is `md5(lowercase(name))`. 宇多田ヒカル and Hikaru Utada can never resolve to one artist. | `src/index/db.rs:14-26`, `:124-146` |
| 7 | FLAC files with no `VORBIS_COMMENT` block are rejected outright (`bail!`), so an untagged FLAC is never imported. WAV/AIFF/MP3 survive missing tags but produce `""` for name/album/album_artist, which creates an artist row named `""`. | `src/metadata/formats/flac.rs:60-63`, `mp3.rs:79-86`, `wav.rs:41-46` |
| 8 | Search is `unaccent(col) ILIKE '%q%'` on song/artist/album name. `unaccent` folds Latin diacritics only: it does not touch full-width forms, kana, or Han. | `src/api/index.rs:149-151` |
| 9 | Three mismatched fallbacks for a missing album title: FLAC `""`, MP3 `""`, and `number` falls back to `0` (FLAC) vs `25565` (ID3). | `flac.rs:119`, `mp3.rs:81` |
| 10 | `sort_string` uses NFKD and then strips English/Germanic articles. Applied to Japanese titles it is meaningless but harmless. | `src/helpers/mod.rs:34-49` |
| 11 | Content-addressed `audio_assets` (blake3) + `song.audio_hash` already exist, with `audio_similarity_features` / `audio_mix_profiles` keyed by hash. This is already a "local asset outlives the row" model. | `migrations/20260709000100_audio_assets.sql` |
| 12 | `album.mbid` and `album.disambiguation` already exist — but **`album.mbid` holds two different entity types depending on the code path**, and the cover art lookup assumes one of them. See finding 16. | `20260412000000_album_disambiguation.sql`, `db.rs:323-352` |
| 13 | `analysis.rs` is an async-orchestration precedent, **not** a job queue: pending work is derived from missing artifacts (`:252-280`), there is no `pending/running/done/failed` state, no `SKIP LOCKED`, no attempt/backoff. Phase 0.2 must specify the queue itself. | `src/analysis.rs:89-113`, `:252-280` |
| 14 | `.sqlx` offline cache is committed and `maki/Dockerfile` builds without a database, so every query change needs `cargo sqlx prepare` (with a CLI matching sqlx **0.9**) or CI breaks. Was stale; regenerated 2026-11-16. | `git ls-files maki/.sqlx`, `maki/Dockerfile:29-32` |
| 15 | Local builds need Postgres at `localhost:5438` (container `muse-maki-db-1` is running) **and** `GIT_CONFIG_GLOBAL=/dev/null CARGO_NET_GIT_FETCH_WITH_CLI=true` for the `darwin-libproc` git dep. | verified this session |
| 16 | **`album.mbid` is an ambiguous entity column.** Tagged files store `MUSICBRAINZ_ALBUMID` (a **release** MBID). Untagged files matched by search store `rg.id` from a `/release-group?query=` search (a **release-group** MBID). Both land in the same column, and `get_cover_art_bytes` always requests `coverartarchive.org/release/{mbid}`, which is a release-only endpoint — so cover art for search-matched albums asks for a release that does not exist. | `db.rs:323-352` (both arms), `musicbrainz.rs:84`, `musicbrainz.rs:143-147` |
| 17 | `delete_stale_songs` deletes every song not stamped since `scan_start`, and parse failures are logged-and-skipped, so a transient read error prunes the existing row. This is a pre-existing bug that phase 2 must not inherit. | `index/mod.rs:48-58`, `db.rs:889-900`, `metadata/mod.rs:105-108` |
| 18 | The watcher acts on `event.paths[0]` as a file: directory-create events produce nothing (extension lookup returns `None`), multi-path events lose all but the first path, and a file still being copied is parsed half-written after a fixed 75 ms sleep. | `index/mod.rs:107-144` |

---

## 1. Where I disagree with the draft

### 1.1 Don't build the MusicBrainz entity graph yet

The draft proposes `Artist → Release Group → Release → Track → Recording → Local asset`.
Maki already has `artist` / `album` / `song` + `audio_assets`, and every API query in
`api/album.rs`, `api/artist.rs`, `api/index.rs`, `api/home.rs`, `api/playlist.rs` joins
`song → album → artist`. Splitting album into release-group + release means rewriting all
of those and migrating existing rows that cannot be filled without more network calls.

The actual problem stated in the draft ("two folders might be the original Japanese edition
and an international edition, don't merge them") needs **one column**:

```sql
ALTER TABLE album ADD COLUMN release_group_mbid varchar;
CREATE INDEX idx_album_release_group ON album(release_group_mbid);
```

- `album` stays what it already is: a *release* (which is what `MUSICBRAINZ_ALBUMID` denotes).
- Editions of one album = `SELECT … FROM album WHERE release_group_mbid = $1`. "Other editions"
  becomes one query; no join changes.
- **Prerequisite:** `album.mbid` must first be made unambiguous (finding 16, phase 5.0). Today
  the column holds a release-group id for search-matched albums, so "editions share a
  release-group" cannot be derived from existing rows without knowing which arm wrote them.

Revisit the entity split only when you need per-recording identity *across* editions
("this track also appears on X"). That is a real feature, but it is not this problem.

### 1.2 Don't implement local transliteration

The draft lists Japanese romanisation, simplified↔traditional conversion, pinyin with and
without tones, and kana conversion as a subsystem to build. Most of that is a trap:

- Japanese romanisation needs morphological segmentation; naive per-character tables produce
  garbage for 君に夢中 (kimi ni muchuu) — the reading of 中 depends on context.
- Han→pinyin needs a dictionary plus polyphone disambiguation.
- 周杰倫 → Jay Chou is **not a transliteration at all**. It is an alias. No amount of local
  conversion produces it.
- Traditional→simplified is a table of thousands of pairs with 1:n ambiguity (發/髮 → 发).

Guessed strings must never produce guessed *identity*, or you merge two different artists and
write the wrong metadata to disk.

What actually solves the problem, with far less code:

1. **MusicBrainz aliases.** `artist/{mbid}?inc=aliases` returns the romanised and localised
   names with a locale and script tag. One request per artist, already rate-limited. This is
   how 宇多田ヒカル and Hikaru Utada become one identity — from data someone curated, not from
   a heuristic we wrote.
2. **MB search already matches across scripts** when you query in the alternate form, so
   trying the tag value *and* the alias value as two queries covers the rest.

Two local wins are exact and worth keeping:

- **NFKC**: full-width `ＢＡＤモード` → `BADモード`, compatibility forms. No dictionary needed.
- **Katakana↔hiragana**: a fixed codepoint offset (U+30A1–U+30F6 ↔ U+3041–U+3096). No dictionary.

Both belong in a search *key*, never in a stored/displayed name. If offline
traditional↔simplified is later required, take a maintained crate; do not hand-roll it.

### 1.3 The real P0 is the two-pass scan, not album matching

Album-first identification is impossible today because the scan is single-pass and does
network I/O per file inside the same loop (finding 4). You cannot "collect all files in the
folder, then compare the whole tracklist against MB" if file #1 has already triggered six
HTTP calls before file #2 is even parsed.

So the order is: make extraction pure and fast → group by folder → *then* match albums.
The draft puts the lookups in phase 5/6; they are phase 0.

### 1.4 No staging table

The draft's flow implies holding extracted file data somewhere before promoting it to an
album. Not needed: pass 1 writes folder-derived albums marked with `album.edition_key`, and
pass 2 reads them back from the DB. The database after pass 1 *is* the staging area. No second
copy of tag data, no sync problem.

---

## 2. Target model

```sql
-- migration A
ALTER TABLE artist ADD COLUMN search_key varchar,
                   ADD COLUMN search_key_version smallint NOT NULL DEFAULT 0;
ALTER TABLE album  ADD COLUMN search_key varchar,
                   ADD COLUMN search_key_version smallint NOT NULL DEFAULT 0,
                   ADD COLUMN release_group_mbid varchar,
                   ADD COLUMN edition_key varchar,        -- folder identity while unmatched
                   ADD COLUMN mbid_source varchar,        -- 'tag-release' | 'search-release-group' | 'match' | 'legacy-unverified'
                   ADD COLUMN metadata_source varchar;    -- 'tag' | 'folder' | 'musicbrainz'
ALTER TABLE song   ADD COLUMN search_key varchar,
                   ADD COLUMN search_key_version smallint NOT NULL DEFAULT 0,
                   ADD COLUMN metadata_source varchar;

-- migration B
CREATE TABLE artist_alias (
  id serial PRIMARY KEY,
  artist integer NOT NULL REFERENCES artist(id) ON DELETE CASCADE,
  alias varchar NOT NULL,        -- as written, never overwritten
  alias_key varchar NOT NULL,    -- search_key(alias), computed in Rust
  locale varchar,        -- MB locale, e.g. 'ja-Latn'
  script varchar,        -- e.g. 'Latn', 'Jpan', 'Hans'
  kind varchar,          -- MB alias type: Artist name / Legal name / ...
  source varchar NOT NULL,   -- 'musicbrainz' | 'tag' | 'user'
  is_primary bool NOT NULL DEFAULT false,
  created_at timestamptz NOT NULL,
  UNIQUE (artist, alias_key)
);
CREATE INDEX idx_artist_alias_key ON artist_alias (alias_key);

-- migration C
CREATE TABLE enrichment_job (
  id serial PRIMARY KEY,
  kind varchar NOT NULL,      -- 'artist_profile' | 'artist_alias' | 'album_art' | 'album_match' | 'track_mbid'
  target integer NOT NULL,
  state varchar NOT NULL DEFAULT 'pending',   -- pending | running | done | failed
  attempts integer NOT NULL DEFAULT 0,
  not_before timestamptz NOT NULL DEFAULT now(),
  last_error text,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz,
  UNIQUE (kind, target)
);
CREATE INDEX idx_enrichment_pending ON enrichment_job (state, not_before, id);

-- migration D
CREATE TABLE album_match_candidate (
  id serial PRIMARY KEY,
  album integer NOT NULL REFERENCES album(id) ON DELETE CASCADE,
  release_mbid varchar NOT NULL,
  release_group_mbid varchar,
  score real NOT NULL,
  payload jsonb NOT NULL,      -- titles, track count, per-track duration deltas
  created_at timestamptz NOT NULL,
  UNIQUE (album, release_mbid)      -- re-matching replaces, never accumulates
);
```

Unchanged and deliberately so: `audio_assets` (hash-keyed), `song.slug = md5(path)`,
`album.slug`, `artist.slug`. Slugs are public URLs; they must not move when metadata changes.

---

## 3. Phases

Each phase is a coherent commit series, independently shippable and independently testable.

### Phase 0 — a repo that compiles, and a scan that does not call the network

**Status: implemented 2026-11-16.** See `CURRENT.md` for the file-by-file record and the
dev-DB incident that happened during its smoke test.

**Why first.** Nothing below can be verified until the tree builds. And the album-matching
work in phase 5 depends on pass 1 finishing fast.

**0.0 Fix the build — DONE 2026-11-16** (leftover breakage from the dependency bump):

- `src/api/song.rs` lastfm call sites: the real behaviour is that **sqlx 0.9 infers projected
  columns of a `LEFT JOIN` inconsistently** — with plain SQL, `song.name`/`artist.name` come
  back `Option<String>` while `album.name` comes back `String`, for the same query shape.
  Verified against the live DB schema (`song.name`, `song.duration`, `album.name`,
  `artist.name` are all `NOT NULL`), so none of it is real nullability. Wrapping all four in
  `COALESCE` makes them uniformly `Option`, which is the only shape callable uniformly; the
  `COALESCE` is an inference workaround, not a NULL guard, and says so in a comment.
  Call sites: `.as_deref().unwrap_or("")`, `.unwrap_or_default() as u32`.
- `.sqlx` regenerated with **sqlx-cli 0.9.0** (58 files modified, 18 added, 3 orphaned entries
  removed; net 76 changed). The globally installed CLI was `0.8.6`, which cannot produce cache
  data sqlx 0.9 accepts; it has been upgraded to **0.9.0**, so plain `cargo sqlx prepare` is now
  correct. Verified with `cargo sqlx prepare --check -- --all-targets` (exit 0).
- Verified: `cargo check --all-targets` passes both online (live DB) and with `SQLX_OFFLINE=true`;
  `cargo test` = 50 passed in both modes. One pre-existing dead-code warning (`AdminUser::payload`).

**Note for every later phase:** because of the inference above, any query with a `LEFT JOIN`
yields `Option` fields. Prefer explicit `Option` handling at call sites over fighting the
inference, and never assume a `NOT NULL` column comes back non-optional just because a join
"cannot" fail.

**0.1 Purge network calls from the write path.**
Remove from the scan call graph, replacing each with a job insert:

| removed | becomes |
|---|---|
| `get_artist_mbid` in `artist_foc` | `enrichment_job('artist_alias', artist_id)` — MB search happens there, and a found MBID is written back |
| Last.fm bio/tags/similar, Deezer artist id, TheAudioDB/Spotify image | `enrichment_job('artist_profile', artist_id)` |
| `get_album_info` in `album_foc` | `enrichment_job('album_match', album_id)` (phase 5 fills the body) |
| CAA + Deezer cover fetch | `enrichment_job('album_art', album_id)` |
| `get_track_mbid` in `song_foc` | `enrichment_job('track_mbid', song_id)` |

**0.2 `src/enrich.rs`** — a durable job queue, specified as such. The lifecycle, because
"copy `analysis.rs`" is wrong: that module derives pending work from missing artifacts and has
no job state machine.

| | |
|---|---|
| claim | `UPDATE enrichment_job SET state='running', attempts=attempts+1, updated_at=now() WHERE id = (SELECT id … WHERE state='pending' AND not_before <= now() ORDER BY id FOR UPDATE SKIP LOCKED LIMIT 1) RETURNING *` — claim and state change in one statement, never a select-then-update |
| success | `state='done'` |
| failure | `attempts < max` → `state='pending'`, `not_before = now() + backoff(attempts)`; else `state='failed'` with `last_error` |
| crash recovery | on worker startup, `state='running'` rows older than `now() - lease` go back to `pending` (they are only ever claimed by this one process, so no fencing token is needed — document that assumption) |
| dedupe | `UNIQUE (kind, target)`; re-scanning a file must not enqueue a duplicate, and a `done` job must not be re-run unless the underlying input changed (delete the job row on metadata change) |
| shutdown | worker observes the same cancellation as the scanner; a cancelled job is left `running` and is recovered by the lease rule above |

Per-provider limiters reuse the existing clients unchanged. No audio specifics copied from
`analysis.rs`; only the "spawn one task, loop, record failures" orchestration.

**0.3 Config:** `enrichment_enabled` (default `true`), `enrichment_concurrency` (default `1`),
`enrichment_max_attempts` (default `5`), `enrichment_lease_secs` (default `300`).

**0.3b Admin visibility** — the plan otherwise promises job state in the admin API without
building it. Explicit step: `GET /api/v1/admin/enrichment` (counts by state and kind, oldest
pending, last errors) and `POST /api/v1/admin/enrichment/{id}/retry`, behind the existing
admin auth extractor (`AdminUser`, `api/middleware/jwt.rs`), with a test that a non-admin token
gets 403 and an admin gets 200. This is the only operator surface for phase 0 and it is
therefore in scope.

**0.4 Scan completeness** (finding 17). `delete_stale_songs` must only run after a walk that
completed without a single unhandled error, and must never delete a row whose path is in a
folder that failed this scan. Concretely: track `files_seen`, `files_failed`, and
`folders_failed` for the pass; if any file in a folder failed, that folder's rows are excluded
from the stale sweep (keep the previous `last_scanned_at`). Test: make one file unreadable,
complete a scan, assert its existing row survives and its folder is untouched by pruning.

**Tests**
- Unit: backoff schedule; state transitions; `UNIQUE (kind, target)` dedupe on re-scan.
- Integration (live DB): `add_song` with a synthetic `AudioMetadata` creates exactly the
  expected job rows and **no** HTTP client is constructed. Assert the latter by making the
  clients' base URLs configurable *for tests* and pointing them at a closed port — if anything
  still dials out during scan, the test fails loudly instead of silently hanging.

**Done when** scanning a 500-file fixture tree performs zero outbound requests (verified by
pointing the process at a dead proxy) and the songs are all present.

---

### Phase 1 — never drop a file, never invent an empty entity (P0)

**Status: implemented 2026-11-16.** Note: `id3::partial_tag_ok` does not recover `NoTag`, so
untagged WAV/AIFF/MP3 still needed `metadata::read_id3_or_empty`; this was found by the live
smoke test, not by reading the code. See `CURRENT.md`.

**1.1 `src/metadata/fallback.rs`**: `TrackHints` derived from a path.

```rust
pub struct TrackHints { pub number: Option<u32>, pub title: String, pub disc: Option<u32> }
pub fn from_path(path: &Path) -> TrackHints;
pub fn parse_folder_name(name: &str) -> FolderHints; // { artist?, album?, year? }
```

Track-number rules (each unit-tested): `01 - x`, `01. x`, `01_x`, `01 x`, `[01] x`, `01x`
(no), full-width `０１ － x`, bare `x` (no number). Title = stem minus the number prefix,
trimmed; if empty, the stem.

Folder-name rules: `Artist - Album`, `Artist - Album (2007)`, `[2007] Album`,
`Album [FLAC]` → strip bracketed format junk. Conservative: when unsure, return no artist and
the whole string as the album title. Never invent a title from a guess.

**1.2 `flac.rs`**: `vorbis_comments()` becomes optional. Missing streaminfo stays an error
(that is real file corruption, not a tagging gap).

**1.3 `metadata/mod.rs`**: one `resolve(raw, hints) -> AudioMetadata` applied after every
format parser, so all four formats agree. Fills `name`, `album`, `album_artist`, `number`,
`disc` from tags first, hints second; sets `metadata_source = 'tag' | 'folder'`. Kills the
`0` vs `25565` inconsistency.

**1.4 `index/db.rs`**: refuse to create an `artist` or `album` whose name is empty. Log the
path and skip, rather than writing a row named `""` (finding 7). `album_artist` falls back to
the first artist, then to a single `Unknown Artist` row, never `""`.

**Tests**: fixture files (see §5) plus table-driven `TrackHints` tests covering the folder
tree in the draft, including `宇多田ヒカル - BADモード/01 - BADモード.flac` and
`周杰倫 - 七里香/02 - 七里香.flac`.

**Done when** an untagged FLAC in `宇多田ヒカル - BADモード/01 - BADモード.flac` imports as
album `BADモード`, artist `宇多田ヒカル`, track 1, title `BADモード`.

---

### Phase 2 — folder = album candidate (P0)

**Status: implemented 2026-11-16.** The migration backfills `edition_key` from each song's
*immediate* parent, so disc-subfolder albums from the old schema can be flagged ambiguous;
a rescan re-derives them correctly. No `album_match` job is queued yet (phase 5).

**2.1 `src/index/folder.rs`**: `album_key(path) -> AlbumKey { folder, disc, name_override }`.

- `Disc 1` / `CD2` / `Disk 03` / `Album (Disc 2)` → album folder is the parent, `disc` comes
  from the folder name.
- Otherwise the album folder is the immediate parent.
- Files directly in the scan root are not an album: key them per-file, album title from tags
  else `Unknown Album`.
- Special-cased names are a small enum with tests, not a regex soup.

**2.2 `index/mod.rs`**: two-pass, streaming.

`WalkDir::sort(true)` yields a folder's files contiguously, so pass 1 groups one **resolved
`album_key`** at a time (not one raw parent directory — a disc folder resolves to its parent,
so the group boundary is the key, not the path):

```text
for each entry (sorted):
    if album_key(entry) != current key: finish current candidate -> db::save_candidate
    else: extract metadata, push to current candidate
```

A folder is buffered **fully** before anything is written, and `save_candidate` is one
transaction: album row + all song rows + junction rows. A candidate is never partially visible
to pass 2. Two consequences to implement explicitly:

- a file whose parse fails does not silently shrink the album; it is counted, its folder is
  marked incomplete, and pass 2 **skips matching** for that album until a scan sees the folder
  cleanly (matching an album against a tracklist you know is missing a track is how you write
  wrong MBIDs);
- a cap (`max_tracks_per_folder`, default 500) aborts grouping for absurd folders with a warning
  rather than buffering unboundedly.

Pass 2 (`db::match_pending_albums`) runs after the walk and enqueues `album_match` jobs for
albums with `edition_key IS NOT NULL` and no outstanding incompleteness, so the enrichment
worker picks them up (phase 5).

**2.3 `db::save_candidate`**: resolve album artist once, upsert the album once
(`mbid → edition_key → (name, album_artist)`), then insert/update all songs in **one
transaction**. Today it is N albums-worth of queries plus per-song junction churn; this is
where most of the scan time goes once the network is gone.

**2.3b Existing rows.** Albums indexed before this change have `edition_key IS NULL` and, worse,
editions that previously collapsed into one album row now share a row with songs from two
folders. Policy, stated once and tested:

1. migration backfills `edition_key` from each song's parent folder: for every album, group its
   songs by resolved `album_key`; exactly one group → set it; more than one → **leave NULL** and
   insert an `album` row of `metadata_source='unreconciled'` marker in the review table, because
   guessing which folder owns the album rewrites someone's library;
2. a full rescan then re-derives each folder independently and attaches via `edition_key`,
   producing correct per-edition albums for the folders it sees;
3. `delete_stale_songs` must not delete rows that were re-keyed but not yet re-stamped in the
   same pass (see 0.4).

Upgrade test: build a pre-migration DB with a merged two-edition album, run the migration and a
rescan, assert the two editions are now separate albums with the right songs, and that no song
row was lost.

**2.4 Watcher**: the current handler treats `event.paths[0]` as a file (finding 18). Required
behaviour:

- iterate **all** `event.paths`, not just the first;
- a path that is a directory → scan that directory as one candidate (this is what happens when
  someone `mv`s a finished album folder in, and today produces nothing);
- `Create` on a file → wait until size **and** mtime are stable across two polls (`with_poll_interval`
  is already 30 s; the fixed 75 ms sleep is not a completeness guarantee) before parsing, with a
  bail-out after N polls so a permanently-growing file cannot wedge the watcher;
- `Remove` keeps the existing path-based delete; a removed directory removes every song under it.

A new file in a known folder must attach to the existing album via `edition_key`, not create a
second one.

**2.5 `edition_key`** = the album folder path, absolute. It is a *derived* identity, so a moved
library re-derives it on the next scan. Document that: `edition_key` is not a stable public id
(unlike `slug`).

**Tests**
- Grouping: two editions of one album stay two albums; a various-artists album stays one; a
  disc-per-folder album collapses to one album with `disc` set; loose root files each get
  their own album; an album folder with one file works; a folder with one unreadable file is
  marked incomplete and is **not** matched.
- DB: `save_candidate` on a synthetic 12-track candidate creates 1 album + 12 songs + N
  `song_artist` rows, and re-running it creates no duplicates; a failed transaction leaves no
  album row.
- Watcher: a `Create` event carrying a directory path scans the whole folder; an event with
  two paths processes both; a file still growing is not parsed until stable.

---

### Phase 3 — script-aware search keys (P0)

**Status: implemented 2026-11-16.** Search is `search_key ILIKE … ESCAPE '\'` with the query
normalized in Rust. Existing rows get an approximate SQL backfill (version 0) and
`maki reindex-search` makes them exact. No HTTP-level regression test; unit tests + a direct SQL
check were used.

**3.1 `src/helpers/normalize.rs`**

```rust
pub fn search_key(s: &str) -> String
// NFKC -> drop U+0300..U+036F (Latin combining marks) -> lowercase
//       -> fold katakana to hiragana -> collapse whitespace -> trim
```

Order matters and is documented: NFKC composes first so that か + ゛ becomes が and is *not*
then stripped as a combining mark; dropping only the Latin diacritic range keeps kana and Han
intact. This replaces `unaccent()` in queries and fixes full-width matching that `unaccent`
cannot do.

**3.2 Migration**: `search_key` on `artist`, `album`, `song`, plus `search_key_version smallint`.
Choosing one strategy rather than leaving it open: **do both**, because the cheap half is not
correct and the correct half needs a trigger.

1. Migration backfills `search_key = lower(normalize(x, NFKC))`, `search_key_version = 0`
   (approximate: no kana fold, no mark stripping).
2. A one-shot `maki --reindex-search` computes the real key in Rust and sets
   `search_key_version = 1`. It is ~30 lines, idempotent, and `WHERE search_key_version < 1`
   makes it resumable.
3. Search treats `search_key_version = 0` rows as searchable but logs a count at startup, so an
   operator can see that the reindex has not run. Silent staleness is the failure mode to avoid.

**3.3 Write paths.** Enumerated from the code, not guessed. Every site that writes a searchable
name, all in `index/db.rs`: `artist` insert (`:231`), `album` insert (`:485`), `song` insert
(`:731`) and update (`:652`). There is no admin rename endpoint today (`api/admin.rs` has none),
so nothing else to change; if one is added it must set the key. Guard: a test that inserts
through each path and asserts `search_key IS NOT NULL`, so a future write path that forgets it
fails the suite rather than silently becoming unsearchable.

**3.4 `api/index.rs` search** matches on `search_key ILIKE '%' || $1 || '%'` with the query
normalised server-side. Acceptance criteria, because this is a live contract for `muse-web` and
the iOS app: the response type (`SearchSong`), field set, `sortby`/`dir` handling, and the
`picture` URL rewriting must all be unchanged; only the `WHERE` changes. Add an API regression
test that a query finding a row before the change finds the same row after it. Fold the artist
alias into the same query once phase 4 lands (`OR EXISTS (SELECT 1 FROM artist_alias a WHERE …)`)
in a *separate* commit. Defer `pg_trgm`/tantivy until a query is measurably slow.

**Tests**: `search_key("ＢＡＤモード") == search_key("BADモード")`,
`search_key("バド") == search_key("ばど")`, `search_key("Café") == "cafe"`, Han untouched,
`蓄音` ≠ `贮音` (documented limitation, resolved by aliases in phase 4).

---

### Phase 4 — aliases and identity resolution (P1)

**Status: implemented 2026-11-16.** See `CURRENT.md`.

**4.1** Migration `artist_alias` (§2), with the key made explicit: `alias_key` is
`search_key(alias)` (phase 3.1) computed in Rust and stored in its own column, with a
non-unique index. It is **not** `lower(alias)`, which would not match 宇多田ヒカル typed as
half-width or with a different kana form. `UNIQUE (artist, alias_key)` so one artist cannot
accumulate duplicates through re-import; two *different* artists may legitimately hold the same
alias_key, which the resolver must treat as ambiguous rather than resolve.

**4.2** `artist_foc` resolution order, explicitly documented in code:

```text
mbid  ->  exact name  ->  alias_key  ->  create new
```

The alias step collects **all** matching artists. One match → use it. More than one with
different MBIDs → ambiguous: log both, attach to none of them automatically, and record a review
row. Fuzzy similarity is a suggestion channel only; it never writes an identity.

**4.3** Alias import in the enrichment worker: `artist/{mbid}?inc=aliases`, storing
`locale`/`script`/`kind`/`is_primary` and the computed `alias_key`. Re-runs are idempotent.

**4.4** `api/artist.rs` returns canonical `name` plus `aliases` (opt-in query param).
Per-user display preference is **out of scope here** (needs user settings + frontend), but the
data model supports it: display picks an alias by locale at render time, and the stored `name`
never changes. Renaming an artist is an admin action that keeps `slug` fixed.

**Tests**: 宇多田ヒカル / Hikaru Utada / Utada resolve to one artist when an alias row exists;
`mbid` always wins over a name match; an alias cannot silently reassign an artist that already
has a different `mbid`; two artists sharing one alias_key resolve to neither and produce a
review row; aliases differing only by case, width, or kana form collide on the same key.

---

### Phase 5 — album-level MusicBrainz matching with confidence (P1)

**Status: implemented 2026-11-16.** No live MusicBrainz call was made; scoring is unit-tested.

**5.0 Fix the entity ambiguity first** (finding 16). Everything below assumes `album.mbid` means
one thing. Today it means two:

| path | value stored in `album.mbid` |
|---|---|
| file tags carry `MUSICBRAINZ_ALBUMID` | a **release** MBID (`db.rs:331-347`) |
| no tag, matched by `get_album_info` search | a **release-group** MBID (`db.rs:348-360` → `musicbrainz.rs:143-147`) |

And `get_cover_art_bytes` always calls `coverartarchive.org/release/{mbid}` (`musicbrainz.rs:84`),
so the search path asks a release endpoint about a release-group id. Fix:

- `album.mbid` = release MBID, `album.release_group_mbid` = release-group MBID, never mixed;
- on the search path store into `release_group_mbid`, leave `mbid` NULL until a release is
  actually identified (that is what phase 5 is for);
- cover art tries `release/{mbid}`, then `release-group/{release_group_mbid}`;
- migration: existing `album.mbid` values cannot be classified offline, so they are marked
  `album.mbid_source = 'legacy-unverified'` and re-resolved by the album-matching job;
- delete the misleading comment at `db.rs:325` ("do a direct release-group lookup") — it is a
  *release* lookup that extracts the release-group, and that comment is what made a plan review
  conclude the opposite.

I did not verify the CAA 404 behaviour live: musicbrainz.org returned 503 from this machine
during the session, so "a release-group id is not a valid release id" is inferred from the two
distinct CAA endpoints rather than observed. First task in 5.0 is to observe it.

**5.1 `metadata/musicbrainz.rs`**
- `search_releases(title, artist, limit)` → candidates **with `score`**. The current structs
  drop MB's `score` field entirely and pick `.first()`, which is the actual reason matches are
  untrustworthy.
- `get_release(mbid)` with `inc=recordings+artist-credits` → media → tracks
  (`position`, `title`, `length`).

**5.2 `src/match_album.rs`** — pure scoring over `(folder tracks) × (release candidate)`:

```text
reject unless track_count matches (or |Δ| <= 1, flagged)
score = w1 * duration_fit(median |Δms| per position)   // dominant signal
      + w2 * title_similarity(normalised)              // helps break ties
      + w3 * artist_similarity(name or alias)
confidence floor -> candidate row only
confidence >= threshold -> apply
```

Weights and `album_match_min_score` live in config. Duration fit uses the *median*, not the
mean, so one 20-minute bonus track does not poison an otherwise exact match.

**5.3** Everything between the floor and the threshold goes to `album_match_candidate` (§2) with
the per-track deltas in `payload`. Lifecycle, so rescans do not accumulate garbage:

- `UNIQUE (album, release_mbid)` + upsert; a re-match replaces the row's score and payload
  rather than adding one;
- when an album is matched or its identity changes (`edition_key` moves, album re-derived),
  delete its candidate rows;
- rows are meaningless without a consumer, so ship the minimal one: `GET /api/v1/admin/albums/{id}/matches`
  and `POST /api/v1/admin/albums/{id}/match` (apply a chosen release) behind the admin extractor,
  same pattern as 0.3b. If that is not built, do not write the table — an unread table is a
  false promise.

**5.4** Application is **non-destructive by default**: write `album.mbid`,
`album.release_group_mbid`, `song.mbid` where a position+title match is unambiguous. Do *not*
rewrite titles or the album name unless `--apply-mb-titles` is passed. Keep `edition_key` after
a match so a re-scan can re-verify the same folder against the same release.

**Tests**: captured MB JSON in `testdata/mb/`; synthetic folder tracklists with jittered
durations; assert the correct release wins among decoys, that two editions of one
release-group stay two albums, that a folder whose durations match nothing gets candidate rows
and no `release_group_mbid`, that re-matching an album twice leaves one candidate row per
release, and that cover art is requested from the endpoint matching the entity type stored.

---

### Phase 6 — mostly implemented (fingerprinting still deferred)

**Status 2026-11-16:** review UI, display preferences and album aliases are implemented;
AcoustID/Chromaprint fingerprinting remains out of scope.

- Review UI for `album_match_candidate` / unresolved artists.
- AcoustID / Chromaprint fingerprinting. Only worth it for tracks that survive phase 5
  unmatched. Note `audio_assets` is already content-addressed, so a fingerprint result would
  hang off `audio_hash` and survive file moves for free.
- Per-user display language / romanised titles in the UI.
- `album_alias` (alternate album titles). Low value until search covers albums explicitly.

---

## 4. Explicitly cut

| Not building | Why |
|---|---|
| `release_group` / `release` / `recording` tables | Duplicates MusicBrainz in your DB and rewrites every API query. One column solves the stated problem. (§1.1) |
| Local kana/pinyin/traditional-Chinese transliteration | Guesses, not identity. Aliases from MB are curated data. (§1.2) |
| `raw_file` staging table | Pass 1's output in `album`/`song` already is the staging area. (§1.4) |
| `search_key` via DB trigger | Normalisation logic lives in Rust; two implementations would drift. |
| `pg_trgm` / tantivy | Not slow yet at this library size. Revisit with measurements. |
| Fingerprinting before album matching | Album-level matching gets most releases for free, as the draft itself notes. |

---

## 5. Test infrastructure

- `maki/testdata/` (tracked, small): one ~1 s silent FLAC, MP3, WAV, AIFF; the same four
  tagged with Japanese/Chinese/full-width metadata; one FLAC with **no** VORBIS_COMMENT at all;
  captured `mb/*.json` responses. Generate the audio once and commit (they are a few KB each).
- Unit tests for every pure function: `TrackHints`, `parse_folder_name`, `album_key`,
  `search_key`, scoring, backoff. These need no DB and should be the bulk of the coverage.
- Integration tests needing Postgres: follow the existing precedent (throwaway container +
  `migration run`), behind a `just test-db` recipe that starts `postgres:17` on 5438, applies
  `maki/migrations`, and runs `cargo test`. Requires the two env workarounds from finding 15.
- After **every** migration: `cargo sqlx prepare`, commit `.sqlx`. Fix the current staleness in
  phase 0.0 so this stays honest.

---

## 6. Risks

1. **Two-pass scan changes watcher semantics.** A file added to an existing folder must join
   the existing album, not create one. Mitigation: resolve by `edition_key` first; explicit test.
2. **`edition_key` is a path.** Moving the library re-derives album identity and can create
   duplicates if a partial scan runs mid-move. Mitigation: keep the existing
   `delete_stale_songs` sweep; do not treat `edition_key` as a public id.
3. **Removing inline lookups changes perceived behaviour.** New artists appear with no bio or
   art for the first minutes after a scan. That is the correct tradeoff for a library server;
   surface job state in the admin API so it is visible rather than mysterious.
4. **`unaccent` removal.** Check for other consumers (grep shows only `api/index.rs`) before
   dropping usage; leave the extension installed.
5. **Existing rows keep approximate `search_key`s** until reindexed. Startup now logs the count
   (`run maki reindex-search`), and reindex is batched at 500 rows/page.
6. **Enrichment on the ultra.cc box**: 1 req/s across the whole library is slow by design.
   `enrichment_enabled` must be configurable and default to on only after phase 0 shows the
   scan itself is fast.
7. **~~`delete_stale_songs` assumes `MOUNT` is the entire library.~~ RESOLVED 2026-11-16.**
   Pruning now requires a non-empty scan, a persisted `index_state.library_root` match, and a
   prune set below `max_prune_fraction` of the library. Skipped files (500-track cap) count as
   failures. The review also found the original root check was a tautology in the serving binary;
   the persisted guard replaces it. See `CURRENT.md` for the review round.

---

## 7. Open questions

1. **`album` semantics**: resolved — keep it as a *release*, with `release_group_mbid` for
   editions. (Phase 5.0 must first disambiguate the values already in `album.mbid`.)
2. **`edition_key`**: folder path (simple, breaks on move) or
   `(normalised album, album artist, year, track_count)` (survives moves, collides on two
   genuine editions with identical counts)? Still open, and phase 2.3b (existing-row
   reconciliation) changes shape depending on the answer.
3. **Enrichment default on ultra.cc**: on by default, or a flag you flip after the first scan?
4. **Admin surface**: 0.3b and 5.3 now require six small admin endpoints. Confirm that is
   wanted, or drop the corresponding promises instead of building them.
5. ~~The 4 build errors in `api/song.rs`~~ — resolved 2026-11-16, see Phase 0.0.

---

## 8. What the plan review changed (2026-11-16)

Reviewed by `plan-reviewer`. Accepted: most of it. Rejected: its P0.

**Rejected — the review's P0 was wrong.** It claimed `album_foc` "passes the tag MBID to
`get_release_group_info`; it does not perform a release lookup there", therefore the
"album = release" premise was unsound. `get_release_group_info` is named for what it
*returns*, and it GETs `/release/{mbid}?inc=release-groups` (`musicbrainz.rs:189-191`) — a
release lookup. The function name plus a misleading in-code comment (`db.rs:325`, "do a direct
release-group lookup") is what misled the review. The premise stands.

**Promoted to P0 — what that wrong finding was circling.** `album.mbid` really is ambiguous,
just not for the stated reason: the tag path stores a *release* id, the search path stores a
*release-group* id, and cover art always queries the release endpoint. New findings 16 and 17,
new phase 5.0, `album.mbid_source`, and the bogus comment gets deleted.

**Accepted and folded in:**

| Review finding | Where it landed |
|---|---|
| No migration policy for pre-existing albums / previously-merged editions | §3 Phase 2.3b |
| Stale-prune is unsafe under a failed or partial scan (pre-existing bug) | §0 finding 17, §3 Phase 0.4 |
| "Copy the shape of `analysis.rs`" is wrong — it is not a queue | §0 finding 13, §3 Phase 0.2 (full job lifecycle table) |
| Candidate completeness: a folder with a failed file must not be matched | §3 Phase 2.2 |
| Watcher drops directory events, extra paths, and half-copied files | §0 finding 18, §3 Phase 2.4 |
| `artist_alias` had no defined key or collision contract | §2 schema (`alias_key`), §3 Phase 4.1/4.2 |
| `album_match_candidate` has no lifecycle or consumer | §3 Phase 5.3 |
| Admin visibility promised but never scheduled | §3 Phase 0.3b |
| Group by resolved `album_key`, not raw parent; bound memory | §3 Phase 2.2 |
| "Approximate backfill *or* reindex" leaves search quietly broken | §3 Phase 3.2 (both, with a version column) |
| Name-write paths guessed, not enumerated | §3 Phase 3.3 (actual sites, plus a guard test) |
| Search API contract not stated as acceptance criteria | §3 Phase 3.4 |

**Small deviations made while implementing phase 0:** the MBID job is called `artist_mbid`
(alias import needs `search_key` and the alias table from phases 3/4, so it is not built yet), and
no `album_match` job is queued in phase 0 — phase 5 owns it. Queuing a job with no consumer
would be dead work.
