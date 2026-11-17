-- Search keys are computed in Rust (NFKC + kana fold + diacritic stripping) and stored, so a
-- search can match full-width and kana variants that `unaccent()` cannot see.
ALTER TABLE artist ADD COLUMN search_key varchar,
                   ADD COLUMN search_key_version smallint NOT NULL DEFAULT 0;
ALTER TABLE album  ADD COLUMN search_key varchar,
                   ADD COLUMN search_key_version smallint NOT NULL DEFAULT 0;
ALTER TABLE song   ADD COLUMN search_key varchar,
                   ADD COLUMN search_key_version smallint NOT NULL DEFAULT 0;

-- Cheap approximation for existing rows. Version 0 means "not yet computed in Rust";
-- `maki reindex-search` makes them exact and is resumable via the version column.
UPDATE artist SET search_key = lower(normalize(name, NFKC)) WHERE name IS NOT NULL;
UPDATE album  SET search_key = lower(normalize(name, NFKC)) WHERE name IS NOT NULL;
UPDATE song   SET search_key = lower(normalize(name, NFKC)) WHERE name IS NOT NULL;

CREATE INDEX idx_artist_search_key ON artist (search_key varchar_pattern_ops);
CREATE INDEX idx_album_search_key ON album (search_key varchar_pattern_ops);
CREATE INDEX idx_song_search_key ON song (search_key varchar_pattern_ops);
