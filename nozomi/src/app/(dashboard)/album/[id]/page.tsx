import s2t, { s2s } from "@/helpers/s2t";
import { Album } from "@/types/album";
import { ContextType } from "@/stores/queueStore";
import { PiClock } from "react-icons/pi";
import { TbHeart } from "react-icons/tb";
import { SongEach } from "@/components/songEach";
import AlbumActions from "./albumActions";
import { albumTrackToTrack } from "@/helpers/albumTrackToTrack";
import { IoDisc } from "react-icons/io5";
import SetNavTitle from "@/components/helpers/setNavTitle";
import Link from "next/link";
import React from "react";

// For ISR
export const dynamicParams = true;

async function getAlbumData(id: string): Promise<Album> {
  const res = await fetch(
    (process.env.INTERNAL_MAKI_BASE_URL ?? "http://localhost:3031") +
      "/api/v1/album/" +
      id,
  );

  if (!res.ok) {
    // This will activate the closest `error.js` Error Boundary
    throw new Error(res.statusText + ": " + (await res.text()));
  }

  return res.json();
}

export default async function AlbumPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = await params;
  let album = await getAlbumData(id);
  console.log(album.art);
  // get disc count
  let discArr = album.tracks
    .map((track) => track.disc)
    .filter((value, index, self) => self.indexOf(value) === index)
    .sort();
  let discs = discArr[discArr.length - 1];

  let external_url = process.env.EXTERNAL_MAKI_BASE_URL as string;

  console.log(
    "Generating album ctx: ",
    album.id,
    "with external url",
    external_url,
  );
  let tracks = album.tracks.map((track) => albumTrackToTrack(album, track));
  const context = {
    type: ContextType.Album,
    id: String(album.id),
    tracks: tracks,
  };

  return (
    <div className="flex flex-col place-items-center flex-1 w-full px-14 h-max min-h-max">
      <div className="flex flex-col md:flex-row gap-6 mt-14 md:mt-8 lg:mt-24 xl:mt-44 max-w-7xl w-full items-center md:items-end ">
        <div className="flex flex-col items-center aspect-square max-h-full h-full w-3/4 md:h-64 lg:h-48 xl:h-64 md:w-fit">
          <img
            className="max-w-full h-fit self-center rounded-xl ambilight transition-all duration-700 ring-2 ring-slate-500/10"
            src={
              album.art.length > 0
                ? album.art[0]
                : "https://i.imgur.com/moGByde.jpeg"
            }
          />
        </div>
        <div className="flex flex-col min-w-32 w-full justify-start z-10">
          <div className="text-2xl md:text-4xl lg:text-5xl xl:text-6xl mb-0 md:mb-2 font-semibold transition-all duration-700">
            {album.name}
          </div>
          <SetNavTitle title={album.name} />
          <div className="flex-col items-baseline">
            <Link
              href={`/artist/${album.artist.id}`}
              className="text-base md:text-lg lg:text-xl xl:text-4xl transition-all text-slate-400 hover:text-blue-400/75 duration-700"
            >
              {album.artist.name}
            </Link>
            <div className="text-sm transition-all text-slate-400 duration-700 mt-1 md:mt-2">
              {/* <span className="px-1.5 md:hidden inline">・</span> */}
              {album.year ? album.year + "・" : ""}
              {album.tracks.length} tracks
              {discs > 1 ? "・" + discs + " discs" : ""}・
              {s2s(
                album.tracks.map((t) => t.duration).reduce((a, b) => a + b, 0),
              )}
            </div>
          </div>
        </div>
      </div>
      <div className="flex flex-row max-w-7xl w-full my-4 text-slate-200">
        <AlbumActions album={album} />
      </div>
      <div className="flex flex-col max-w-7xl px-2 w-full">
        <table className="table-auto -mx-6">
          <thead>
            <tr className="content-start font-thin mx-2 text-slate-400 border-b border-gray-400/50">
              <th className="text-left font-mono font-thin pl-6 w-14 pb-2 ">
                #
              </th>
              <th className="text-left font-normal pb-2 ">Track</th>
              <th className="hidden md:table-cell text-left font-normal pb-2 w-8">
                Plays
              </th>
              <th className="hidden md:table-cell text-right font-normal w-10 pb-2 ">
                <TbHeart />
              </th>
              <th className="text-right font-normal flex flex-row justify-end align-baseline pr-2">
                <div className="flex-1 h-0" />
                <PiClock className="mt-1" />
              </th>
              <th className="text-right font-normal opacity-5">.</th>
            </tr>
          </thead>
          <tbody>
            {album.tracks.map((t, i) => {
              // get context for track
              const track = {
                title: t.name,
                artist: album.artist.name,
                album: album.name,
                artwork:
                  album.art.length > 0
                    ? album.art[0]
                    : "https://i.imgur.com/moGByde.jpeg",
                stream: t.id.toString(),
              };
              return (
                <React.Fragment key={t.id + t.disc + i}>
                  {discs != 1 && t.disc != album.tracks[i - 1]?.disc && (
                    <tr
                      className="text-slate-400 border-b border-gray-400/25 items-center"
                      key={t.id + t.disc + i + "pre"}
                    >
                      <td className="text-right text-xl flex flex-row justify-end align-baseline pr-5 py-3.5 pb-2">
                        <IoDisc />
                      </td>
                      <td className="text-left font-normal pt-2 ">
                        Disc {t.disc}
                      </td>
                      <td className="hidden md:table-cell text-gray-800/5">
                        .
                      </td>
                      <td className="hidden md:table-cell text-gray-800/5">
                        .
                      </td>
                      <td className="text-right font-mono text-sm flex flex-row justify-end -translate-y-1 pr-2">
                        {s2t(
                          album.tracks
                            .filter((o) => o.disc == t.disc)
                            .map((t) => t.duration)
                            .reduce((a, b) => a + b, 0),
                        )}
                      </td>
                    </tr>
                  )}
                  <SongEach
                    key={t.id + t.disc + i}
                    t={t}
                    album={album}
                    track={track}
                    context={context}
                  />
                </React.Fragment>
              );
            })}
          </tbody>
        </table>
        <div className="text-slate-400 text-base mb-8 mt-2">
          {album.year ? album.year + "・" : ""}
        </div>
      </div>
    </div>
  );
}
