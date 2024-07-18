import SetNavTitle from "@/components/helpers/setNavTitle";
import NavControls from "@/components/navControls";
import PlayAlbumButtonOnAction from "@/components/playButtonOnAction";
import { Artist } from "@/types/artist";
import { cookies } from "next/headers";
import Link from "next/link";

async function getArtistData(id: string): Promise<Artist> {
  const res = await fetch(process.env.MAKI_BASE_URL  ?? "http://localhost:3031" + "/artist/" + id);
  if (!res.ok) {
    throw new Error(res.statusText + ": " + (await res.text()));
  }

  return res.json();
}

export default async function ArtistPage({
  params,
}: {
  params: { id: string };
}) {
    // for dynamic rendering
    const _ = cookies();
  const artist = await getArtistData(params.id);

  let bio_split = artist.bio.split('<a href="');
  let bio = bio_split[0];
  let link = bio_split[1].split('"')[0];
  if (bio.length > 1) {
    if (bio[bio.length - 1] == " ") {
      bio = bio.substring(0, bio.length - 1);
    }
    if (bio[bio.length - 1] == ".") {
      bio = bio.substring(0, bio.length - 1);
    }
    bio = bio + "...";
  }

  return (
    <div
      className="flex flex-col place-items-center justify-center content-center flex-1 w-full h-max min-h-max bg-black/5"
      id="main"
    >
      <NavControls />
      <div className="flex flex-col px-4 place-items-center flex-1 w-full h-max min-h-max">
        <div className="flex flex-col md:flex-row gap-6 mt-14 md:mt-8 lg:mt-24 xl:mt-44 max-w-7xl w-full items-center md:items-end ">
          <div className="flex flex-col items-center aspect-square max-h-full h-full w-3/4 md:h-64 lg:h-48 xl:h-64 md:w-fit">
            <img
              className="max-w-full aspect-square object-cover h-fit self-center rounded-full ambilight transition-all duration-700 ring-2 ring-slate-500/10"
              src={artist.picture}
            />
          </div>
          <div className="flex flex-col min-w-32 w-full justify-start z-10">
            <div className="text-2xl md:text-4xl lg:text-5xl xl:text-6xl mb-0 md:mb-2 font-semibold transition-all duration-700">
              {artist.name}
            </div>
            <SetNavTitle title={artist.name} />
            <div className="flex-col items-baseline">
              <div className="text-xs md:text-sm lg:text-lg xl:text-xl mb-1 md:mb-2 transition-all text-slate-400 duration-700">
                {artist.tags.split(",").join("・")}
              </div>
            </div>
          </div>
        </div>
        <div className="md:flex-row gap-6 mt-4 max-w-7xl w-full items-center md:items-end">
          <div className="line-clamp-5">
            {bio}{" "}
            <span className="text-blue-400">
              <Link href={link}>Read more on last.fm</Link>
            </span>
          </div>
          <div className="text-2xl md:text-2xl lg:text-3xl xl:text-4xl mb-0 md:mb-2 mt-8 font-medium transition-all duration-700">
            Albums
          </div>
          <div className="grid grid-cols-[repeat(auto-fit,_minmax(250px,_1fr))] md:gap-4 my-4 transition-all duration-300">
            {artist.albums.map((album) => (
              <div
                key={album.id}
                className="group flex flex-row md:flex-col justify-start place-content-start hover:bg-slate-800 border-gray-300 rounded-lg p-2 shadow-none hover:shadow-md hover:scale-[1.01] transition-all duration-300"
              >
                <div className="relative w-full h-full max-w-16 md:max-w-full aspect-square">
                  <Link
                    href={"/album/" + album.id}
                    key={album.id}
                    className="block margin-auto aspect-square max-w-16 md:max-w-full w-full h-full"
                  >
                    <img
                      className="mx-auto max-w-16 md:max-w-full max-h-full self-center aspect-square object-cover contain-content rounded-lg margin-auto shadow-xl group-hover:shadow-slate-950 hover:scale-[0.98] transition-all duration-700"
                      src={
                        album.art.length > 0
                          ? `${process.env.MAKI_BASE_URL}/art/${album.art[0]}`
                          : "https://i.imgur.com/moGByde.jpeg"
                      }
                    />
                  </Link>
                  <div className="hidden md:block absolute text-pink-600 hover:text-pink-500 opacity-0 group-hover:opacity-100 text-7xl -bottom-2 transition-all duration-300 drop-shadow-lg">
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
            {artist.albums.length < 4
              ? Array.from({ length: 4 - artist.albums.length }).map((_, i) => (
                  <div key={i} className="hidden md:block w-full h-64"></div>
                ))
              : null}
          </div>
        </div>
      </div>
    </div>
  );
}
