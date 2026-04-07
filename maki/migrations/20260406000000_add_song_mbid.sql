ALTER TABLE song ADD COLUMN mbid varchar;

CREATE INDEX idx_song_mbid ON song(mbid);
