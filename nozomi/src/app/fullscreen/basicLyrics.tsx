"use client";
import { usePlayerStore } from "@/stores/playerStore";
import { useEffect, useRef, useState } from "react";
import { JLF, SyncedLines } from "./types";
import Ellipsis from "./ellipsis";
import getLyricStatus from "@/helpers/lyricStatus";
import LyricsMenu from "./lyricsMenu";
import { LyricText } from "@/stores/useLangAnalyzer";
import {
  JapaneseOptions,
  TranslitLanguage,
} from "@/stores/lyricsSettingsStore";

export default function BasicLyrics({
  lines,
  copyright,
}: {
  lines: SyncedLines | null;
  copyright: string | null;
}) {
  const { currentTime } = usePlayerStore();
  const [offset, setOffset] = useState(0);
  const [jpOpts, setJpOpts] = useState(JapaneseOptions.FURIGANA_HIRAGANA);
  const [translit, setTranslit] = useState(TranslitLanguage.NONE);
  const activeLyricRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (activeLyricRef.current) {
      console.log("Scrolling into view");
      activeLyricRef.current.scrollIntoView({
        behavior: "smooth",
        block: "center",
      });
    }
  }, [activeLyricRef.current]);

  useEffect(() => {
    console.log("Changing lang or jpOpts");
  }, [translit, jpOpts]);

  return (
    <div className="flex flex-col flex-1 px-4 xl:px-8 xl:max-w-[100rem] w-full align-middle items-start">
      <div
        className="h-[33vh]"
        ref={
          (lines?.lines[0]?.time as number) > currentTime
            ? activeLyricRef
            : null
        }
      />
      {lines
        ? lines.lines.map((line, i) => {
            const { isActive, percentage, secondsAfterActive } = getLyricStatus(
              currentTime,
              line.time,
              lines.lines[i + 1]?.time ?? lines.linesEnd,
            );
            return line.text ? (
              <div
                key={String(i) + line.text}
                ref={isActive ? activeLyricRef : null}
                className={`transition-all bg-transparent duration-200 mb-2 md:mb-4 lg:mb-8 py-2 text-left origin-left text-3xl md:text-4xl lg:text-5xl xl:text-6xl ${isActive ? "scale-100 text-[rgb(240_171_252)]" : "scale-95 text-[rgb(209_213_219_/_0.65)]"} `}
              >
                <LyricText text={line.text} lang={translit} jpOpts={jpOpts} />
              </div>
            ) : (
              <div
                key={String(i) + line.text}
                ref={isActive ? activeLyricRef : null}
                className={
                  isActive
                    ? "mb-2 md:mb-4 transition-all bg-transparent duration-200"
                    : "transition-all bg-transparent duration-200"
                }
              >
                <Ellipsis
                  currentTime={currentTime}
                  start={line.time}
                  end={lines.lines[i + 1]?.time ?? lines.linesEnd}
                />
              </div>
            );
          })
        : null}
      {copyright && (
        <div className="text-sm font-mono text-left text-gray-500">
          {copyright &&
            copyright.split("\n").map((line, i) => <p key={i}>{line}</p>)}
          <br />
          Timings may differ among releases, especially with fan-submitted
          lyrics.
        </div>
      )}
      <div className="h-[33vh]" />
      <LyricsMenu
        offset={offset}
        setOffset={setOffset}
        translit={translit}
        setTranslit={setTranslit}
        jpOpts={jpOpts}
        setJpOpts={setJpOpts}
      />
    </div>
  );
}
