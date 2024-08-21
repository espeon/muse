"use client";
import Link from "next/link";
import { useState } from "react";
import { LuMenu } from "react-icons/lu";
import { PiHouseBold, PiMagnifyingGlassBold } from "react-icons/pi";
import { RiAccountBoxLine, RiAlbumLine, RiSettings3Line } from "react-icons/ri";

const menuItemClass =
  "text-slate-400 text-base flex items-center mx-2 my-4 p-2 hover:bg-slate-800 rounded-md gap-2 transition-all duration-300";

export default function Menu() {
  const [open, setOpen] = useState(false);

  return (
    <>
      <div
        className={`hidden md:block relative w-full h-full pt-2 rounded-md ${
          open ? "min-w-40" : "min-w-8"
        } transition-all duration-150`}
      >
        <button
          className="text-slate-400 text-base flex items-center mx-2 p-2 hover:bg-slate-800 rounded-md gap-2 transition-all duration-300"
          onClick={() => setOpen(!open)}
        >
          <LuMenu className="h-8 w-8" />
        </button>
        <div className="mx-1">
          <Link href="/" className={menuItemClass}>
            {" "}
            <PiHouseBold className="h-6 w-6" /> {open ? "Home" : ""}{" "}
          </Link>
          <Link href="/search" className={menuItemClass}>
            {" "}
            <PiMagnifyingGlassBold className="h-6 w-6" /> {open ? "Search" : ""}
          </Link>
          <Link href="/album" className={menuItemClass}>
            {" "}
            <RiAlbumLine className="h-6 w-6" /> {open ? "Albums" : ""}
          </Link>
          <Link href="/artist" className={menuItemClass}>
            {" "}
            <RiAccountBoxLine className="h-6 w-6" /> {open ? "Artists" : ""}
          </Link>
          <div className={menuItemClass}>
            {" "}
            <RiSettings3Line className="h-6 w-6" /> {open ? "Settings" : ""}
          </div>
        </div>
      </div>
      <div className="sticky top-4 h-4 md:h-8 flex flex-row md:hidden justify-between items-center my-4 md:mx-4">
        <div className="flex flex-row justify-evenly w-full">
          <Link href="/" className={menuItemClass}>
            <PiHouseBold className="h-8 w-8" />
          </Link>
          <Link href="/search" className={menuItemClass}>
            <PiMagnifyingGlassBold className="h-8 w-8" />
          </Link>
          <Link href="/album" className={menuItemClass}>
            <RiAlbumLine className="h-8 w-8" />
          </Link>
          <Link href="/artist" className={menuItemClass}>
            <RiAccountBoxLine className="h-9 w-9 -m-1" />
          </Link>
          <div className={menuItemClass}>
            <RiSettings3Line className="h-8 w-8" />
          </div>
        </div>
      </div>
    </>
  );
}
