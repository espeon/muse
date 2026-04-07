ALTER TABLE artist ADD COLUMN deezer_id bigint;

CREATE INDEX idx_artist_deezer_id ON artist(deezer_id);

CREATE TABLE artist_similar (
  id serial primary key,
  artist integer not null references artist(id) on delete cascade,
  name varchar not null,
  source varchar not null,
  created_at timestamp with time zone not null
);

CREATE INDEX idx_artist_similar_artist ON artist_similar(artist);
