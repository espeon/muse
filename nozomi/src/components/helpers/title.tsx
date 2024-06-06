"use client"
import { usePlayerStore } from "@/stores/playerStore";
import { useQueueStore } from "@/stores/queueStore";
import { useEffect } from "react";

export default function Title() {
  const { currentTrack } = useQueueStore();
  const {isPlaying} = usePlayerStore();
  useEffect(() => {
    document.title = `${currentTrack?.artist ? ` ${isPlaying ? "⏸️" : "▶️"} ${currentTrack?.title} - ${currentTrack?.artist}` : "muse audio"}`
  }, [currentTrack, isPlaying]);

  return (
    <title>
      {currentTrack?.artist ? ` ${isPlaying ? "▶️" : "⏸️"} ${currentTrack?.title} - ${currentTrack?.artist}` : ""}
    </title>
  );
}
