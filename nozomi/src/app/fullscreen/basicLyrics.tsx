"use client";
import { usePlayerStore } from "@/stores/playerStore";
import { useEffect, useRef } from "react";
import { JLF, SyncedLines } from "./types";
import Ellipsis from "./ellipsis";

export default function BasicLyrics({ lines }: { lines: SyncedLines | null }) {
  const { currentTime } = usePlayerStore();
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
            return (
              <>
                <div
                  key={String(i) + line.text}
                  ref={isActive ? activeLyricRef : null}
                  className={`transition-all bg-transparent duration-200 mb-2 md:mb-4 lg:mb-8 py-2 text-left ${isActive ? "text-3xl md:text-4xl lg:text-5xl xl:text-6xl text-blue-300" : "text-2xl md:text-3xl lg:text-4xl xl:text-5xl"}`}
                >
                  {line.text || (
                    <Ellipsis
                      currentTime={currentTime}
                      start={line.time}
                      end={lines.lines[i + 1]?.time ?? lines.linesEnd}
                    />
                  )}
                </div>
              </>
            );
          })
        : null}
      <div className="h-[33vh]" />
    </div>
  );
}

function getLyricStatus(
  currentTime: number,
  lyricStart: number,
  lyricEnd: number,
) {
  // Check if the lyric is active
  let isActive = currentTime >= lyricStart && currentTime <= lyricEnd;

  // Initialize variables for percentage and elapsed seconds
  let percentage = 0;
  let secondsAfterActive = 0;

  if (isActive) {
    let duration = lyricEnd - lyricStart;
    secondsAfterActive = currentTime - lyricStart;
    percentage = (secondsAfterActive / duration) * 100;
  } else if (currentTime > lyricEnd) {
    secondsAfterActive = currentTime - lyricEnd;
  }

  return {
    isActive: isActive,
    percentage: Number(percentage.toFixed(2)),
    secondsAfterActive: secondsAfterActive,
  };
}
