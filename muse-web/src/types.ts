/**
 * Maki API models — mirror the Rust structs in maki/src/api/mod.rs.
 * Only fields muse-web consumes are typed; optionals follow the Rust `Option`.
 */

export interface ArtistPartial {
  id: number;
  slug?: string;
  name: string;
  picture?: string;
  num_albums?: number;
}

/** Full artist response from /api/v1/artist/{id}. */
export interface Artist {
  id: number;
  slug: string;
  name: string;
  picture?: string;
  tags?: string;
  bio?: string;
  created_at: string;
  updated_at?: number;
  albums: AlbumPartial[];
}

/** Track entry inside a playlist response. */
export interface PlaylistTrack {
  item_id: number;
  song_id: number;
  name: string;
  duration: number;
  number?: number;
  disc?: number;
  liked?: boolean;
  lossless?: boolean;
  album_id: number;
  album_name: string;
  artist_name: string;
  art_url?: string;
}

/** Full playlist response from /api/v1/playlist/{id}. */
export interface Playlist {
  id: number;
  name: string;
  description?: string;
  art_path?: string;
  created_at: string;
  updated_at: string;
  tracks: PlaylistTrack[];
}

/** Track entry returned by /api/v1/search/{slug}. */
export interface SearchSong {
  id: number;
  song_name: string;
  artist_name: string;
  album_name: string;
  artist_id?: number;
  album_id?: number;
  picture?: string;
}

export interface PlaylistSummary {
  id: number;
  name: string;
  description?: string;
  art_path?: string;
  track_count: number;
  created_at: string;
  updated_at: string;
}

export interface Track {
  id: number;
  slug: string;
  name: string;
  album_artist: number;
  artists: ArtistPartial[];
  plays?: number;
  duration: number;
  liked?: boolean;
  /** unix seconds (time::serde::timestamp) */
  last_play?: number;
  year?: number;
  number?: number;
  disc?: number;
  lossless?: boolean;
  sample_rate?: number;
  bits_per_sample?: number;
  num_channels?: number;
  composer?: string;
  isrc?: string;
  bpm?: number;
  /** RFC3339 (time::serde::rfc3339) */
  created_at: string;
  /** unix seconds */
  updated_at?: number;
  album: number;
  album_name: string;
  artist_name: string;
  art_url?: string;
}

export interface Album {
  id: number;
  slug: string;
  name: string;
  disambiguation?: string;
  art: string[];
  year?: number;
  genres: string[];
  copyright?: string;
  label?: string;
  created_at: string;
  updated_at?: number;
  artist: ArtistPartial;
  tracks?: Track[];
}

/** Lightweight album used in lists/home/carousels (maki AlbumPartial). */
export interface AlbumPartial {
  id: number;
  slug?: string;
  name: string;
  disambiguation?: string;
  /** One or more art URLs (an album can have multiple covers). */
  art: string[];
  year?: number;
  count?: number;
  artist?: ArtistPartial;
}

/** Home feed row type (maki HomeRowType). */
export type HomeRowType = "Album" | "Artist" | "Track";

/** A titled horizontal section of the home feed (maki HomeRow). */
export interface HomeRow {
  name: string;
  albums: AlbumPartial[];
  row_type: HomeRowType;
  /** Optional deep-link target, e.g. muse://album/<id>. */
  resource: string | null;
}

/** Authenticated user info from /api/v1/me. */
export interface Me {
  id: number;
  name?: string;
  email?: string;
  picture?: string;
  is_admin: boolean;
  lastfm_connected: boolean;
}

/** HLS quality profile from /api/v1/hls/profiles. */
export interface HlsProfile {
  name: string;
  codec: string;
  /** bps, null for lossless */
  bitrate?: number | null;
}
