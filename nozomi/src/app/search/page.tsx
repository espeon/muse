"use client";
import NavControls from "@/components/navControls";
import { Track as AlbumTrack } from "@/types/album";
import Link from "next/link";
import { useState } from "react";

async function getAlbumData(id: string): Promise<AlbumTrack> {
    const res = await fetch(process.env.NEXT_PUBLIC_MAKI_BASE_URL + "/album/" + id);
    // The return value is *not* serialized
    // You can return Date, Map, Set, etc.
  
    if (!res.ok) {
      // This will activate the closest `error.js` Error Boundary
      throw new Error(res.statusText + ": " + (await res.text()));
    }
  
    return res.json();
  }

export default async function SearchPage({
  params,
}: {
  params: { query: string };
}) {
    const [res, setRes] = useState<AlbumTrack[]>([]);
  return (
    <div className="flex flex-col w-full" id="main">
        <NavControls />
      <div className="flex flex-col min-w-32 mx-4 md:mx-12 mt-16">
        <input></input>
      </div>
      <div className="grid grid-cols-[repeat(auto-fit,_minmax(250px,_1fr))] md:gap-4 my-4 mx-2 md:mx-12">
      </div>
    </div>
  );
}
