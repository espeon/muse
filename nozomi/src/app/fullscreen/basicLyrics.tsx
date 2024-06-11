"use client";
import { useQueueStore } from "@/stores/queueStore";
import { usePlayerStore } from "@/stores/playerStore";
import { useState, useEffect, useRef } from "react";
import { JLF, SyncedLines } from "./types";
import { clamp } from "@/helpers/animath";

export default function BasicLyrics({lines}: {lines: SyncedLines | null}) {
  const { currentTime } = usePlayerStore();
  const activeLyricRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (activeLyricRef.current) {
        console.log("Scrolling into view")
      activeLyricRef.current.scrollIntoView({
        behavior: "smooth",
        block: "center",
      });
    }
  }, [activeLyricRef.current]);

  return (
    <div className="flex flex-col flex-1 text-center items-center justify-center">
        <div className="h-[33vh]" />
      {lines ? (
        lines.lines.map((line, i) => {
            const {isActive, percentage, secondsAfterActive} = getLyricStatus(currentTime, line.time, lines.lines[i+1]?.time ?? lines.linesEnd);
          return (
            <div
              key={String(i) + line.text}
              ref={isActive ? activeLyricRef : null}
              className={`transition-all bg-transparent duration-200 mb-4 py-2 ${isActive ? "text-5xl" : "text-4xl"}`}
              style={{
                //top: `${(line.time - currentTime) * 100}%`,
                backgroundImage: `linear-gradient(to right, rgba(255, 255, 255, ${isActive ? 0.5 : clamp((secondsAfterActive * 3) - 0.5, 0.5, 1)}) ${1.25 * percentage}%, rgba(255, 255, 255, ${isActive ? 0.9 : secondsAfterActive === 0 ? 1 : clamp((secondsAfterActive * 3) - 0.5, 0.5, 1)}) ${1.5 * percentage}%)`,
                backgroundClip: "text",
                WebkitBackgroundClip: "text",
                color: "transparent",
                WebkitTextFillColor: "transparent",
              }}
            >
              {line.text}
            </div>
          );
        })
      ) : null}
      <div className="h-[33vh]" />
    </div>
  );
}

function getLyricStatus(currentTime: number, lyricStart: number, lyricEnd: number) {
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
        secondsAfterActive: secondsAfterActive
    };
}