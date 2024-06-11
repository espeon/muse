"use client";
import { useQueueStore } from "@/stores/queueStore";
import { useState, useEffect } from "react";
import { JLF } from "./types";
import BasicLyrics from "./basicLyrics";
import RichLyrics from "./richLyrics";
import { useTitleStore } from "@/stores/titleStore";

export default function Lyrics() {
  const [lyrics, setLyrics] = useState<JLF | null>();
  const { currentTrack } = useQueueStore();
  const { pageTitleVisible, setPageTitleVisible, setPageTitle } = useTitleStore();
  useEffect(() => {
    if (currentTrack !== null) {
      console.log("lyrics", currentTrack);
      fetch(
        "http://localhost:3002/lyrics?track=" +
          encodeURIComponent(currentTrack.title) +
          "&artist=" +
          encodeURIComponent(currentTrack.artist) +
          "&album=" +
          encodeURIComponent(currentTrack.album)
      )
        .then((res) => res.json())
        .then((res) => {
          console.log("lyrics", res);
          setLyrics(res);
        })
        .catch(() => {
          setLyrics(null);
        });
    }
  }, [currentTrack]);

  // if width > 1280px
  useEffect(() => {
    if (typeof window !== "undefined" && window.innerWidth > 1280) {
      setPageTitleVisible(true);
    } else {
      setPageTitleVisible(false);
      setPageTitle("Lyrics");
    }
  }, [typeof window !== "undefined" && window?.innerWidth]);

  return (
    <div className="flex flex-col flex-1 text-center items-center justify-center bg-gray-900/5">
      {lyrics ? (
        lyrics.richsync ? (
          <RichLyrics rich={lyrics.richsync} copyright={lyrics.metadata?.Copyright ?? null} />
        ) : (
          <BasicLyrics lines={lyrics.lines} />
        )
      ) : (
        <div className="text-4xl">No lyrics found</div>
      )}
    </div>
  );
}