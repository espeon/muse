-- Content-addressed audio assets outlive individual song rows. A moved or
-- re-imported byte-identical file therefore reuses its analysis results.
CREATE TABLE audio_assets (
  hash bytea PRIMARY KEY CHECK (octet_length(hash) = 32),
  hash_algorithm varchar NOT NULL CHECK (hash_algorithm = 'blake3-256'),
  byte_size bigint NOT NULL CHECK (byte_size >= 0),
  created_at timestamptz NOT NULL DEFAULT now()
);

ALTER TABLE song ADD COLUMN audio_hash bytea;
ALTER TABLE song ADD COLUMN audio_hash_size bigint;
ALTER TABLE song ADD COLUMN audio_hash_mtime_ns bigint;
ALTER TABLE song ADD CONSTRAINT song_audio_hash_fkey
  FOREIGN KEY (audio_hash) REFERENCES audio_assets(hash) ON DELETE SET NULL;
CREATE INDEX idx_song_audio_hash ON song(audio_hash);

CREATE TABLE audio_similarity_features (
  audio_hash bytea NOT NULL REFERENCES audio_assets(hash) ON DELETE CASCADE,
  version integer NOT NULL,
  vector bytea NOT NULL,
  analyzed_at timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (audio_hash, version)
);

CREATE TABLE audio_similarity_feature_stats (
  version integer PRIMARY KEY,
  vector_dimension integer NOT NULL,
  means bytea NOT NULL,
  std_devs bytea NOT NULL,
  updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE audio_mix_profiles (
  audio_hash bytea NOT NULL REFERENCES audio_assets(hash) ON DELETE CASCADE,
  version integer NOT NULL,
  provider varchar NOT NULL,
  provider_version varchar NOT NULL,
  bpm real,
  beat_count integer,
  key_root varchar,
  key_mode varchar,
  key_confidence real,
  loudness_integrated_lufs real,
  loudness_range_lu real,
  true_peak_db real,
  duration_seconds real,
  leading_silence_ms integer,
  trailing_silence_ms integer,
  head_rms_dbfs real,
  tail_rms_dbfs real,
  payload jsonb NOT NULL,
  analyzed_at timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (audio_hash, version)
);

CREATE TABLE audio_analysis_failures (
  audio_hash bytea NOT NULL REFERENCES audio_assets(hash) ON DELETE CASCADE,
  analyzer varchar NOT NULL,
  version integer NOT NULL,
  error text NOT NULL,
  failed_at timestamptz NOT NULL DEFAULT now(),
  PRIMARY KEY (audio_hash, analyzer, version)
);

CREATE TABLE song_hash_failures (
  song integer PRIMARY KEY REFERENCES song(id) ON DELETE CASCADE,
  error text NOT NULL,
  failed_at timestamptz NOT NULL DEFAULT now()
);
