"use client";
import { usePlayerStore } from "@/stores/playerStore";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { JLF, SyncedLines } from "./types";
import Ellipsis from "./ellipsis";
import getLyricStatus from "@/helpers/lyricStatus";
import LyricsMenu from "./lyricsMenu";
import { LyricText } from "@/stores/useLangAnalyzer";
import {
  JapaneseOptions,
  TranslitLanguage,
  useLyricsSettings,
} from "@/stores/lyricsSettingsStore";
import { useSmoothTimer } from "@/stores/useSmoothTimer";
import { mapRange } from "@/helpers/animath";

export default function BasicLyrics({
  lines,
  copyright,
}: {
  lines: SyncedLines | null;
  copyright: string | null;
}) {
  const { currentTime: globalCurrentTime, duration } = usePlayerStore();
  const { isPlaying, isBuffering } = usePlayerStore();
  const { richPreActiveRange, richPostActiveRange } = useLyricsSettings();
  const { currentTime } = useSmoothTimer({
    currentTime: globalCurrentTime,
    duration,
    isActivelyPlaying: !isPlaying && !isBuffering,
    onUpdate: (time) => {},
  });
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

  const getLyricStyles = useCallback(
    (
      segStatus: {
        secondsAfterActive: number;
        secondsBeforeActive: number;
        percentage: number;
      },
      seg: { timeEnd: number; timeStart: number },
      activeColor: string = "oklch(0.82 0.255 314.384)",
    ) => ({
      ["--lyric-seg-percentage" as any]: `${
        // set post-active range
        mapRange(
          segStatus.secondsAfterActive -
            (seg.timeEnd - seg.timeStart) * segStatus.percentage,
          0.1,
          richPostActiveRange ?? 0.3,
          100,
          0,
        ) *
        // set pre-active range
        mapRange(
          segStatus.secondsBeforeActive,
          0,
          richPreActiveRange ?? 1.3,
          1,
          0,
        )
      }%`,
      color: `color-mix(in sRGB, ${activeColor} var(--lyric-seg-percentage), rgb(209 213 219 / 0.65))`,
      filter: `drop-shadow(0 0px 4px ${activeColor.substring(0, -1)} / calc(var(--lyric-seg-percentage) * 0.35)))`,
    }),
    [richPostActiveRange, richPreActiveRange], // Include dependencies if needed
  );

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
            const segStatus = getLyricStatus(
              currentTime,
              line.time,
              lines.lines[i + 1]?.time ?? lines.linesEnd,
              -0.3,
            );

            const styles = useMemo(
              () =>
                getLyricStyles(segStatus, {
                  timeStart: line.time,
                  timeEnd: lines.lines[i + 1]?.time ?? lines.linesEnd,
                }),
              [
                segStatus,
                line.time,
                lines.lines[i + 1]?.time ?? lines.linesEnd,
                getLyricStyles,
              ],
            );

            return line.text ? (
              <div
                key={String(i) + line.text}
                className={`transition-all bg-transparent duration-1000 mb-2 md:mb-4 lg:mb-8 py-2 text-left origin-left text-3xl md:text-4xl lg:text-5xl xl:text-6xl ${segStatus.isActive ? "scale-100" : "scale-90"}`}
                style={styles}
              >
                <div
                  ref={segStatus.isActive ? activeLyricRef : null}
                  className={`md:top-0 top-60 h-4 w-4 absolute rounded-full`}
                />
                <LyricText text={line.text} lang={translit} jpOpts={jpOpts} />
              </div>
            ) : (
              <div
                key={String(i) + line.text}
                className={
                  segStatus.isActive
                    ? "mb-2 md:mb-4 transition-all bg-transparent duration-200"
                    : "transition-all bg-transparent duration-200"
                }
              >
                <div
                  ref={segStatus.isActive ? activeLyricRef : null}
                  className={`md:top-0 top-60 h-4 w-4 absolute rounded-full`}
                />
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
