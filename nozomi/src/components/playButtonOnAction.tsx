"use client";
import { albumTrackToTrack } from "@/helpers/albumTrackToTrack";
import { useConfig } from "@/stores/configStore";
import {
  Context,
  ContextType,
  Track,
  useQueueStore,
} from "@/stores/queueStore";
import { Track as AlbumTrack } from "@/types/album";
import { AlbumPartial } from "@/types/albumPartial";
import { PiPlayCircleFill, PiPlayFill } from "react-icons/pi";

export default function PlayAlbumButtonOnAction({
  album,
  children,
  ...props
}: {
  album: AlbumPartial;
  children?: React.ReactNode;
}) {
  const { externalMakiBaseURL } = useConfig();

  const handleGenerateAndPlay = (album: AlbumPartial) => {
    // fetch album
    fetch(externalMakiBaseURL + "/album/" + album.id)
      .then((res) => res.json())
      .then((data) => {
        // generate tracks
        const tracks = data.tracks.map((t: AlbumTrack) =>
          albumTrackToTrack(data, t, externalMakiBaseURL),
        );
        // play tracks
        useQueueStore.getState().clearQueue();
        useQueueStore.getState().playTrack(tracks[0]);
        tracks.shift();
        useQueueStore.getState().setContext({
          type: ContextType.Album,
          id: String(album.id),
          tracks: tracks,
          display: album.name,
        });
      });
  };

  return (
    <button
      onClick={() => {
        handleGenerateAndPlay(album);
      }}
      {...props}
    >
      {children ?? <PiPlayCircleFill className="z-10" />}
    </button>
  );
}
