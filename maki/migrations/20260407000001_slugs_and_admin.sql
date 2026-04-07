-- Stable public slugs for song, album, artist
-- Song slug: md5 of path (already the stable unique key)
ALTER TABLE song ADD COLUMN slug varchar;
UPDATE song SET slug = md5(path);
ALTER TABLE song ALTER COLUMN slug SET NOT NULL;
CREATE UNIQUE INDEX idx_song_slug ON song (slug);

-- Artist slug: md5 of lowercased name
ALTER TABLE artist ADD COLUMN slug varchar;
UPDATE artist SET slug = md5(lower(name));
ALTER TABLE artist ALTER COLUMN slug SET NOT NULL;
CREATE UNIQUE INDEX idx_artist_slug ON artist (slug);

-- Album slug: md5 of lowercased "name|artist_name"
ALTER TABLE album ADD COLUMN slug varchar;
UPDATE album SET slug = md5(lower(album.name) || '|' || lower(artist.name))
FROM artist WHERE album.artist = artist.id;
ALTER TABLE album ALTER COLUMN slug SET NOT NULL;
CREATE UNIQUE INDEX idx_album_slug ON album (slug);

-- Admin flag on users
ALTER TABLE users ADD COLUMN is_admin bool NOT NULL DEFAULT false;
