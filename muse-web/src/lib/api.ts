import { getAuthHeader } from "@/lib/auth";
import { MAKI_URL, UMI_URL } from "@/lib/config";
import { parseJlf, type Jlf } from "@/lyrics/jlf";
import type {
  Album,
  AlbumPartial,
  Artist,
  ArtistPartial,
  HlsProfile,
  HomeRow,
  Me,
  Playlist,
  PlaylistSummary,
  SearchSong,
} from "@/types";

async function apiFetch<T>(path: string, init?: RequestInit): Promise<T> {
  const auth = await getAuthHeader();
  const headers: Record<string, string> = {
    ...((init?.headers as Record<string, string> | undefined) ?? {}),
  };
  if (auth) headers.Authorization = `Bearer ${auth}`;
  if (init?.body) headers["Content-Type"] = "application/json";

  const res = await fetch(`${MAKI_URL}${path}`, { ...init, headers });
  if (res.status === 401) {
    throw new Error(`${path} → 401 (not authenticated)`);
  }
  if (!res.ok) {
    const body = await res.text().catch(() => "");
    throw new Error(
      `${path} → ${res.status} ${res.statusText}${body ? `: ${body.slice(0, 200)}` : ""}`,
    );
  }
  return (await res.json()) as T;
}

// ----------------------------------------------------------------- read endpoints

export async function getHome(): Promise<HomeRow[]> {
  return apiFetch<HomeRow[]>("/api/v1/home/");
}

export interface AlbumsPage {
  albums: AlbumPartial[];
  limit: number;
  offset: number;
}

export async function getAlbums(cursor?: number): Promise<AlbumsPage> {
  const params = cursor ? `?cursor=${cursor}` : "";
  return apiFetch<AlbumsPage>(`/api/v1/album${params}`);
}

export interface ArtistsPage {
  artists: ArtistPartial[];
  limit: number;
  cursor: number;
}

export async function getArtists(cursor?: number): Promise<ArtistsPage> {
  const params = cursor ? `?cursor=${cursor}` : "";
  return apiFetch<ArtistsPage>(`/api/v1/artist${params}`);
}

export async function getPlaylists(): Promise<PlaylistSummary[]> {
  return apiFetch<PlaylistSummary[]>("/api/v1/playlist");
}

export async function getAlbum(id: number | string): Promise<Album> {
  return apiFetch<Album>(`/api/v1/album/${id}`);
}

export async function getArtist(id: number | string): Promise<Artist> {
  return apiFetch<Artist>(`/api/v1/artist/${id}`);
}

export async function getPlaylist(id: number | string): Promise<Playlist> {
  return apiFetch<Playlist>(`/api/v1/playlist/${id}`);
}

export async function searchSongs(query: string): Promise<SearchSong[]> {
  return apiFetch<SearchSong[]>(`/api/v1/search/${encodeURIComponent(query)}`);
}

// ----------------------------------------------------------------- playback

interface SignResult {
  id: number;
  url: string;
  signed_at: string;
  expires_at: string;
}

/** Sign a single track → absolute, CORS-enabled stream URL (tk-authenticated). */
export async function signTrack(id: number | string): Promise<string> {
  const r = await apiFetch<SignResult>(`/api/v1/track/${id}/sign`);
  return r.url;
}

/** `resolveUrl` for the gapless engine. */
export const resolveTrackUrl = (id: number): Promise<string> => signTrack(id);

// ----------------------------------------------------------------- lyrics

/**
 * Fetch synced lyrics from umi. Returns null on any failure (network, non-200,
 * parse error) so the UI can degrade gracefully — matching the iOS/Android
 * "fail silently" behavior.
 */
export async function fetchLyrics(
  track: string,
  artist: string,
  album: string,
): Promise<Jlf | null> {
  try {
    const params = new URLSearchParams({ track, artist, album });
    const res = await fetch(`${UMI_URL}/lyrics?${params}`);
    if (!res.ok) return null;
    const raw = await res.json();
    return parseJlf(raw);
  } catch {
    return null;
  }
}

// ----------------------------------------------------------------- user

export async function fetchMe(): Promise<Me> {
  return apiFetch<Me>("/api/v1/me");
}

// ----------------------------------------------------------------- like / scrobble

interface LikedResponse {
  id: number;
  liked: boolean;
}

/** Toggle like status for a track. Returns the new liked state. */
export async function toggleLike(trackId: number | string): Promise<boolean> {
  const r = await apiFetch<LikedResponse>(`/api/v1/track/${trackId}/like`, {
    method: "POST",
  });
  return r.liked;
}

/** Tell maki to set the track as "now playing" (also sets Last.fm now-playing). */
export async function setPlaying(trackId: number | string): Promise<void> {
  await apiFetch(`/api/v1/track/${trackId}/play`, { method: "POST" });
}

/** Record a completed play — increments play count and scrobbles to Last.fm. */
export async function scrobbleTrack(trackId: number | string): Promise<void> {
  await apiFetch(`/api/v1/track/${trackId}/scrobble`, { method: "POST" });
}

// ----------------------------------------------------------------- last.fm pairing

export interface LastfmTokenResponse {
  token: string;
  url: string;
}

export interface LastfmSessionResponse {
  session_key: string;
  username: string;
}

/** Step 1: get a Last.fm auth token + the approval URL. */
export async function fetchLastfmToken(): Promise<LastfmTokenResponse> {
  return apiFetch<LastfmTokenResponse>("/api/v1/lastfm/token");
}

/** Step 2: complete the pairing after the user approves on last.fm. */
export async function completeLastfmSession(token: string): Promise<LastfmSessionResponse> {
  const params = new URLSearchParams({ token });
  return apiFetch<LastfmSessionResponse>(`/api/v1/lastfm/session?${params}`, {
    method: "POST",
  });
}

/** Disconnect Last.fm — deletes the stored session key. */
export async function disconnectLastfm(): Promise<void> {
  await apiFetch("/api/v1/lastfm/session", { method: "DELETE" });
}

// ----------------------------------------------------------------- HLS

/** Fetch available HLS quality profiles. */
export async function fetchHlsProfiles(): Promise<HlsProfile[]> {
  return apiFetch<HlsProfile[]>("/api/v1/hls/profiles");
}

/** Sign a track for HLS playback — returns a master.m3u8 URL. */
export async function signTrackHls(trackId: number | string): Promise<string> {
  const r = await apiFetch<SignResult>(`/api/v1/track/${trackId}/sign?mode=hls`);
  return r.url;
}
