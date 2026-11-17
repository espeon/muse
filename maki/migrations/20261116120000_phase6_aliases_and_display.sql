-- Alternate album titles from MusicBrainz, keyed by the same Rust-computed search key as
-- artist aliases.
CREATE TABLE album_alias (
  id serial PRIMARY KEY,
  album integer NOT NULL REFERENCES album(id) ON DELETE CASCADE,
  alias varchar NOT NULL,
  alias_key varchar NOT NULL,
  locale varchar,
  script varchar,
  kind varchar,
  source varchar NOT NULL,
  is_primary bool NOT NULL DEFAULT false,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (album, alias_key)
);
CREATE INDEX idx_album_alias_key ON album_alias (alias_key);

-- Per-user display preference: original, romanised, or english (falls back to canonical name).
ALTER TABLE users ADD COLUMN name_display varchar NOT NULL DEFAULT 'original';
