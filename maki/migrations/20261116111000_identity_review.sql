-- Ambiguous identity matches are recorded instead of guessed.
CREATE TABLE IF NOT EXISTS identity_review (
  id serial PRIMARY KEY,
  kind varchar NOT NULL,
  target integer NOT NULL,
  detail text NOT NULL,
  created_at timestamptz NOT NULL DEFAULT now(),
  UNIQUE (kind, target)
);
