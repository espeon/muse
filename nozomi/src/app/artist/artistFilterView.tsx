"use client";
import PlayAlbumButtonOnAction from "@/components/playButtonOnAction";
import { ArtistPartial } from "@/types/artistPartial";
import Link from "next/link";
import { useState } from "react";
import { BiSearch } from "react-icons/bi";

export default function AlbumFilterView({
  artists,
}: {
  artists: ArtistPartial[];
}) {
  // TODO: use the search endpoint
  const [searchQuery, setSearchQuery] = useState("");

  const a = artists.filter((a) => {
    if (searchQuery === "") {
      return true;
    }
    return a.name.toLowerCase().includes(searchQuery.toLowerCase());
  });
  return (
    <div>
      <div className="flex flex-row min-w-3 md:w-1/2 lg:1/3 xl:1/4 2xl:w-1/5 mx-4 lg:mx-6 my-4 md:mx-12 text-black rounded-lg bg-slate-800 focus-within:bg-slate-200 focus-within:ring-2 ring-pink-500 transition-all duration-300">
        <BiSearch className="mx-2 my-2" />
        <input
          className="rounded-lg w-full bg-transparent outline-none"
          type="text"
          placeholder="Find an artist..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
        />
      </div>
      <div className="grid grid-cols-[repeat(auto-fit,_minmax(250px,_1fr))] md:gap-8 my-4 mx-2 md:mx-10">
        {a.map((artist) => (
          <div
            key={artist.id}
            className="group flex flex-col w-full rounded-md hover:bg-slate-700"
          >
            <div className="relative h-full max-w-16 md:max-w-full mx-2 mt-2">
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
                <div className="text-slate-400">{artist.num_albums} album{artist.num_albums > 1 ? "s" : ""}</div>
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
