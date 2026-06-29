import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { Disc, ListMusic, Mic2, Play, User } from "lucide-react";
import { AlbumCard } from "@/components/AlbumCard";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { getAlbum, getAlbums, getArtists, getPlaylists } from "@/lib/api";
import { resumeAudioContext } from "@/player/audio-context";
import { usePlayer } from "@/player/use-player";
import type { AlbumPartial, ArtistPartial, PlaylistSummary } from "@/types";

function Empty({ message }: { message: string }) {
  return (
    <div className="flex flex-col items-center justify-center py-16 text-center text-sm text-muted-foreground">
      {message}
    </div>
  );
}

function ArtistCard({
  artist,
  onClick,
}: {
  artist: ArtistPartial;
  onClick?: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="group flex flex-col items-center gap-2 text-center"
    >
      <div className="relative aspect-square w-full overflow-hidden rounded-full bg-accent shadow-lg shadow-black/20 transition-shadow group-hover:shadow-xl group-hover:shadow-black/30">
        {artist.picture ? (
          <img
            src={artist.picture}
            alt=""
            loading="lazy"
            className="h-full w-full object-cover transition-transform duration-300 group-hover:scale-105"
          />
        ) : (
          <div className="flex h-full w-full items-center justify-center">
            <User size={32} className="text-muted-foreground" />
          </div>
        )}
      </div>
      <div className="w-full min-w-0">
        <div className="truncate text-sm font-semibold">{artist.name}</div>
      </div>
    </button>
  );
}

function PlaylistCard({
  playlist,
  onClick,
  onPlay,
}: {
  playlist: PlaylistSummary;
  onClick?: () => void;
  onPlay?: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className="group flex w-full flex-col items-start gap-2 text-left"
    >
      <div className="relative aspect-square w-full overflow-hidden rounded-xl bg-accent shadow-lg shadow-black/20 transition-shadow group-hover:shadow-xl group-hover:shadow-black/30">
        {playlist.art_path ? (
          <img
            src={playlist.art_path}
            alt=""
            loading="lazy"
            className="h-full w-full object-cover transition-transform duration-300 group-hover:scale-105"
          />
        ) : (
          <div className="flex h-full w-full items-center justify-center">
            <ListMusic size={32} className="text-muted-foreground" />
          </div>
        )}
        {onPlay && (
          <span
            className="absolute bottom-2 right-2 flex h-10 w-10 items-center justify-center rounded-full bg-primary text-primary-foreground opacity-0 shadow-lg transition-all duration-200 group-hover:translate-y-0 group-hover:opacity-100 translate-y-1"
            onClick={(e) => {
              e.stopPropagation();
              onPlay();
            }}
          >
            <Play size={16} className="ml-0.5 fill-current" />
          </span>
        )}
      </div>
      <div className="w-full min-w-0">
        <div className="truncate text-sm font-semibold">{playlist.name}</div>
        <div className="truncate text-xs text-muted-foreground">
          {playlist.track_count} tracks
        </div>
      </div>
    </button>
  );
}

export function Library() {
  const navigate = useNavigate();
  const player = usePlayer();

  const albumsQuery = useQuery({
    queryKey: ["albums"],
    queryFn: getAlbums,
  });
  const artistsQuery = useQuery({
    queryKey: ["artists"],
    queryFn: getArtists,
  });
  const playlistsQuery = useQuery({
    queryKey: ["playlists"],
    queryFn: getPlaylists,
  });

  async function playAlbum(id: number) {
    resumeAudioContext();
    const album = await getAlbum(id);
    await player.playTracks(
      (album.tracks ?? []).map((t) => ({
        id: t.id,
        title: t.name,
        artistName: t.artist_name,
        albumName: t.album_name,
        artUrl: t.art_url,
        duration: t.duration,
      })),
    );
  }

  return (
    <div className="flex flex-col gap-6 p-4 md:p-6">
      <h1 className="text-2xl font-bold tracking-tight">Library</h1>

      <Tabs defaultValue="albums" className="w-full">
        <TabsList variant="default" className="w-full justify-start">
          <TabsTrigger value="albums">
            <Disc size={16} />
            Albums
          </TabsTrigger>
          <TabsTrigger value="artists">
            <Mic2 size={16} />
            Artists
          </TabsTrigger>
          <TabsTrigger value="playlists">
            <ListMusic size={16} />
            Playlists
          </TabsTrigger>
        </TabsList>

        <TabsContent value="albums" className="mt-4">
          {albumsQuery.isLoading ? (
            <div className="flex items-center gap-3 py-12 text-sm text-muted-foreground">
              <span className="inline-block h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent" />
              Loading albums…
            </div>
          ) : albumsQuery.error ? (
            <div className="py-12 text-sm text-destructive">
              {albumsQuery.error instanceof Error
                ? albumsQuery.error.message
                : String(albumsQuery.error)}
            </div>
          ) : albumsQuery.data?.albums.length ? (
            <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
              {albumsQuery.data.albums.map((album: AlbumPartial) => (
                <AlbumCard
                  key={album.id}
                  album={album}
                  onClick={() =>
                    void navigate({
                      to: "/album/$albumId",
                      params: { albumId: String(album.id) },
                    })
                  }
                  onPlay={() => void playAlbum(album.id)}
                />
              ))}
            </div>
          ) : (
            <Empty message="No albums in your library yet." />
          )}
        </TabsContent>

        <TabsContent value="artists" className="mt-4">
          {artistsQuery.isLoading ? (
            <div className="flex items-center gap-3 py-12 text-sm text-muted-foreground">
              <span className="inline-block h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent" />
              Loading artists…
            </div>
          ) : artistsQuery.error ? (
            <div className="py-12 text-sm text-destructive">
              {artistsQuery.error instanceof Error
                ? artistsQuery.error.message
                : String(artistsQuery.error)}
            </div>
          ) : artistsQuery.data?.artists.length ? (
            <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
              {artistsQuery.data.artists.map((artist: ArtistPartial) => (
                <ArtistCard
                  key={artist.id}
                  artist={artist}
                  onClick={() =>
                    void navigate({
                      to: "/artist/$artistId",
                      params: { artistId: String(artist.id) },
                    })
                  }
                />
              ))}
            </div>
          ) : (
            <Empty message="No artists in your library yet." />
          )}
        </TabsContent>

        <TabsContent value="playlists" className="mt-4">
          {playlistsQuery.isLoading ? (
            <div className="flex items-center gap-3 py-12 text-sm text-muted-foreground">
              <span className="inline-block h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent" />
              Loading playlists…
            </div>
          ) : playlistsQuery.error ? (
            <div className="py-12 text-sm text-destructive">
              {playlistsQuery.error instanceof Error
                ? playlistsQuery.error.message
                : String(playlistsQuery.error)}
            </div>
          ) : playlistsQuery.data?.length ? (
            <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
              {playlistsQuery.data.map((playlist: PlaylistSummary) => (
                <PlaylistCard
                  key={playlist.id}
                  playlist={playlist}
                  onClick={() =>
                    void navigate({
                      to: "/playlist/$playlistId",
                      params: { playlistId: String(playlist.id) },
                    })
                  }
                />
              ))}
            </div>
          ) : (
            <Empty message="No playlists yet." />
          )}
        </TabsContent>
      </Tabs>
    </div>
  );
}
