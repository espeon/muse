-- Pruning is destructive; record which tree produced the last accepted scan so a scan of a
-- different root can never prune rows belonging to the real library.
CREATE TABLE IF NOT EXISTS index_state (
  key varchar PRIMARY KEY,
  value varchar NOT NULL
);
