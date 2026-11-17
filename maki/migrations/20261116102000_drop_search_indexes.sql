-- `ILIKE '%q%'` is not left-anchored, so btree varchar_pattern_ops indexes cannot serve it.
-- They only added write amplification on every scan.
DROP INDEX IF EXISTS idx_artist_search_key;
DROP INDEX IF EXISTS idx_album_search_key;
DROP INDEX IF EXISTS idx_song_search_key;
