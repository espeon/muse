-- Lease ownership: a reclaim must not let a second worker run a job that is still executing.
ALTER TABLE enrichment_job ADD COLUMN claim_id varchar;
ALTER TABLE enrichment_job ADD COLUMN lease_expires_at timestamptz;

-- Reclaim jobs stranded by an older process version (no lease columns yet).
UPDATE enrichment_job SET state = 'pending', not_before = now()
WHERE state = 'running' AND claim_id IS NULL;
