import NavControls from "@/components/navControls";
import SetNavTitle from "@/components/helpers/setNavTitle";
import { ArtistPartials } from "@/types/artistPartial";
import ArtistFilterView, { FilterProps } from "./artistFilterView";
import { cookies } from "next/headers";

async function getArtistData({
  ...props
}: FilterProps): Promise<ArtistPartials> {
  "use server";
  const res = await fetch(
    (process.env.INTERNAL_MAKI_BASE_URL ?? "http://localhost:3031") +
      `/api/v1/artist?limit=${props.limit}&sortby=${props.sortby}&dir=${props.direction}${props.cursor ? `&cursor=${props.cursor}` : ""}${props.filter ? `&filter=${props.filter}` : ""}`,
  );
  if (!res.ok) {
    throw new Error(res.statusText + ": " + (await res.text()));
  }

  return res.json();
}

export default async function AlbumPage({
  searchParams,
}: {
  searchParams: Promise<{ [key: string]: string | string[] | undefined }>;
}) {
  // for dynamic rendering
  const _ = cookies();

  let queries = await searchParams;

  const a = await getArtistData({
    limit: queries["limit"] ? parseInt(queries["limit"][0] ?? "20") : 20,
    sortby: queries["sortby"] ? (queries["sortby"][0] ?? "artist") : "artist",
    direction: queries["direction"]
      ? (queries["direction"][0] ?? "asc")
      : "asc",
    //cursor: queries["cursor"] ? parseInt(queries["cursor"][0] ?? "0") : 0,
    filter: queries["filter"] ? (queries["filter"][0] ?? undefined) : undefined,
  });
  return (
    <>
      <div className="flex flex-col min-w-32 mx-4 md:mx-12 mt-16">
        <div className="text-4xl lg:text-4xl xl:text-6xl font-semibold transition-all duration-700">
          Artists
        </div>
        <SetNavTitle title="Artists" />
      </div>
      <ArtistFilterView
        initialArtists={a}
        initialFilter={
          queries["filter"]
            ? queries["filter"].length > 0
              ? queries["filter"][0]
              : (queries["filter"] as string)
            : undefined
        }
        fetchMoreArtists={getArtistData}
      />
    </>
  );
}
