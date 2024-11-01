"use client";
import PlayAlbumButtonOnAction from "@/components/playButtonOnAction";
import Dropdown from "@/components/ui/dropdown";
import { ArtistPartial, ArtistPartials } from "@/types/artistPartial";
import Link from "next/link";
import { useCallback, useEffect, useRef, useState } from "react";
import { BiSearch } from "react-icons/bi";

interface ArtistFilterViewProps {
  initialArtists: ArtistPartials;
  initialFilter?: string;
  fetchMoreArtists: ({ ...props }: FilterProps) => Promise<ArtistPartials>;
}

export interface FilterProps {
  sortby: "id" | "artist" | string;
  direction: "asc" | "desc" | string;
  limit: number;
  filter?: string;
  cursor?: number;
}

export default function ArtistFilterView({
  initialArtists,
  initialFilter,
  fetchMoreArtists,
}: ArtistFilterViewProps) {
  // TODO: use the search endpoint
  const [searchQuery, setSearchQuery] = useState(initialFilter ?? "");
  const [artists, setArtists] = useState<ArtistPartials>(initialArtists);
  const [isLoading, setIsLoading] = useState(false);
  const [hasMore, setHasMore] = useState(true);

  const [filterProps, setFilterProps] = useState<FilterProps>({
    sortby: "artist",
    direction: "asc",
    limit: 20,
  });

  const lastArtistRef = useRef<HTMLDivElement>(null);

  const observerCallback = useCallback(
    (entries: IntersectionObserverEntry[]) => {
      const [entry] = entries;
      if (entry.isIntersecting && hasMore && !isLoading) {
        handleFetch();
      }
    },
    [hasMore, isLoading, fetchMoreArtists, filterProps],
  );

  // if filter props change, re-fetch albums with new props
  useEffect(() => {
    console.log("filter props changed, refreshing album list");
    handleFetch(true);
  }, [filterProps, searchQuery]);

  useEffect(() => {
    const observer = new IntersectionObserver(observerCallback, {
      root: null,
      rootMargin: "0px",
      threshold: 0.1,
    });

    if (lastArtistRef.current) {
      observer.observe(lastArtistRef.current);
    }

    return () => {
      if (lastArtistRef.current) {
        observer.unobserve(lastArtistRef.current);
      }
    };
  }, [observerCallback]);

  const handleFetch = async (replace: boolean = false) => {
    setIsLoading(true);
    const newArtists = await fetchMoreArtists({
      limit: filterProps.limit,
      sortby: filterProps.sortby,
      direction: filterProps.direction,
      cursor: replace
        ? undefined
        : artists.artists[artists.artists.length - 1].id,
      filter: searchQuery ? searchQuery : undefined,
    });
    if (replace) {
      setArtists(newArtists);
      setHasMore(newArtists.artists.length >= filterProps.limit);
    } else {
      setArtists((prevArtists) => ({
        ...newArtists,
        artists: [...prevArtists.artists, ...newArtists.artists],
      }));
    }
    setIsLoading(false);
    if (newArtists.artists.length === 0) {
      setHasMore(false);
    }
  };

  console.log(artists);
  const a: ArtistPartial[] = artists.artists;
  return (
    <div>
      <div className="flex flex-row align-center rounded-lg mt-2 md:px-2">
        <div className="flex flex-row  align-middle justify-center mx-4 bg-slate-800 rounded-lg md:max-w-sm flex-1">
          <BiSearch className="mx-2 my-auto text-slate-200" />
          <input
            className="rounded-lg w-full bg-transparent outline-none"
            type="text"
            placeholder="Find an album..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
          />
        </div>
        <Dropdown
          options={[
            { label: "ID", value: "id" },
            { label: "Artist", value: "artist" },
          ]}
          defaultOption="artist"
          selectedOption={filterProps.sortby}
          selectedDirection={filterProps.direction}
          onValueChange={(option: string) => {
            console.log(option);
            setFilterProps({
              ...filterProps,
              sortby: option,
            });
          }}
          onDirectionChange={(direction: "asc" | "desc") => {
            console.log("changed dir to", direction);
            setFilterProps({
              ...filterProps,
              direction: direction,
            });
          }}
        />
      </div>
      <div className="grid grid-cols-[repeat(auto-fit,_minmax(250px,_1fr))] md:gap-8 my-4 mx-2 md:mx-10">
        {a.map((artist, i) => (
          <div
            key={artist.id}
            ref={i === a.length - 1 ? lastArtistRef : null}
            className="group flex flex-row lg:flex-col w-full rounded-md hover:bg-slate-700"
          >
            <div className="relative h-full max-w-16 lg:max-w-full mx-2 mt-2">
              <Link
                href={"/artist/" + artist.id}
                className="block margin-auto aspect-square max-w-16 md:max-w-full h-full w-full"
              >
                <img
                  className="mx-auto max-w-16 md:max-w-full max-h-full aspect-square object-cover self-center contain-content rounded-full margin-auto shadow-xl group-hover:shadow-slate-950 group-hover:scale-[0.95] transition-all duration-700"
                  src={artist.picture}
                />
              </Link>
              <div className="hidden md:block absolute text-pink-600 hover:text-pink-500 opacity-0 group-hover:opacity-100 text-7xl -bottom-2 transition-all duration-300 drop-shadow-lg"></div>
            </div>
            <Link
              href={"/artist/" + artist.id}
              className="flex flex-row justify-start items-center flex-1 pl-4 md:pl-0 py-1 group-hover:scale-[0.98] group-hover:translate-y-[0.1rem] transition-all duration-300 ease-in-out"
            >
              <div className="mx-2 mt-1 group-hover:scale-[0.98] group-hover:translate-y-[0.1rem] transition-all duration-300 ease-in-out">
                <div className="line-clamp-1 text-xl">{artist.name}</div>
                <div className="text-slate-400">
                  {artist.num_albums} album
                  {artist.num_albums && artist.num_albums > 1 ? "s" : ""}
                </div>
              </div>
            </Link>
          </div>
        ))}
        {a.length < 6
          ? Array.from({ length: 9 - a.length }).map((_, i) => (
              <div key={i} className="hidden md:block w-full h-64"></div>
            ))
          : null}
      </div>
    </div>
  );
}
