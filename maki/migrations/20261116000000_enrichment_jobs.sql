-- Network-dependent metadata work runs after the file scan, not inside it. A scan of N
-- files must not make N * provider requests before the first song is importable.
-- `target` has no FK: it is an id in a different table depending on `kind`.
CREATE TABLE enrichment_job (
  id serial PRIMARY KEY,
  kind varchar NOT NULL,
  target integer NOT NULL,
  state varchar NOT NULL DEFAULT 'pending'
    CHECK (state IN ('pending', 'running', 'done', 'failed')),
  attempts integer NOT NULL DEFAULT 0,
  not_before timestamptz NOT NULL DEFAULT now(),
  last_error text,
  created_at timestamptz NOT NULL DEFAULT now(),
  updated_at timestamptz,
  UNIQUE (kind, target)
);

CREATE INDEX idx_enrichment_pending ON enrichment_job (state, not_before, id);
