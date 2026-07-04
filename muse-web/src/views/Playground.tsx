import { useState } from "react";
import { resumeAudioContext } from "@/player/audio-context";
import type { TrackInfo } from "@/player/types";
import { usePlayer } from "@/player/use-player";
import { cn, formatTime } from "@/lib/utils";
import { getAlbum } from "@/lib/api";
import { useSession } from "@/lib/auth";
import type { Track } from "@/types";

function toTrackInfo(t: Track): TrackInfo {
  return {
    id: t.id,
    title: t.name,
    artistName: t.artist_name,
    albumName: t.album_name,
    artUrl: t.art_url,
    duration: t.duration,
    artists: t.artists,
    albumId: t.album,
    albumArtistId: t.album_artist,
  };
}

/** Dev tool at /playground: load an arbitrary album by id and play it through
 *  the gapless engine. The app shell provides sign-in + the player bar. */
export function Playground() {
  const player = usePlayer();
  const { isLoggedIn } = useSession();
  const [albumId, setAlbumId] = useState("");
  const [tracks, setTracks] = useState<Track[]>([]);
  const [albumName, setAlbumName] = useState("");
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function loadAlbum() {
    setError(null);
    setLoading(true);
    try {
      const album = await getAlbum(albumId.trim());
      setTracks(album.tracks ?? []);
      setAlbumName(album.name);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      setTracks([]);
    } finally {
      setLoading(false);
    }
  }

  async function playFrom(index: number) {
    resumeAudioContext();
    await player.playTracks(tracks.map(toTrackInfo), index);
  }

  return (
    <div className="p-4 md:p-6">
      <h1 className="mb-4 text-sm font-semibold tracking-tight text-muted-foreground">
        muse-web · gapless playground
      </h1>
      {!isLoggedIn ? (
        <p className="text-sm text-muted-foreground">Sign in first.</p>
      ) : (
        <div className="flex flex-col gap-4">
          <div className="flex flex-wrap items-center gap-2">
            <input
              placeholder="album id"
              value={albumId}
              onChange={(e) => setAlbumId(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") void loadAlbum();
              }}
              className="w-32 rounded-md border border-input bg-background px-2.5 py-1.5 text-sm"
            />
            <button
              type="button"
              onClick={() => void loadAlbum()}
              disabled={loading || !albumId.trim()}
              className="rounded-md bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-40"
            >
              {loading ? "loading…" : "load album"}
            </button>
          </div>
          {error && <p className="text-xs text-destructive">{error}</p>}
          {tracks.length > 0 && (
            <div>
              <div className="mb-2 flex items-center justify-between gap-2">
                <h2 className="truncate font-semibold">{albumName}</h2>
                <button
                  type="button"
                  onClick={() => void playFrom(0)}
                  className="rounded-md bg-primary px-3 py-1.5 text-xs font-medium text-primary-foreground hover:bg-primary/90"
                >
                  play
                </button>
              </div>
              <ol className="divide-y divide-border">
                {tracks.map((t, i) => {
                  const active = player.currentIndex === i;
                  return (
                    <li key={t.id}>
                      <button
                        type="button"
                        onClick={() => void playFrom(i)}
                        className={cn(
                          "flex w-full items-center gap-3 px-1 py-2 text-left text-sm hover:bg-accent/50",
                          active && "text-primary",
                        )}
                      >
                        <span className="w-5 shrink-0 text-right tabular-nums text-muted-foreground">
                          {t.number ?? i + 1}
                        </span>
                        <span className="min-w-0 flex-1 truncate">{t.name}</span>
                        <span className="shrink-0 text-xs tabular-nums text-muted-foreground">
                          {formatTime(t.duration)}
                        </span>
                      </button>
                    </li>
                  );
                })}
              </ol>
            </div>
          )}
        </div>
      )}
    </div>
  );
}
