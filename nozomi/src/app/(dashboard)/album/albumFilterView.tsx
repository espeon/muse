"use client";
import PlayAlbumButtonOnAction from "@/components/playButtonOnAction";
import { useConfig } from "@/stores/configStore";
import { AlbumPartial, AlbumPartials } from "@/types/albumPartial";
import Link from "next/link";
import { useCallback, useEffect, useRef, useState } from "react";
import { BiSearch } from "react-icons/bi";
import Lottie from "react-lottie";

import loadingAnimation from "@/lotties/throbber.json";
import Dropdown from "@/components/ui/dropdown";
import { ALL } from "dns";

interface AlbumFilterViewProps {
  initialAlbums: AlbumPartials;
  initialFilter?: string;
  fetchMoreAlbums: (props: FilterProps) => Promise<AlbumPartials>;
}

export interface FilterProps {
  sortby: "id" | "artist" | "album" | "year" | string;
  direction: "asc" | "desc" | string;
  limit: number;
  album_id?: number | null;
  filter?: string;
}

export default function AlbumFilterView({
  initialAlbums,
  initialFilter,
  fetchMoreAlbums,
}: AlbumFilterViewProps) {
  // TODO: use the search endpoint
  const [searchQuery, setSearchQuery] = useState(initialFilter);
  const [albums, setAlbums] = useState<AlbumPartials>(initialAlbums);
  const [isLoading, setIsLoading] = useState(false);
  const [hasMore, setHasMore] = useState(true);

  const [filterProps, setFilterProps] = useState<FilterProps>({
    sortby: "artist",
    direction: "asc",
    limit: 20,
  });

  const lastAlbumRef = useRef<HTMLDivElement>(null);

  const observerCallback = useCallback(
    (entries: IntersectionObserverEntry[]) => {
      const [entry] = entries;
      if (entry.isIntersecting && hasMore && !isLoading) {
        handleFetch();
      }
    },
    [hasMore, isLoading, fetchMoreAlbums, filterProps],
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

    if (lastAlbumRef.current) {
      observer.observe(lastAlbumRef.current);
    }

    return () => {
      if (lastAlbumRef.current) {
        observer.unobserve(lastAlbumRef.current);
      }
    };
  }, [observerCallback]);

  const handleFetch = async (replace: boolean = false) => {
    setIsLoading(true);
    const newAlbums = await fetchMoreAlbums({
      limit: filterProps.limit,
      sortby: filterProps.sortby,
      direction: filterProps.direction,
      album_id: replace ? null : albums.albums[albums.albums.length - 1].id,
      filter: searchQuery ? searchQuery : undefined,
    });
    if (replace) {
      setAlbums(newAlbums);
      setHasMore(newAlbums.albums.length >= filterProps.limit);
    } else {
      setAlbums((prevAlbums) => ({
        ...newAlbums,
        albums: [...prevAlbums.albums, ...newAlbums.albums],
      }));
    }
    setIsLoading(false);
    if (newAlbums.albums.length === 0) {
      setHasMore(false);
    }
  };

  const a: AlbumPartial[] = albums.albums;

  console.log("albums", albums);

  return (
    <div className="flex flex-col w-screen md:w-full">
      <div className="flex flex-row align-center rounded-lg mt-2 md:px-2 mr-4 md:mr-2">
        <div className="flex flex-row mx-4 bg-slate-800 rounded-lg md:max-w-sm flex-1">
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
            { label: "Album", value: "album" },
            { label: "Year", value: "year" },
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
      <div className="grid grid-cols-[repeat(auto-fit,_minmax(250px,_1fr))] w-full lg:w-auto lg:gap-4 gap-2 my-4 mx-2 md:mx-4 px-2 transition-all duration-300">
        {albums.albums.map((album, i) => (
          <div
            key={album.id}
            ref={i === a.length - 1 ? lastAlbumRef : null}
            className="group flex flex-row lg:flex-col gap-2 lg:gap-0 justify-start place-content-start hover:bg-slate-800 border-gray-300 rounded-lg shadow-none hover:shadow-md transition-all duration-300"
          >
            <div className="relative w-full h-full max-w-16 lg:max-w-full md:group-hover:translate-y-[0.25rem] aspect-square md:group-hover:scale-[0.95] transition-all duration-700">
              <Link
                href={"/album/" + album.id}
                key={album.id}
                className="block margin-auto aspect-square max-w-16 lg:max-w-full w-full h-full"
              >
                <img
                  className="mx-auto max-w-16 md:max-w-full max-h-full self-center contain-content rounded-lg margin-auto shadow-xl group-hover:shadow-slate-950 transition-all duration-700"
                  src={
                    album.art.length > 0
                      ? album.art[0]
                      : "https://i.imgur.com/moGByde.jpeg"
                  }
                />
              </Link>
              <div className="hidden md:block absolute text-aoi-400 hover:text-aoi-300 opacity-0 group-hover:opacity-100 text-7xl -bottom-2 transition-all duration-300 drop-shadow-lg">
                <PlayAlbumButtonOnAction
                  album={album}
                  context={"album/" + album.id}
                />
              </div>
            </div>
            <Link
              href={"/album/" + album.id}
              className="flex flex-row justify-start items-center flex-1 pl-4 md:pl-0 py-1 md:group-hover:scale-[0.98] md:group-hover:translate-y-[0.1rem] transition-all duration-300 ease-in-out"
            >
              <div className="group-hover:ml-2 md:group-hover:scale-[0.98] md:group-hover:translate-y-[-0.25rem] transition-all duration-700 ease-in-out">
                <div className="line-clamp-1">{album.name}</div>
                <div className="text-slate-400">{album.artist.name}</div>
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
      <div className="mb-4">
        {isLoading && (
          <div className="">
            <Lottie
              options={{
                animationData: loadingAnimation,
              }}
              width="4rem"
            />
          </div>
        )}
        {!hasMore && (
          <div className="text-center py-4 text-sm text-neutral-500">
            That's all, folks!
          </div>
        )}
      </div>
    </div>
  );
}
