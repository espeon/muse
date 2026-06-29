import { useQuery } from "@tanstack/react-query";
import { useParams } from "@tanstack/react-router";
import { Play } from "lucide-react";
import { Button } from "@/components/ui/button";
import { getPlaylist } from "@/lib/api";
import { formatTime } from "@/lib/utils";
import { resumeAudioContext } from "@/player/audio-context";
import type { TrackInfo } from "@/player/types";
import { usePlayer } from "@/player/use-player";
import type { PlaylistTrack } from "@/types";

function toTrackInfo(t: PlaylistTrack): TrackInfo {
  return {
    id: t.song_id,
    title: t.name,
    artistName: t.artist_name,
    albumName: t.album_name,
    artUrl: t.art_url,
    duration: t.duration,
  };
}

export function PlaylistDetail() {
  const { playlistId } = useParams({ from: "/playlist/$playlistId" as never });
  const id = Number(playlistId);
  const player = usePlayer();

  const { data: playlist, isLoading, error } = useQuery({
    queryKey: ["playlist", id],
    queryFn: () => getPlaylist(id),
    enabled: Number.isFinite(id),
  });

  async function playAll() {
    resumeAudioContext();
    await player.playTracks((playlist?.tracks ?? []).map(toTrackInfo));
  }

  async function playTrack(trackIndex: number) {
    resumeAudioContext();
    await player.playTracks((playlist?.tracks ?? []).map(toTrackInfo), trackIndex);
  }

  if (isLoading) {
    return (
      <div className="flex items-center gap-3 p-6 text-sm text-muted-foreground">
        <span className="inline-block h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent" />
        Loading playlist…
      </div>
    );
  }
  if (error) {
    return (
      <div className="p-6 text-sm text-destructive">
        {error instanceof Error ? error.message : String(error)}
      </div>
    );
  }
  if (!playlist) return null;

  return (
    <div className="flex flex-col gap-8 p-4 md:p-6">
      {/* header */}
      <div className="flex flex-col gap-6 md:flex-row md:items-end">
        {playlist.art_path ? (
          <img
            src={playlist.art_path}
            alt=""
            className="aspect-square w-48 shrink-0 rounded-xl object-cover shadow-2xl shadow-black/40 md:w-56"
          />
        ) : (
          <div className="aspect-square w-48 shrink-0 rounded-xl bg-accent md:w-56" />
        )}
        <div className="flex min-w-0 flex-col gap-1">
          <span className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
            Playlist
          </span>
          <h1 className="text-3xl font-bold tracking-tight md:text-4xl">
            {playlist.name}
          </h1>
          {playlist.description ? (
            <p className="max-w-xl text-sm text-muted-foreground">
              {playlist.description}
            </p>
          ) : null}
          <Button
            onClick={() => void playAll()}
            className="mt-3 h-9 w-fit gap-2 rounded-full px-5"
          >
            <Play size={16} className="fill-current" />
            Play
          </Button>
        </div>
      </div>

      {/* tracks */}
      <section className="flex flex-col gap-1">
        {playlist.tracks.map((track, i) => (
          <PlaylistTrackRow
            key={track.item_id}
            track={track}
            index={i}
            onPlay={() => void playTrack(i)}
          />
        ))}
      </section>
    </div>
  );
}

function PlaylistTrackRow({
  track,
  index,
  onPlay,
}: {
  track: PlaylistTrack;
  index: number;
  onPlay?: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onPlay}
      className="group flex h-11 w-full select-none items-center gap-3 rounded-lg px-2 text-left transition-colors hover:bg-white/5"
    >
      <span className="flex h-6 w-6 items-center justify-center text-xs text-muted-foreground group-hover:hidden">
        {index + 1}
      </span>
      <span className="hidden h-6 w-6 items-center justify-center text-xs text-foreground group-hover:flex">
        <Play size={14} className="fill-current" />
      </span>
      {track.art_url ? (
        <img src={track.art_url} alt="" className="h-8 w-8 rounded object-cover" />
      ) : (
        <div className="h-8 w-8 rounded bg-accent" />
      )}
      <div className="min-w-0 flex-1">
        <div className="truncate text-sm">{track.name}</div>
        <div className="truncate text-xs text-muted-foreground">
          {track.artist_name}
        </div>
      </div>
      <span className="w-10 text-right text-xs tabular-nums text-muted-foreground">
        {formatTime(track.duration)}
      </span>
    </button>
  );
}
