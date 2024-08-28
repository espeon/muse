"use client";

import PlayButton from "@/components/playButton";
import s2t from "@/helpers/s2t";
import {
  Context,
  ContextType,
  Track,
  useQueueStore,
} from "@/stores/queueStore";
import { Album, Track as AlbumTrack } from "@/types/album";
import { PiDotsThreeVerticalBold } from "react-icons/pi";
import { TbHeartFilled, TbHeart } from "react-icons/tb";

import Lottie from "react-lottie";
import anim from "@/lotties/audio.json";
import { usePlayerStore } from "@/stores/playerStore";
import { albumTrackToTrack } from "@/helpers/albumTrackToTrack";
import { TouchEvent, useState } from "react";
import Link from "next/link";
import { useConfig } from "@/stores/configStore";

const LottieOptions = {
  loop: true,
  autoplay: true,
  animationData: anim,
  rendererSettings: {
    preserveAspectRatio: "xMidYMid slice",
  },
};

export function SongEach({
  t,
  album,
  track,
}: {
  t: AlbumTrack;
  album: Album;
  track: Track;
}) {
  const { makiExternalBaseURL } = useConfig();
  const { playTrack, currentTrack } = useQueueStore();
  const { isPlaying } = usePlayerStore();
  // generate context on load
  const [context, setContext] = useState(
    genContextFromAlbum(album, makiExternalBaseURL),
  );
  const [lastTouch, setLastTouch] =
    useState<TouchEvent<HTMLTableRowElement> | null>(null);

  const handlePlay = (track: Track) => {
    console.log("Play track", track);
    playTrack(track);
    // set context for queue

    let tracks = album.tracks.map((track) =>
      albumTrackToTrack(album, track, makiExternalBaseURL),
    );
    // find the current track
    let i = tracks.findIndex((ftrack) => ftrack.stream === track.stream);
    // get rid of everything before and including the current track
    let cTracks = tracks.slice(i + 1, tracks.length);
    const pastTracks = tracks.slice(0, i);
    useQueueStore.getState().setContext({
      type: ContextType.Album,
      id: String(album.id),
      tracks: cTracks,
      display: album.name,
    });
    for (let i = 0; i < pastTracks.length; ++i) {
      useQueueStore.getState().addTrackPast(pastTracks[i]);
    }
    console.log("Set context");
  };
  const handleTouchEventStart = (e: TouchEvent<HTMLTableRowElement>) => {
    e.preventDefault();
    console.log(e.targetTouches);
    setLastTouch(e);
  };
  const handleTouchEventEnd = (
    e: TouchEvent<HTMLTableRowElement>,
    f: (track: Track) => void,
    a: Track,
  ) => {
    e.preventDefault();
    console.log(e.changedTouches);
    if (lastTouch === null) {
      return;
    }
    if (
      lastTouch.targetTouches[0].clientX > e.changedTouches[0].clientX - 10 &&
      lastTouch.targetTouches[0].clientX < e.changedTouches[0].clientX + 10
    ) {
      console.log(
        "x",
        lastTouch.targetTouches[0].clientX,
        e.changedTouches[0].clientX,
      );
      if (
        lastTouch.targetTouches[0].clientY > e.changedTouches[0].clientY - 10 &&
        lastTouch.targetTouches[0].clientY < e.changedTouches[0].clientY + 10
      ) {
        console.log(
          "y",
          lastTouch.targetTouches[0].clientY,
          e.changedTouches[0].clientY,
        );
        f(a);
      }
    }
    setLastTouch(null);
  };
  return (
    <tr
      key={t.id}
      className={`group rounded-md transition-all duration-75 ${
        currentTrack?.stream === track.stream
          ? "text-pink-400"
          : "text-slate-200"
      }`}
      onDoubleClick={() => handlePlay(track)}
      onTouchStart={(e) => handleTouchEventStart(e)}
      onTouchEnd={(e) => handleTouchEventEnd(e, handlePlay, track)}
    >
      <td className="text-right pl-6 pr-6 py-5 group-hover:bg-slate-800 rounded-l transition-all duration-75">
        <div className="relative mb-5 text-right w-5 -ml-2.5 font-mono font-thin">
          {currentTrack?.stream === track.stream && !isPlaying ? (
            <div className="absolute text-lg transition-all duration-75">
              <Lottie options={LottieOptions} height={25} width={25} />
            </div>
          ) : (
            <>
              <div className="absolute group-hover:opacity-0 transition-all duration-75 w-full">
                {t.number}
              </div>
              <div className="absolute group-hover:opacity-100 hover:text-slate-200 opacity-0 text-lg transition-all duration-75 mt-0.5 ml-1">
                <PlayButton track={track} ctx={context} />
              </div>
            </>
          )}
        </div>
      </td>
      <td className="text-left flex-col group-hover:bg-slate-800 transition-all duration-75">
        <div className="line-clamp-1">{t.name}</div>
        <div className="text-slate-400 text-sm group-hover:bg-slate-800 transition-all duration-75">
          <Link href={`/artist/${t.album_artist}`}>
            {t.artists.find((a) => a.id === t.album_artist)?.name}
          </Link>
          {t.artists
            .filter((a) => a.id !== t.album_artist)
            .sort((a, b) => a.name.localeCompare(b.name))
            .map((a) => (
              <span key={a.id}>
                ,{" "}
                {a.num_albums ? (
                  a.num_albums > 1 ? (
                    <Link key={a.id} href={`/artist/${a.id}`}>
                      {a.name}
                    </Link>
                  ) : (
                    a.name
                  )
                ) : (
                  a.name
                )}
              </span>
            ))}
        </div>
      </td>
      <td className="hidden md:table-cell w-24 max-w-full text-left group-hover:bg-slate-800 transition-all duration-75">
        {t.plays}
      </td>
      <td className="hidden md:table-cell text-right text-slate-900 group-hover:text-slate-500 w-16 max-w-full transition-all duration-75 group-hover:bg-slate-800">
        {t.liked ? (
          <TbHeartFilled className="text-pink-500 hover:text-slate-100" />
        ) : (
          <div className="group/heart relative mb-4">
            <TbHeartFilled className="absolute text-pink-500/50 group-hover/heart:opacity-100 opacity-0 transition-all duration-75" />
            <TbHeart className="absolute hover:text-slate-100 opacity-100 group-hover/heart:opacity-0" />
          </div>
        )}
      </td>
      <td className="text-right font-mono text-sm pr-2 w-16 max-w-full group-hover:bg-slate-800 transition-all duration-75">
        {s2t(t.duration)}
      </td>
      <td className="text-right w-3 pr-3 group-hover:bg-slate-800 duration-75 rounded-r group-hover:opacity-100 opacity-100 md:opacity-0 text-lg">
        <PiDotsThreeVerticalBold />
      </td>
    </tr>
  );
}

function genContextFromAlbum(a: Album, makiBaseURL: string): Context {
  let tracks = a.tracks.map((track) =>
    albumTrackToTrack(a, track, makiBaseURL),
  );
  return { type: ContextType.Album, id: String(a.id), tracks: tracks };
}
