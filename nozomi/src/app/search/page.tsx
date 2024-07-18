"use client";
import NavControls from "@/components/navControls";
import { Track as AlbumTrack } from "@/types/album";
import Link from "next/link";
import { useState } from "react";

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
