import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function formatTime(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs.toString().padStart(2, "0")}`;
}

/**
 * Build the artist display string for a track.
 *
 * - If the track has >1 artist in `artists`, show all of them (album artist first).
 * - Otherwise, just show `artistName` (the album artist).
 */
export function displayArtists(track: { artistName?: string; artists?: { name: string }[] }): string {
  const artists = track.artists;
  if (!artists || artists.length <= 1) {
    return track.artistName ?? "";
  }

  // Dedupe by name, album artist (matching artistName) first.
  const seen = new Set<string>();
  const ordered: string[] = [];
  const albumArtistName = track.artistName;

  if (albumArtistName && !seen.has(albumArtistName)) {
    ordered.push(albumArtistName);
    seen.add(albumArtistName);
  }

  for (const a of artists) {
    if (!seen.has(a.name)) {
      ordered.push(a.name);
      seen.add(a.name);
    }
  }

  return ordered.join(", ");
}
