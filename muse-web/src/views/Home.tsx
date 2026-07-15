import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { AlbumCard } from "@/components/AlbumCard";
import { HeroBanner } from "@/components/HeroBanner";
import { HorizontalScroll } from "@/components/HorizontalScroll";
import { Button } from "@/components/ui/button";
import { AlbumCardSkeleton, Skeleton } from "@/components/ui/skeleton";
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
    artists: t.artists,
    albumId: t.album,
    albumArtistId: t.album_artist,
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
      <div className="relative flex min-h-[calc(100dvh-2rem)] flex-col items-center justify-center gap-8 overflow-hidden px-6 text-center">
        {/* Animated gradient backdrop */}
        <div
          className="pointer-events-none absolute inset-0 opacity-40"
          style={{
            background:
              'radial-gradient(ellipse 80% 60% at 50% 0%, rgba(120,60,60,0.3), transparent), radial-gradient(ellipse 60% 50% at 80% 80%, rgba(60,60,120,0.2), transparent), radial-gradient(ellipse 50% 40% at 20% 70%, rgba(80,40,80,0.15), transparent)',
          }}
        />
        {/* Logo */}
        <div className="relative flex items-center justify-center gap-3">
          <div className="flex h-14 w-14 items-center justify-center rounded-2xl bg-gradient-to-br from-neutral-800 to-neutral-900 shadow-2xl shadow-black/50">
          </div>
          <h1 className="text-4xl font-bold tracking-tight">muse</h1>
        </div>
        <div className="relative flex flex-col items-center gap-2">
          <p className="max-w-sm text-base text-muted-foreground">
            Your personal music streaming server.
          </p>
          <p className="text-xs text-muted-foreground/60">
            Stream your library anywhere — lossless, synced lyrics, and more.
          </p>
        </div>
        <Button
          size="lg"
          onClick={login}
          className="relative rounded-full px-8 text-base shadow-lg shadow-black/30 transition-transform hover:scale-105"
        >
          Sign in
        </Button>
      </div>
    );
  }

  if (isLoading) {
    return (
      <div className="flex flex-col gap-8 p-4 md:p-6">
        <Skeleton className="h-8 w-24" />
        <Skeleton className="aspect-[16/7] w-full rounded-2xl" />
        <div className="flex flex-col gap-3">
          <Skeleton className="h-6 w-32" />
          <div className="flex gap-4 overflow-hidden">
            {Array.from({ length: 5 }).map((_, i) => (
              <AlbumCardSkeleton key={i} className="w-36 shrink-0 md:w-44" />
            ))}
          </div>
        </div>
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
