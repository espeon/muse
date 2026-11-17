-- Phase 4: curated multilingual artist aliases. `alias_key` is search_key(alias), computed in
-- Rust, so width/kana/romanisation variants collapse onto one comparison key.
CREATE TABLE artist_alias (
  id serial PRIMARY KEY,
  artist integer NOT NULL REFERENCES artist(id) ON DELETE CASCADE,
  alias varchar NOT NULL,
  alias_key varchar NOT NULL,
  locale varchar,
  script varchar,
  kind varchar,
  source varchar NOT NULL,
  is_primary bool NOT NULL DEFAULT false,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (artist, alias_key)
);
CREATE INDEX idx_artist_alias_key ON artist_alias (alias_key);

-- Phase 5.0: album.mbid currently holds a release MBID for tagged albums and a release-group
-- MBID for search-matched albums. Separate them; existing values cannot be classified offline.
ALTER TABLE album ADD COLUMN release_group_mbid varchar,
                   ADD COLUMN mbid_source varchar;
UPDATE album SET mbid_source = 'legacy-unverified' WHERE mbid IS NOT NULL;

-- Phase 5.3: candidates for ambiguous album matches, idempotent per release.
CREATE TABLE album_match_candidate (
  id serial PRIMARY KEY,
  album integer NOT NULL REFERENCES album(id) ON DELETE CASCADE,
  release_mbid varchar NOT NULL,
  release_group_mbid varchar,
  score real NOT NULL,
  payload jsonb NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (album, release_mbid)
);
