import NavControls from "@/components/navControls";
import SetNavTitle from "@/components/helpers/setNavTitle";
import { ArtistPartials } from "@/types/artistPartial";
import ArtistFilterView from "./artistFilterView";
import { cookies } from "next/headers";

async function getArtistData(
  limit: number,
  sortby: "id" | "artist" | string,
  direction: "asc" | "desc" | string,
  cursor: number | null,
): Promise<ArtistPartials> {
  "use server";
  const res = await fetch(
    (process.env.INTERNAL_MAKI_BASE_URL ?? "http://localhost:3031") +
      `/api/v1/artist?limit=${limit}&sortby=${sortby}&dir=${direction}${
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
  const a = await getArtistData(16, "artist", "asc", null);
  return (
    <>
      <div className="flex flex-col min-w-32 mx-4 md:mx-12 mt-16">
        <div className="text-4xl lg:text-4xl xl:text-6xl font-semibold transition-all duration-700">
          Artists
        </div>
        <SetNavTitle title="Artists" />
      </div>
      <ArtistFilterView initialArtists={a} fetchMoreArtists={getArtistData} />
    </>
  );
}
