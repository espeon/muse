-- Folder-derived album identity. `edition_key` is a local, derived id, not a public slug.
ALTER TABLE album ADD COLUMN edition_key varchar;
ALTER TABLE album ADD COLUMN metadata_source varchar;   -- 'tag' | 'folder' | 'musicbrainz'
ALTER TABLE album ADD COLUMN scan_complete bool NOT NULL DEFAULT true;
ALTER TABLE song ADD COLUMN metadata_source varchar;
CREATE INDEX idx_album_edition_key ON album(edition_key);

-- Backfill from existing file paths. An album whose songs come from more than one folder was
-- created by the old name-only matching and cannot be assigned a folder identity safely:
-- mark it incomplete so album-level matching leaves it alone until a rescan re-derives it.
UPDATE album a
SET edition_key = f.folder, metadata_source = 'folder'
FROM (
  SELECT album, min(folder) AS folder
  FROM (SELECT album, regexp_replace(path, '/[^/]*$', '') AS folder FROM song) s
  GROUP BY album HAVING count(DISTINCT folder) = 1
) f
WHERE a.id = f.album;

UPDATE album a
SET scan_complete = false
FROM (
  SELECT album
  FROM (SELECT album, regexp_replace(path, '/[^/]*$', '') AS folder FROM song) s
  GROUP BY album HAVING count(DISTINCT folder) > 1
) f
WHERE a.id = f.album;
