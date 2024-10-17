"use server";
import SetNavTitle from "@/components/helpers/setNavTitle";
import { AlbumPartials } from "@/types/albumPartial";
import AlbumFilterView from "./albumFilterView";
import { cookies } from "next/headers";

async function getAlbumData(
  limit: number,
  sortby: "id" | "artist" | "album" | "year" | string,
  direction: "asc" | "desc" | string,
  cursor: number | null = 0,
): Promise<AlbumPartials> {
  "use server";
  console.log(process.env.INTERNAL_MAKI_BASE_URL);
  const res = await fetch(
    (process.env.INTERNAL_MAKI_BASE_URL ?? "http://localhost:3031") +
      `/api/v1/album?limit=${limit}&sortby=${sortby}&dir=${direction}${
        (cursor ?? 0 > 0) ? `&cursor=${cursor}` : ""
      }`,
  );
  if (!res.ok) {
    throw new Error(res.statusText + ": " + (await res.text()));
  }

  return res.json();
}

export default async function AlbumPage() {
  // for dynamic rendering
  const _ = cookies();
  const albums = await getAlbumData(20, "artist", "asc");
  return (
    <>
      <div className="flex flex-col min-w-32 mx-6 mt-16">
        <div className="text-4xl lg:text-4xl xl:text-6xl font-semibold transition-all duration-700">
          Albums
        </div>
        <SetNavTitle title="Albums" />
      </div>
      <AlbumFilterView initialAlbums={albums} fetchMoreAlbums={getAlbumData} />
    </>
  );
}
