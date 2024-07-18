"use client";
import { useConfig } from "@/stores/configStore";
import { ContextType, Track, useQueueStore } from "@/stores/queueStore";
import { Album } from "@/types/album";
import { MouseEvent } from "react";
import { PiPlayCircleFill, PiQueue } from "react-icons/pi";

export default function AlbumActions({ album }: { album: Album }) {
  const {makiBaseURL} = useConfig();
  const handlePlay = () => {
    let handledFirst = false;
    let tracks = [];
    for (const t in album.tracks) {
      let track: Track = {
        title: album.tracks[t].name,
        artist: album.artist.name,
        album: album.name,
        context: "album/" + album.id,
        artwork:
          album.art.length > 0
            ? `${makiBaseURL}/art/${album.art[0]}`
            : "https://i.imgur.com/moGByde.jpeg",
        stream: `${makiBaseURL}/track/${album.tracks[t].id}/stream`,
      };
      tracks.push(track);
      if (!handledFirst) {
        handledFirst = true;
        // clear queue
        useQueueStore.getState().clearQueue();
        useQueueStore.getState().playTrack(track);
      }
    }
    // set context for queue
    tracks.pop()
    useQueueStore.getState().setContext({ type: ContextType.Album, id: String(album.id), tracks: tracks, display: album.name });
  };

  const handleQueue = () => {
    let handledFirst = false;
    for (const t in album.tracks) {
      let track: Track = {
        title: album.tracks[t].name,
        artist: album.artist.name,
        album: album.name,
        context: "album/" + album.id,
        artwork:
          album.art.length > 0
            ? `${makiBaseURL}/art/${album.art[0]}`
            : "https://i.imgur.com/moGByde.jpeg",
        stream: `${makiBaseURL}/track/${album.tracks[t].id}/stream`,
      };
      useQueueStore.getState().addTrack(track);
    }
  };
  return (
    <div className="flex flex-row gap-4">
      <button
        className={`hover:text-pink-300 text-gray-300 transition-colors duration-300`}
        onClick={(e) => handlePlay()}
      >
        <PiPlayCircleFill className="text-5xl md:text-6xl drop-shadow-sm hover:drop-shadow-xl" />
      </button>
      <button
        className="hover:text-pink-500 text-gray-300 transition-colors duration-300"
        onClick={(e) => handleQueue()}
      >
        <PiQueue className="text-3xl md:text-4xl" />
      </button>
    </div>
  );
}
