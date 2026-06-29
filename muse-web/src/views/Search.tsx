import { useEffect, useMemo, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { useNavigate } from "@tanstack/react-router";
import { Disc, ListMusic, Mic2, Music, Play, Search as SearchIcon } from "lucide-react";
import { Input } from "@/components/ui/input";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { getAlbums, getArtists, getPlaylists, searchSongs } from "@/lib/api";
import { cn } from "@/lib/utils";
import { resumeAudioContext } from "@/player/audio-context";
import { usePlayer } from "@/player/use-player";
import type { AlbumPartial, ArtistPartial, PlaylistSummary, SearchSong } from "@/types";

function normalize(q: string) {
  return q.trim().toLowerCase();
}

function matches(q: string, ...fields: (string | undefined | null)[]) {
  if (!q) return false;
  const n = normalize(q);
  return fields.some((f) => f && normalize(f).includes(n));
}

function useDebounce<T>(value: T, delay = 250) {
  const [debounced, setDebounced] = useState(value);
  useEffect(() => {
    const t = setTimeout(() => setDebounced(value), delay);
    return () => clearTimeout(t);
  }, [value, delay]);
  return debounced;
}

export function Search() {
  const [query, setQuery] = useState("");
  const debounced = useDebounce(query);
  const navigate = useNavigate();
  const player = usePlayer();

  const songsQuery = useQuery({
    queryKey: ["search", "songs", debounced],
    queryFn: () => searchSongs(debounced),
    enabled: debounced.length > 0,
  });

  const albumsQuery = useQuery({
    queryKey: ["albums"],
    queryFn: getAlbums,
    staleTime: Infinity,
  });
  const artistsQuery = useQuery({
    queryKey: ["artists"],
    queryFn: getArtists,
    staleTime: Infinity,
  });
  const playlistsQuery = useQuery({
    queryKey: ["playlists"],
    queryFn: getPlaylists,
    staleTime: Infinity,
  });

  const albums = useMemo(() => {
    if (!debounced) return [];
    return (albumsQuery.data?.albums ?? []).filter((a: AlbumPartial) =>
      matches(debounced, a.name, a.artist?.name),
    );
  }, [debounced, albumsQuery.data?.albums]);

  const artists = useMemo(() => {
    if (!debounced) return [];
    return (artistsQuery.data?.artists ?? []).filter((a: ArtistPartial) =>
      matches(debounced, a.name),
    );
  }, [debounced, artistsQuery.data?.artists]);

  const playlists = useMemo(() => {
    if (!debounced) return [];
    return (playlistsQuery.data ?? []).filter((p: PlaylistSummary) =>
      matches(debounced, p.name, p.description),
    );
  }, [debounced, playlistsQuery.data]);

  async function playSong(song: SearchSong) {
    resumeAudioContext();
    await player.playTracks([
      {
        id: song.id,
        title: song.song_name,
        artistName: song.artist_name,
        albumName: song.album_name,
        artUrl: song.picture,
        duration: 0,
      },
    ]);
  }

  return (
    <div className="flex flex-col gap-6 p-4 md:p-6">
      <h1 className="text-2xl font-bold tracking-tight">Search</h1>

      <div className="relative">
        <SearchIcon
          size={18}
          className="absolute left-3 top-1/2 -translate-y-1/2 text-muted-foreground"
        />
        <Input
          type="search"
          placeholder="Search your library"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          className="h-11 rounded-full pl-10"
          autoFocus
        />
      </div>

      <Tabs defaultValue="songs" className="w-full">
        <TabsList variant="line" className="w-full justify-start">
          <TabsTrigger value="songs">
            <Music size={16} />
            Songs
          </TabsTrigger>
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

        <TabsContent value="songs" className="mt-4">
          {!debounced ? (
            <Empty message="Type something to search songs." />
          ) : songsQuery.isLoading ? (
            <Loading />
          ) : songsQuery.error ? (
            <ErrorMessage error={songsQuery.error} />
          ) : songsQuery.data?.length ? (
            <div className="flex flex-col gap-0.5">
              {songsQuery.data.map((song) => (
                <SongRow key={song.id} song={song} onPlay={() => void playSong(song)} />
              ))}
            </div>
          ) : (
            <Empty message={`No songs matching "${debounced}".`} />
          )}
        </TabsContent>

        <TabsContent value="albums" className="mt-4">
          {!debounced ? (
            <Empty message="Type something to search albums." />
          ) : albums.length ? (
            <EntityGrid
              items={albums.map((a) => ({
                id: a.id,
                title: a.name,
                subtitle: a.artist?.name,
                image: a.art?.[0],
                onClick: () =>
                  void navigate({
                    to: "/album/$albumId",
                    params: { albumId: String(a.id) },
                  }),
              }))}
            />
          ) : (
            <Empty message={`No albums matching "${debounced}".`} />
          )}
        </TabsContent>

        <TabsContent value="artists" className="mt-4">
          {!debounced ? (
            <Empty message="Type something to search artists." />
          ) : artists.length ? (
            <EntityGrid
              items={artists.map((a) => ({
                id: a.id,
                title: a.name,
                subtitle: `${a.num_albums ?? 0} albums`,
                image: a.picture,
                round: true,
                onClick: () =>
                  void navigate({
                    to: "/artist/$artistId",
                    params: { artistId: String(a.id) },
                  }),
              }))}
            />
          ) : (
            <Empty message={`No artists matching "${debounced}".`} />
          )}
        </TabsContent>

        <TabsContent value="playlists" className="mt-4">
          {!debounced ? (
            <Empty message="Type something to search playlists." />
          ) : playlists.length ? (
            <EntityGrid
              items={playlists.map((p) => ({
                id: p.id,
                title: p.name,
                subtitle: `${p.track_count} tracks`,
                image: p.art_path,
                onClick: () =>
                  void navigate({
                    to: "/playlist/$playlistId",
                    params: { playlistId: String(p.id) },
                  }),
              }))}
            />
          ) : (
            <Empty message={`No playlists matching "${debounced}".`} />
          )}
        </TabsContent>
      </Tabs>
    </div>
  );
}

function SongRow({ song, onPlay }: { song: SearchSong; onPlay: () => void }) {
  return (
    <button
      type="button"
      onClick={onPlay}
      className="group flex h-14 w-full items-center gap-3 rounded-lg px-2 text-left transition-colors hover:bg-white/5"
    >
      {song.picture ? (
        <img src={song.picture} alt="" className="h-11 w-11 rounded-md object-cover" />
      ) : (
        <div className="flex h-11 w-11 shrink-0 items-center justify-center rounded-md bg-accent">
          <Music size={18} className="text-muted-foreground" />
        </div>
      )}
      <div className="min-w-0 flex-1">
        <div className="truncate text-sm font-semibold">{song.song_name}</div>
        <div className="truncate text-xs text-muted-foreground">
          {song.artist_name} — {song.album_name}
        </div>
      </div>
      <span className="flex h-8 w-8 items-center justify-center rounded-full opacity-0 transition-opacity group-hover:opacity-100">
        <Play size={16} className="fill-current" />
      </span>
    </button>
  );
}

function EntityGrid({
  items,
}: {
  items: {
    id: number;
    title: string;
    subtitle?: string | null;
    image?: string;
    round?: boolean;
    onClick: () => void;
  }[];
}) {
  return (
    <div className="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6">
      {items.map((item) => (
        <button
          key={item.id}
          type="button"
          onClick={item.onClick}
          className="group flex flex-col gap-2 text-left"
        >
          <div
            className={cn(
              "relative aspect-square w-full overflow-hidden bg-accent shadow-lg shadow-black/20 transition-shadow group-hover:shadow-xl group-hover:shadow-black/30",
              item.round ? "rounded-full" : "rounded-xl",
            )}
          >
            {item.image ? (
              <img
                src={item.image}
                alt=""
                loading="lazy"
                className="h-full w-full object-cover transition-transform duration-300 group-hover:scale-105"
              />
            ) : (
              <div className="flex h-full w-full items-center justify-center">
                <Music size={32} className="text-muted-foreground" />
              </div>
            )}
          </div>
          <div className="min-w-0">
            <div className="truncate text-sm font-semibold">{item.title}</div>
            {item.subtitle ? (
              <div className="truncate text-xs text-muted-foreground">{item.subtitle}</div>
            ) : null}
          </div>
        </button>
      ))}
    </div>
  );
}

function Empty({ message }: { message: string }) {
  return (
    <div className="py-16 text-center text-sm text-muted-foreground">
      {message}
    </div>
  );
}

function Loading() {
  return (
    <div className="flex items-center gap-3 py-16 text-sm text-muted-foreground">
      <span className="inline-block h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent" />
      Searching…
    </div>
  );
}

function ErrorMessage({ error }: { error: unknown }) {
  return (
    <div className="py-16 text-sm text-destructive">
      {error instanceof Error ? error.message : String(error)}
    </div>
  );
}
