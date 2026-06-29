import { useQuery } from "@tanstack/react-query";
import { useNavigate, useParams } from "@tanstack/react-router";
import { Play } from "lucide-react";
import { AlbumCard } from "@/components/AlbumCard";
import { Button } from "@/components/ui/button";
import { HorizontalScroll } from "@/components/HorizontalScroll";
import { TrackRow } from "@/components/TrackRow";
import { getAlbum, getArtist } from "@/lib/api";
import { resumeAudioContext } from "@/player/audio-context";
import type { TrackInfo } from "@/player/types";
import { usePlayer } from "@/player/use-player";
import type { Track } from "@/types";

function toTrackInfo(t: Track): TrackInfo {
  return {
    id: t.id,
    title: t.name,
    artistName: t.artist_name,
    albumName: t.album_name,
    artUrl: t.art_url,
    duration: t.duration,
  };
}

export function AlbumDetail() {
  const { albumId } = useParams({ from: "/album/$albumId" as never });
  const player = usePlayer();
  const navigate = useNavigate();
  const id = Number(albumId);

  const {
    data: album,
    isLoading,
    error,
  } = useQuery({
    queryKey: ["album", id],
    queryFn: () => getAlbum(id),
    enabled: Number.isFinite(id),
  });

  const artistId = album?.artist?.id;
  const { data: artist } = useQuery({
    queryKey: ["artist", artistId],
    queryFn: () => getArtist(artistId!),
    enabled: artistId !== undefined,
  });

  async function playAlbum() {
    resumeAudioContext();
    await player.playTracks((album?.tracks ?? []).map(toTrackInfo));
  }

  async function playTrack(_track: Track, index: number) {
    resumeAudioContext();
    await player.playTracks((album?.tracks ?? []).map(toTrackInfo), index);
  }

  if (isLoading) {
    return (
      <div className="flex items-center gap-3 p-6 text-sm text-muted-foreground">
        <span className="inline-block h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent" />
        Loading album…
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
  if (!album) return null;

  const cover = album.art?.[0];

  return (
    <div className="flex flex-col gap-8 p-4 md:p-6 mt-16">
      {/* header */}
      <div className="flex flex-col gap-6 md:flex-row md:items-end">
        {cover ? (
          <img
            src={cover}
            alt=""
            className="aspect-square w-48 shrink-0 rounded-xl object-cover shadow-2xl shadow-black/40 md:w-56 border"
          />
        ) : (
          <div className="aspect-square w-48 shrink-0 rounded-xl bg-accent md:w-56" />
        )}
        <div className="flex min-w-0 flex-col gap-1">
          <h1 className="text-3xl font-bold tracking-tight md:text-4xl">
            {album.name}
          </h1>
          {album.artist && (
            <button
              type="button"
              onClick={() =>
                void navigate({
                  to: "/artist/$artistId",
                  params: { artistId: String(album.artist.id) },
                })
              }
              className="text-lg font-medium text-primary hover:underline md:text-xl mr-auto"
            >
              {album.artist.name}
            </button>
          )}
          <p className="text-sm text-muted-foreground">
            {[album.genres?.[0], album.year].filter(Boolean).join(" · ")}
          </p>
          {album.disambiguation ? (
            <p className="mt-1 max-w-xl text-sm leading-relaxed text-muted-foreground">
              {album.disambiguation}
            </p>
          ) : null}
          <Button
            onClick={() => void playAlbum()}
            className="mt-3 h-9 w-fit gap-2 rounded-full px-5"
          >
            <Play size={16} className="fill-current" />
            Play
          </Button>
        </div>
      </div>

      {/* tracks */}
      <section className="flex flex-col gap-1">
        {album.tracks?.map((track, i) => (
          <TrackRow
            key={track.id}
            track={track}
            index={i}
            onPlay={() => void playTrack(track, i)}
          />
        ))}
      </section>

      {/* more by artist */}
      {artist && artist.albums.length > 0 ? (
        <section className="flex flex-col gap-3">
          <h2 className="text-lg font-semibold tracking-tight">
            More By {artist.name}
          </h2>
          <HorizontalScroll>
            {artist.albums.map((a) => (
              <AlbumCard
                key={a.id}
                album={a}
                onClick={() =>
                  void navigate({
                    to: "/album/$albumId",
                    params: { albumId: String(a.id) },
                  })
                }
                className="w-36 shrink-0 md:w-44"
              />
            ))}
          </HorizontalScroll>
        </section>
      ) : null}
    </div>
  );
}
