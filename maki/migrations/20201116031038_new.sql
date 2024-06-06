-- init extensions
CREATE EXTENSION IF NOT EXISTS "unaccent";

-- Add migration script here
CREATE TABLE artist (
  id serial primary key,
  name varchar not null,
  bio varchar,
  picture varchar,
  created_at timestamp with time zone not null,
  updated_at timestamp with time zone,
  tags varchar
);

-- Album tables

CREATE TABLE album (
  id serial primary key,
  name varchar not null,
  artist integer not null,
  isrc varchar,
  year integer,
  release_date date,
  copyright varchar,
  created_at timestamp with time zone not null,
  updated_at timestamp with time zone,
  FOREIGN KEY (artist) REFERENCES artist (id)
  on delete cascade
);

CREATE TABLE album_art (
  id serial primary key,
  album integer not null,
  path varchar not null,
  type varchar,
  created_at timestamp with time zone not null,
  updated_at timestamp with time zone,
  FOREIGN KEY (album) REFERENCES album (id)
  on delete cascade
);


CREATE TABLE album_artist (
  id serial primary key,
  album integer not null,
  artist integer not null,
  created_at timestamp with time zone not null,
  updated_at timestamp with time zone,
  FOREIGN KEY (album) REFERENCES album (id),
  FOREIGN KEY (artist) REFERENCES artist (id)
);


-- Genre tables
CREATE TABLE genre (
  id serial primary key,
  name varchar,
  created_at timestamp with time zone not null,
  updated_at timestamp with time zone
);

CREATE TABLE album_genre (
  id serial primary key,
  album integer not null,
  genre integer not null,
  created_at timestamp with time zone not null,
  updated_at timestamp with time zone,
  FOREIGN KEY (album) REFERENCES album (id),
  FOREIGN KEY (genre) REFERENCES genre (id)
);


-- Song tables
CREATE TABLE song (
  id serial primary key,
  number integer,
  disc integer,
  name varchar not null,
  path varchar not null,
  album integer not null,
  -- artist integer not null,
  album_artist integer not null,
  liked bool,
  duration integer not null,
  last_play timestamp with time zone,
  plays integer,
  lossless bool,
  sample_rate integer,
  bits_per_sample integer,
  num_channels integer,
  created_at timestamp with time zone not null,
  updated_at timestamp with time zone,
  FOREIGN KEY (album) REFERENCES album (id),
  FOREIGN KEY (album_artist) REFERENCES artist (id)
);

CREATE TABLE song_artist (
  id serial primary key,
  song integer not null,
  artist integer not null,
  created_at timestamp with time zone not null,
  updated_at timestamp with time zone,
  FOREIGN KEY (song) REFERENCES song (id),
  FOREIGN KEY (artist) REFERENCES artist (id)
);

CREATE TABLE song_genre (
  id serial primary key,
  song integer not null,
  genre integer not null,
  created_at timestamp with time zone not null,
  updated_at timestamp with time zone,
  FOREIGN KEY (song) REFERENCES song (id),
  FOREIGN KEY (genre) REFERENCES genre (id)
);

CREATE TABLE server (
  id serial primary key,
  scan_start timestamp with time zone,
  scan_end timestamp with time zone,
  last_scan timestamp with time zone,
  seconds integer,
  albums integer,
  artists integer,
  size integer
);

-- User tables

CREATE TABLE account (
  id serial primary key,
  username varchar not null,
  created_at timestamp with time zone not null,
  updated_at timestamp with time zone,
  UNIQUE (username)
);