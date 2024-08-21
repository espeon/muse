import HomePageGreeting from "@/components/auth/homePageGreeting";
import Carousel, { CarouselPage } from "@/components/carousel/carousel";
import Controls from "@/components/controls";
import SetNavTitle from "@/components/helpers/setNavTitle";
import PlayAlbumButtonOnAction from "@/components/playButtonOnAction";
import { AlbumPartial } from "@/types/albumPartial";
import { Home } from "@/types/home";
import Link from "next/link";

async function getHomePageData(): Promise<Home> {
  const res = await fetch(
    (process.env.MAKI_BASE_URL ?? "http://localhost:3031") + "/home/",
  );
  if (!res.ok) {
    throw new Error(res.statusText + ": " + (await res.text()));
  }
  console.log(res);
  return res.json();
}

export default async function AlbumPage() {
  let home = await getHomePageData();
  return (
    <>
      <div className="flex flex-col min-w-32 mx-4 md:mx-12 mt-16">
        <div className="text-3xl lg:text-3xl xl:text-4xl font-semibold transition-all duration-700 mb-4">
          <HomePageGreeting />
        </div>
        <SetNavTitle title="Home" />
        <div className="text-lg lg:text-xl xl:text-2xl transition-all duration-700">
          Latest Albums
        </div>
        {/* Only show the first row of albums */}
        <Carousel options={{ slidesToScroll: 1, align: "start" }}>
          {home.latest_albums.map((album, i) => (
            <CarouselPage key={album.id}>
              <AlbumAlwaysVertical album={album} />
            </CarouselPage>
          ))}
        </Carousel>
      </div>
    </>
  );

  function AlbumAlwaysVertical({ album }: { album: AlbumPartial }) {
    return (
      <div
        key={album.id}
        className="group flex flex-col justify-start place-content-start hover:bg-slate-800 border-gray-300 rounded-lg p-1 md:p-2 shadow-none hover:shadow-md hover:scale-[1.01] transition-all duration-300"
      >
        <div className="relative w-full h-full max-w-full aspect-square">
          <Link
            href={"/album/" + album.id}
            className="pl-0 pt-1 group-hover:scale-[0.98] group-hover:translate-y-[0.1rem] transition-all duration-300 ease-in-out"
          >
            <img
              className="mx-auto max-w-full max-h-full self-center aspect-square object-cover contain-content rounded-lg margin-auto shadow-xl group-hover:shadow-slate-950 hover:scale-[0.98] transition-all duration-700"
              src={
                album.art.length > 0
                  ? album.art[0]
                  : "https://i.imgur.com/moGByde.jpeg"
              }
            />
          </Link>
          <div className="hidden md:block absolute text-pink-600 hover:text-pink-500 opacity-0 group-hover:opacity-100 text-6xl -bottom-2 transition-all duration-300 drop-shadow-lg">
            <PlayAlbumButtonOnAction album={album} />
          </div>
        </div>
        <Link
          href={"/album/" + album.id}
          className="pl-0 pt-1 group-hover:scale-[0.98] group-hover:translate-y-[0.1rem] transition-all duration-300 ease-in-out"
        >
          <div className="">
            <div className="line-clamp-2 text-sm md:text-lg">{album.name}</div>
            <div className="text-slate-400 text-sm line-clamp-1">
              {album.artist.name}
            </div>
          </div>
        </Link>
      </div>
    );
  }
}
