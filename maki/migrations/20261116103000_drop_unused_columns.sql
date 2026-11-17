-- Provenance was written on album/song but never read; keep the in-memory MetadataSource enum
-- for fallback bookkeeping and drop the unused persistence.
ALTER TABLE album DROP COLUMN IF EXISTS metadata_source;
ALTER TABLE song DROP COLUMN IF EXISTS metadata_source;
