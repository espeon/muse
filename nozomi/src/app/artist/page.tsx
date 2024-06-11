import NavControls from "@/components/navControls";
import SetNavTitle from "@/components/helpers/setNavTitle";
import { ArtistPartials } from "@/types/artistPartial";
import AlbumFilterView from "./artistFilterView";

async function getAlbumData(): Promise<ArtistPartials> {
  const res = await fetch(
    "http://localhost:3000/artist?limit=1000&sortby=artist&dir=asc"
  );
  if (!res.ok) {
    throw new Error(res.statusText + ": " + (await res.text()));
  }

  return res.json();
}

export default async function AlbumPage() {
  const a = await getAlbumData();
  return (
    <div className="flex flex-col w-full" id="main">
        <NavControls />
      <div className="flex flex-col min-w-32 mx-4 md:mx-12 mt-16">
        <div className="text-4xl lg:text-4xl xl:text-6xl font-semibold transition-all duration-700">
          Albums
        </div>
        <SetNavTitle title="Albums" />
      </div>
      <AlbumFilterView artists={a.artists} />
    </div>
  );
}
