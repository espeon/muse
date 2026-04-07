-- Stale detection: stamp every song row each time it's seen during a scan
ALTER TABLE song ADD COLUMN last_scanned_at timestamptz;
CREATE INDEX idx_song_last_scanned_at ON song (last_scanned_at);

-- Fix missing ON DELETE CASCADE on song_artist and song_genre so that
-- deleting a song doesn't violate FK constraints.
-- (The initial migration defined these FKs without ON DELETE CASCADE.)
ALTER TABLE song_artist DROP CONSTRAINT song_artist_song_fkey;
ALTER TABLE song_artist ADD CONSTRAINT song_artist_song_fkey
  FOREIGN KEY (song) REFERENCES song(id) ON DELETE CASCADE;

ALTER TABLE song_genre DROP CONSTRAINT song_genre_song_fkey;
ALTER TABLE song_genre ADD CONSTRAINT song_genre_song_fkey
  FOREIGN KEY (song) REFERENCES song(id) ON DELETE CASCADE;
