-- Create the generalized 'favorites' table

CREATE TABLE favorites (
  id serial PRIMARY KEY,
  user_id integer NOT NULL,
  favoritable_id integer NOT NULL,
  favoritable_type varchar(50) NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),

  -- Ensure user_id references the users table
  FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,

  -- Ensure that the combination of user_id, favoritable_id, and favoritable_type is unique
  UNIQUE (user_id, favoritable_id, favoritable_type)
);

-- Add a 'plays' table to log user plays of songs
CREATE TABLE plays (
  id serial PRIMARY KEY,
  user_id integer NOT NULL,
  song_id integer NOT NULL,
  played_at timestamptz NOT NULL DEFAULT now(),

  -- Ensure user_id references the users table
  FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE,

  -- Ensure song_id references the song table
  FOREIGN KEY (song_id) REFERENCES song (id) ON DELETE CASCADE
);


-- indices for all the tables that need them
-- Add indices for the 'artist' table
CREATE INDEX idx_artist_name ON artist (name);

-- Add indices for the 'album' table
CREATE INDEX idx_album_artist ON album (artist);
CREATE INDEX idx_album_name ON album (name);

-- Add indices for the 'album_art' table
CREATE INDEX idx_album_art_album ON album_art (album);

-- Add indices for the 'album_artist' table
CREATE INDEX idx_album_artist_album ON album_artist (album);
CREATE INDEX idx_album_artist_artist ON album_artist (artist);

-- Add indices for the 'genre' table
CREATE INDEX idx_genre_name ON genre (name);

-- Add indices for the 'album_genre' table
CREATE INDEX idx_album_genre_album ON album_genre (album);
CREATE INDEX idx_album_genre_genre ON album_genre (genre);

-- Add indices for the 'song' table
CREATE INDEX idx_song_album ON song (album);
CREATE INDEX idx_song_album_artist ON song (album_artist);
CREATE INDEX idx_song_name ON song (name);
CREATE INDEX idx_song_path ON song (path);

-- Add indices for the 'song_artist' table
CREATE INDEX idx_song_artist_song ON song_artist (song);
CREATE INDEX idx_song_artist_artist ON song_artist (artist);

-- Add indices for the 'song_genre' table
CREATE INDEX idx_song_genre_song ON song_genre (song);
CREATE INDEX idx_song_genre_genre ON song_genre (genre);

-- Add indices for the 'server' table
CREATE INDEX idx_server_last_scan ON server (last_scan);

-- Add indices for the 'verification_token' table (next-auth)
CREATE INDEX idx_verification_token_expires ON verification_token (expires);

-- Add indices for the 'accounts' table (next-auth)
CREATE INDEX idx_accounts_userId ON accounts ("userId");
CREATE INDEX idx_accounts_provider ON accounts (provider, "providerAccountId");

-- Add indices for the 'sessions' table (next-auth)
CREATE INDEX idx_sessions_userId ON sessions ("userId");
CREATE INDEX idx_sessions_sessionToken ON sessions ("sessionToken");

-- Add indices for the 'users' table (next-auth)
CREATE INDEX idx_users_email ON users (email);

-- Add indices for the 'user_lastfm' table
CREATE INDEX idx_user_lastfm_userId ON user_lastfm ("userId");
CREATE INDEX idx_user_lastfm_lastfm_username ON user_lastfm (lastfm_username);

CREATE INDEX idx_favorites_user_favoritable_type
  ON favorites (user_id, favoritable_type);

CREATE INDEX idx_plays_user_id ON plays (user_id);
CREATE INDEX idx_plays_song_id ON plays (song_id);
CREATE INDEX idx_plays_played_at ON plays (played_at);
