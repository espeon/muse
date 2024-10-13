"use client";
import PlayAlbumButtonOnAction from "@/components/playButtonOnAction";
import { useConfig } from "@/stores/configStore";
import { AlbumPartials } from "@/types/albumPartial";
import Link from "next/link";
import { useState } from "react";
import { BiSearch } from "react-icons/bi";

export default function AlbumFilterView({ albums }: { albums: AlbumPartials }) {
  // TODO: use the search endpoint
  const [searchQuery, setSearchQuery] = useState("");

  const filteredAlbums = albums.albums.filter((album) => {
    if (searchQuery === "") {
      return true;
    }
    return (
      album.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
      album.artist.name.toLowerCase().includes(searchQuery.toLowerCase())
    );
  });
  return (
    <div className="flex flex-col">
      <div className="flex flex-row min-w-3 lg:max-w-xl xl:max-w-xl w-full mx-4 lg:mx-6 my-4 md:mx-12 text-black rounded-lg bg-slate-800 focus-within:bg-slate-200 focus-within:ring-2 ring-aoi-500 transition-all duration-300">
        <BiSearch className="mx-2 my-2" />
        <input
          className="rounded-lg w-full bg-transparent outline-none"
          type="text"
          placeholder="Find an album..."
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
        />
      </div>
      <div className="grid grid-cols-[repeat(auto-fit,_minmax(250px,_1fr))] lg:gap-4 my-4 mx-auto md:mx-4 pl-2 transition-all duration-300">
        {filteredAlbums.map((album) => (
          <div
            key={album.id}
            className="group flex flex-row lg:flex-col gap-2 lg:gap-0 justify-start place-content-start hover:bg-slate-800 border-gray-300 rounded-lg p-2 shadow-none hover:shadow-md hover:scale-[1.01] transition-all duration-300"
          >
            <div className="relative w-full h-full max-w-16 lg:max-w-full aspect-square">
              <Link
                href={"/album/" + album.id}
                key={album.id}
                className="block margin-auto aspect-square max-w-16 lg:max-w-full w-full h-full"
              >
                <img
                  className="mx-auto max-w-16 md:max-w-full max-h-full self-center contain-content rounded-lg margin-auto shadow-xl group-hover:shadow-slate-950 hover:scale-[0.98] transition-all duration-700"
                  src={
                    album.art.length > 0
                      ? album.art[0]
                      : "https://i.imgur.com/moGByde.jpeg"
                  }
                />
              </Link>
              <div className="hidden md:block absolute text-aoi-400 hover:text-aoi-300 opacity-0 group-hover:opacity-100 text-7xl -bottom-2 transition-all duration-300 drop-shadow-lg">
                <PlayAlbumButtonOnAction album={album} />
              </div>
            </div>
            <Link
              href={"/album/" + album.id}
              className="flex flex-row justify-start items-center flex-1 pl-4 md:pl-0 py-1 group-hover:scale-[0.98] group-hover:translate-y-[0.1rem] transition-all duration-300 ease-in-out"
            >
              <div className="">
                <div className="line-clamp-1">{album.name}</div>
                <div className="text-slate-400">{album.artist.name}</div>
              </div>
            </Link>
          </div>
        ))}
        {filteredAlbums.length < 6
          ? Array.from({ length: 9 - filteredAlbums.length }).map((_, i) => (
              <div key={i} className="hidden md:block w-full h-64"></div>
            ))
          : null}
      </div>
    </div>
  );
}
