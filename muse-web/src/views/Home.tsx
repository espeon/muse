import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { AlbumCard } from "@/components/AlbumCard";
import { HeroBanner } from "@/components/HeroBanner";
import { HorizontalScroll } from "@/components/HorizontalScroll";
import { Button } from "@/components/ui/button";
import { resumeAudioContext } from "@/player/audio-context";
import type { TrackInfo } from "@/player/types";
import { usePlayer } from "@/player/use-player";
import { login, useSession } from "@/lib/auth";
import { getAlbum, getHome } from "@/lib/api";
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

export function Home() {
  const { isLoggedIn } = useSession();
  const player = usePlayer();
  const navigate = useNavigate();
  const { data: rows, isLoading, error } = useQuery({
    queryKey: ["home"],
    queryFn: getHome,
    enabled: isLoggedIn,
  });

  async function playAlbum(id: number) {
    // AudioContext must be resumed from this user gesture.
    resumeAudioContext();
    const album = await getAlbum(id);
    await player.playTracks((album.tracks ?? []).map(toTrackInfo));
  }

  if (!isLoggedIn) {
    return (
      <div className="flex flex-col items-center gap-4 px-6 py-24 text-center">
        <h1 className="text-2xl font-semibold tracking-tight">muse</h1>
        <p className="max-w-sm text-sm text-muted-foreground">
          Sign in with your Maki account to listen.
        </p>
        <Button onClick={login}>Sign in with Maki</Button>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="flex items-center gap-3 p-6 text-sm text-muted-foreground">
        <span className="inline-block h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent" />
        Loading…
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

  const heroAlbum = rows?.[0]?.albums?.[0];

  return (
    <div className="flex flex-col gap-8 p-4 md:p-6">
      <h1 className="text-2xl font-bold tracking-tight">Home</h1>

      {heroAlbum && (
        <HeroBanner
          album={heroAlbum}
          onClick={() => void navigate({ to: "/album/$albumId", params: { albumId: String(heroAlbum.id) } })}
          onPlay={() => void playAlbum(heroAlbum.id)}
        />
      )}

      {rows?.map((row) => (
        <section key={row.name} className="flex flex-col gap-3">
          <div className="flex items-baseline justify-between px-1">
            <h2 className="text-lg font-semibold tracking-tight">{row.name}</h2>
          </div>
          <HorizontalScroll>
            {row.albums.map((album) => (
              <AlbumCard
                key={album.id}
                album={album}
                onClick={() => void navigate({ to: "/album/$albumId", params: { albumId: String(album.id) } })}
                onPlay={() => void playAlbum(album.id)}
                className="w-36 shrink-0 md:w-44"
              />
            ))}
          </HorizontalScroll>
        </section>
      ))}
    </div>
  );
}
