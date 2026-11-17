-- Raw f32 feature vectors are emitted by the external audio-analysis worker.
-- Keep them versioned so feature extractor changes are lazily re-analyzed.
CREATE TABLE song_features (
  song integer PRIMARY KEY REFERENCES song(id) ON DELETE CASCADE,
  version integer NOT NULL,
  vector bytea NOT NULL,
  analyzed_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX idx_song_features_version ON song_features(version);

-- Per-library normalisation statistics. Vectors remain raw in song_features.
CREATE TABLE song_feature_stats (
  version integer PRIMARY KEY,
  vector_dimension integer NOT NULL,
  means bytea NOT NULL,
  std_devs bytea NOT NULL,
  updated_at timestamptz NOT NULL DEFAULT now()
);

-- Avoid retrying corrupt or unsupported files on every tag scan. The admin
-- analysis endpoint explicitly clears these failures for another attempt.
CREATE TABLE song_feature_failures (
  song integer PRIMARY KEY REFERENCES song(id) ON DELETE CASCADE,
  version integer NOT NULL,
  error text NOT NULL,
  failed_at timestamptz NOT NULL DEFAULT now()
);
