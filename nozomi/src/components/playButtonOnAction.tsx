"use client";
import { albumTrackToTrack } from "@/helpers/albumTrackToTrack";
import { useConfig } from "@/stores/configStore";
import { usePlayerStore } from "@/stores/playerStore";
import {
  Context,
  ContextType,
  Track,
  useQueueStore,
} from "@/stores/queueStore";
import { Track as AlbumTrack } from "@/types/album";
import { AlbumPartial } from "@/types/albumPartial";
import {
  PiPauseCircleFill,
  PiPlayCircleFill,
  PiPlayFill,
} from "react-icons/pi";

export default function PlayContextButtonOnAction({
  album,
  children,
  context,
  ...props
}: {
  album: AlbumPartial;
  children?: React.ReactNode;
  context?: string;
}) {
  const { externalMakiBaseURL } = useConfig();
  const { currentContext, currentTrack } = useQueueStore();
  const { isPlaying, togglePlaying } = usePlayerStore();

  const handleGenerateAndPlay = (album: AlbumPartial) => {
    if (
      currentContext?.type === ContextType.Album &&
      currentContext?.id === String(album.id)
    )
      togglePlaying();
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
      {children ?? (
        <>
          {currentContext?.type === context?.split("/")[0] &&
          currentContext?.id === context?.split("/")[1] &&
          !isPlaying ? (
            <PiPauseCircleFill className="z-10" />
          ) : (
            <PiPlayCircleFill className="z-10" />
          )}
        </>
      )}
    </button>
  );
}
