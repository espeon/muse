"use server";
import SetNavTitle from "@/components/helpers/setNavTitle";
import { AlbumPartials } from "@/types/albumPartial";
import AlbumFilterView, { FilterProps } from "./albumFilterView";
import { cookies } from "next/headers";

async function getAlbumData(props: FilterProps): Promise<AlbumPartials> {
  "use server";
  console.log(process.env.INTERNAL_MAKI_BASE_URL);
  const res = await fetch(
    (process.env.INTERNAL_MAKI_BASE_URL ?? "http://localhost:3031") +
      `/api/v1/album?limit=${props.limit}&sortby=${props.sortby}&dir=${props.direction}${props.album_id ? `&cursor=${props.album_id}` : ""}${props.filter ? `&filter=${props.filter}` : ""}`,
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
  const albums = await getAlbumData({
    limit: queries["limit"] ? parseInt(queries["limit"][0] ?? "20") : 20,
    sortby: queries["sortby"] ? (queries["sortby"][0] ?? "artist") : "artist",
    direction: queries["direction"]
      ? (queries["direction"][0] ?? "asc")
      : "asc",
    //cursor: queries["cursor"] ? parseInt(queries["cursor"][0] ?? "0") : 0,
    filter: queries["filter"] ? (queries["filter"][0] ?? undefined) : undefined,
  });
  console.log(albums);
  return (
    <>
      <div className="flex flex-col min-w-32 mx-6 mt-16">
        <div className="text-4xl lg:text-4xl xl:text-6xl font-semibold transition-all duration-700">
          Albums
        </div>
        <SetNavTitle title="Albums" />
      </div>
      <AlbumFilterView
        initialAlbums={albums}
        initialFilter={
          queries["filter"]
            ? queries["filter"].length > 0
              ? queries["filter"][0]
              : (queries["filter"] as string)
            : undefined
        }
        fetchMoreAlbums={getAlbumData}
      />
    </>
  );
}
