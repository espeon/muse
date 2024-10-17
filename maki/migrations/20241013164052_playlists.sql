-- Add migration script here

CREATE TABLE playlist (
  id serial PRIMARY KEY,
  user_id integer NOT NULL,
  name varchar(255) NOT NULL,
  description text,
  art_path text,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),

  FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

-- playlist items table - a doubly linked list of song ids
CREATE TABLE playlist_item (
  id serial PRIMARY KEY,
  playlist_id integer NOT NULL,
  song_id integer NOT NULL,
  prev_song_id integer,
  next_song_id integer,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz NOT NULL DEFAULT now(),

  FOREIGN KEY (playlist_id) REFERENCES playlist (id) ON DELETE CASCADE,
  FOREIGN KEY (song_id) REFERENCES song (id) ON DELETE CASCADE,
  -- we will need to handle prev/next song ids that disappear manually!
  FOREIGN KEY (prev_song_id) REFERENCES playlist_item (id),
  FOREIGN KEY (next_song_id) REFERENCES playlist_item (id)
);

-- add indices
CREATE INDEX idx_playlist_user_id ON playlist (user_id);
CREATE INDEX idx_playlist_name ON playlist (name);

CREATE INDEX idx_playlist_item_playlist_id ON playlist_item (playlist_id);
