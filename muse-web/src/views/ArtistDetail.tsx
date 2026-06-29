import { useQuery } from "@tanstack/react-query";
import { useNavigate, useParams } from "@tanstack/react-router";
import { Play } from "lucide-react";
import { AlbumCard } from "@/components/AlbumCard";
import { HorizontalScroll } from "@/components/HorizontalScroll";
import { ShaderBackground } from "@/components/ShaderBackground";
import { getArtist } from "@/lib/api";

function ArtistAvatar({
  src,
  name,
}: {
  src?: string;
  name: string;
}) {
  return (
    <div className="relative aspect-square w-28 overflow-hidden rounded-full ring-4 ring-white/10 shadow-2xl sm:w-32 md:w-40">
      {src ? (
        <img
          src={src}
          alt={name}
          className="h-full w-full object-cover"
        />
      ) : (
        <div className="flex h-full w-full items-center justify-center bg-accent">
          <span className="text-2xl font-bold text-muted-foreground md:text-3xl">
            {name.charAt(0)}
          </span>
        </div>
      )}
    </div>
  );
}

export function ArtistDetail() {
  const { artistId } = useParams({ from: "/artist/$artistId" as never });
  const id = Number(artistId);

  const { data: artist, isLoading, error } = useQuery({
    queryKey: ["artist", id],
    queryFn: () => getArtist(id),
    enabled: Number.isFinite(id),
  });
  const navigate = useNavigate();

  if (isLoading) {
    return (
      <div className="flex items-center gap-3 p-6 text-sm text-muted-foreground">
        <span className="inline-block h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent" />
        Loading artist…
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
  if (!artist) return null;

  const latest = artist.albums?.[0];

  return (
    <div className="flex flex-col overflow-hidden rounded-2xl">
      {/* hero with animated mesh background and circular pfp */}
      <ShaderBackground
        image={artist.picture}
        className="relative flex min-h-[260px] flex-col items-center justify-end pb-4 pt-12 text-center sm:min-h-[320px] md:pb-6 md:pt-16"
      >
        <ArtistAvatar src={artist.picture} name={artist.name} />
        <h1 className="mt-4 text-3xl font-bold text-white md:text-4xl">
          {artist.name}
        </h1>
        <button
          type="button"
          className="mt-3 flex h-10 w-10 items-center justify-center rounded-full bg-primary text-primary-foreground shadow-lg transition-transform hover:scale-105"
          aria-label="Play"
        >
          <Play size={18} className="ml-0.5 fill-current" />
        </button>
      </ShaderBackground>

      <div className="flex flex-col gap-8 p-4 md:p-6">
        {/* latest release */}
        {latest && (
          <section className="flex flex-col gap-3">
            <h2 className="text-lg font-semibold tracking-tight">Latest Release</h2>
            <AlbumCard
              album={latest}
              onClick={() => void navigate({ to: "/album/$albumId", params: { albumId: String(latest.id) } })}
              className="w-36 md:w-44"
            />
          </section>
        )}

        {/* essential albums */}
        {artist.albums && artist.albums.length > 0 && (
          <section className="flex flex-col gap-3">
            <h2 className="text-lg font-semibold tracking-tight">Essential Albums</h2>
            <HorizontalScroll>
              {artist.albums.map((album) => (
                <AlbumCard
                  key={album.id}
                  album={album}
                  onClick={() => void navigate({ to: "/album/$albumId", params: { albumId: String(album.id) } })}
                  className="w-36 shrink-0 md:w-44"
                />
              ))}
            </HorizontalScroll>
          </section>
        )}

        {/* bio */}
        {artist.bio ? (
          <section className="flex flex-col gap-2">
            <h2 className="text-lg font-semibold tracking-tight">About</h2>
            <p className="max-w-3xl text-sm leading-relaxed text-muted-foreground">
              {artist.bio}
            </p>
          </section>
        ) : null}
      </div>
    </div>
  );
}
