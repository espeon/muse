"use client";
import { useQueueStore } from "@/stores/queueStore";
import { useState, useEffect } from "react";
import { JLF } from "./types";
import BasicLyrics from "./basicLyrics";
import RichLyrics from "./richLyrics";
import { useTitleStore } from "@/stores/titleStore";
import { useConfig } from "@/stores/configStore";

// TODO: get the umi base URL from config via API or smth
export default function Lyrics() {
  const [lyrics, setLyrics] = useState<JLF | null>();
  const [lyricsError, setLyricsError] = useState<string | null>(null);
  const { currentTrack } = useQueueStore();
  const { setPageTitleVisible, setPageTitle } = useTitleStore();
  const { umiBaseURL } = useConfig();
  useEffect(() => {
    if (currentTrack !== null) {
      console.log("lyrics", currentTrack);
      setLyricsError(null);
      console.log("Fetching lyrics from ",         umiBaseURL + "/lyrics?track=" +
        encodeURIComponent(currentTrack.title) +
        "&artist=" +
        encodeURIComponent(currentTrack.artist) +
        "&album=" +
        encodeURIComponent(currentTrack.album))
      fetch(
        umiBaseURL + "/lyrics?track=" +
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
        .catch((e) => {
          setLyricsError("smth happened yuh");
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
      {lyricsError ? <div className="text-4xl">{lyricsError}</div> : null}
    </div>
  );
}