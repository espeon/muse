-- Add MBID columns to artist and album tables
ALTER TABLE artist ADD COLUMN mbid varchar;
ALTER TABLE album ADD COLUMN mbid varchar;

-- Create indexes for faster lookups
CREATE INDEX idx_artist_mbid ON artist(mbid);
CREATE INDEX idx_album_mbid ON album(mbid);
