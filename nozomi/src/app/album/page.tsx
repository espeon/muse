import NavControls from "@/components/navControls";
import PlayButton from "@/components/playButton";
import PlayAlbumButtonOnAction from "@/components/playButtonOnAction";
import SetNavTitle from "@/components/helpers/setNavTitle";
import { AlbumPartial, AlbumPartials } from "@/types/albumPartial";
import Link from "next/link";
import { PiPlayCircle, PiPlayCircleFill } from "react-icons/pi";
import AlbumFilterView from "./albumFilterView";

async function getAlbumData(): Promise<AlbumPartials> {
  const res = await fetch(
    "http://localhost:3000/album?limit=1000&sortby=artist&dir=asc"
  );
  if (!res.ok) {
    throw new Error(res.statusText + ": " + (await res.text()));
  }

  return res.json();
}

export default async function AlbumPage() {
  const albums = await getAlbumData();
  return (
    <div className="flex flex-col w-full" id="main">
        <NavControls />
      <div className="flex flex-col min-w-32 mx-4 md:mx-12 mt-16">
        <div className="text-4xl lg:text-4xl xl:text-6xl font-semibold transition-all duration-700">
          Albums
        </div>
        <SetNavTitle title="Albums" />
      </div>
      <AlbumFilterView albums={albums} />
    </div>
  );
}
