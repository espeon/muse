"use client";
import { useQueueStore } from "@/stores/queueStore";
import { usePlayerStore } from "@/stores/playerStore";
import { useState, useEffect, useRef } from "react";
import { JLF, SyncedLines, SyncedRich } from "./types";
import { easeInOutQuad, mapRange } from "@/helpers/animath";
import Ellipsis from "./ellipsis";

export default function RichLyrics({ rich, copyright }: { rich: SyncedRich | null, copyright: string | null }) {
  const { currentTime } = usePlayerStore();
  const activeLyricRef = useRef<HTMLDivElement>(null);
  const [isScrolling, setIsScrolling] = useState(false);
  const scrollToTopRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (activeLyricRef.current && !isScrolling) {
      console.log("Scrolling into view at", currentTime.toFixed(2));
      setIsScrolling(true);

      //   activeLyricRef.current.addEventListener("scroll", (e) => {
      //     console.log("Scroll end");
      //     setIsScrolling(false);
      //   });

      // Add a timer to set the isScrolling state to false
      setTimeout(() => {
        setIsScrolling(false);
      }, 1500);

      activeLyricRef.current.scrollIntoView({
        behavior: "smooth",
        block: "center",
        inline: "center",
      });
    }
  }, [activeLyricRef.current, rich]);

  // set the noItemRef to true if there is no item and has not been for at least 5 seconds
  useEffect(() => {
    // scroll to top
    if (scrollToTopRef.current && !isScrolling) {
      setTimeout(() => {
        setIsScrolling(false);
      }, 1500);

      scrollToTopRef.current.scrollIntoView({
        behavior: "smooth",
        block: "center",
        inline: "center",
      });
    }
  }, [rich]);

  return (
    <div className="flex flex-col flex-1 px-4 xl:px-8 xl:max-w-[100rem] w-full">
      <div className="h-[10vh]" ref={scrollToTopRef} />
      {rich ? (
        <div className="flex flex-col">
          {rich.sections.map((section, i) => (
            <div
              key={i}
              className="text-3xl md:text-4xl lg:text-5xl xl:text-6xl text-gray-400"
            >
              {section.lines.map((line, j) => {
                let currentLine = line.text;
                // is the current line up?
                const { isActive, percentage, secondsAfterActive, secondsBeforeActive } =
                  getLyricStatus(
                    currentTime,
                    line.timeStart,
                    currentTime >= line.timeStart &&
                      currentTime <= section.lines[j + 1]?.timeStart
                      ? line.timeEnd
                      : section.lines[j + 1]?.timeStart - 0.3
                  );

                let lyricPos =
                  line.agent == rich.agents[0].id ? "left" : "right";
                if (
                  rich.agents.find((a) => a.id == line.agent)?.type == "group"
                ) {
                  lyricPos = "center";
                }
                let textLoc = lyricPos;
                return (
                  <>
                    <div
                      key={i + j}
                      style={{
                        ["--lyric-line-dir" as any]: lyricPos,
                        textAlign: lyricPos as any
                      }}
                      className={`transition-all bg-transparent duration-1000 ease-in-out mb-4 py-4 origin-[--lyric-line-dir]
                       ${
                         isActive ||
                         (secondsBeforeActive < 0.4 && secondsAfterActive < 0.1)
                           ? "text-gray-200/75 scale-100"
                           : "scale-90"
                       }`}
                      ref={isActive || ((section.lines[j - 1]?.timeEnd < currentTime ?? false) && activeLyricRef.current != null) ? activeLyricRef : null}
                    >
                      {line.segments.map((seg, k) => {
                        // check if there is a space after the text
                        let spaceAfter = currentLine[seg.text.length] === " ";
                        // remove the text
                        currentLine = currentLine.slice(
                          seg.text.length + (spaceAfter ? 1 : 0)
                        );

                        const segStatus = getLyricStatus(
                          currentTime,
                          seg.timeStart,
                          seg.timeEnd
                        );

                        return (
                          <span
                            key={i + j + k}
                            className={`transition-all bg-transparent duration-100 ease-in mb-4 ${
                              (segStatus.secondsBeforeActive < 0.4 &&
                                segStatus.secondsAfterActive < 0.4) ||
                              segStatus.isActive
                                ? "scale-105"
                                : ""
                            }`}
                            style={{
                              ["--lyric-seg-percentage" as any]: `${mapRange(segStatus.secondsAfterActive - (seg.timeEnd - seg.timeStart), 0.2, 1, 100, 0) * mapRange(segStatus.secondsBeforeActive, 0, 0.25, 1, 0) }%`,
                              color: `color-mix(in sRGB, rgb(240 171 252) var(--lyric-seg-percentage), rgb(209 213 219 / 0.75))`,
                              filter: "drop-shadow(0 0px 8px rgba(249 168 212 / calc(var(--lyric-seg-percentage) * 0.35)))",
                            }}
                          >
                            {seg.text}
                            {spaceAfter && " "}
                          </span>
                        );
                      })}
                      {line.bgVox &&
                        line.bgVox.segments.map((seg, k) => {
                          // check if there is a space after the text
                          let spaceAfter = currentLine[seg.text.length] === " ";
                          // remove the text
                          currentLine = currentLine.slice(
                            seg.text.length + (spaceAfter ? 1 : 0)
                          );

                          const segStatus = getLyricStatus(
                            currentTime,
                            seg.timeStart,
                            seg.timeEnd
                          );
                          return (
                            <span
                              key={i + j + k + "bgVox"}
                              className={`transition-all bg-transparent duration-100 ease-in mb-4 py-2 ${
                                (segStatus.secondsAfterActive != 0 &&
                                  segStatus.secondsAfterActive < 0.3) ||
                                segStatus.isActive
                                  ? "text-blue-300 scale-105"
                                  : ""
                              }`}
                            >
                              {seg.text}
                              {spaceAfter && " "}
                            </span>
                          );
                        })}
                    </div>
                    <div
                      ref={
                        currentTime >= line.timeStart &&
                        currentTime <= section.lines[j + 1]?.timeStart - 2 &&
                        activeLyricRef.current === null
                          ? activeLyricRef
                          : null
                      }
                    >
                      <Ellipsis
                        currentTime={currentTime}
                        start={line.timeEnd}
                        end={section.lines[j + 1]?.timeStart ?? section.timeEnd}
                      />
                    </div>
                  </>
                );
              })}
            </div>
          ))}
        </div>
      ) : null}
      {copyright && <div className="text-sm font-mono text-left text-gray-500/50">{copyright && copyright.split("\n").map((line, i) => <p key={i}>{line}</p>)}</div>}
    </div>
  );
}

function getLyricStatus(
  currentTime: number,
  lyricStart: number,
  lyricEnd: number,
  offset: number = -0.3
) {
  // Check if the lyric is active
  let isActive =
    currentTime + offset >= lyricStart && currentTime + offset <= lyricEnd;

  // Initialize variables for percentage and elapsed seconds
  let percentage = 0;
  let secondsAfterActive = 0;

  if (isActive) {
    let duration = lyricEnd - lyricStart;
    secondsAfterActive = currentTime + offset - lyricStart;
    percentage = (secondsAfterActive / duration) * 100;
  } else if (currentTime + offset > lyricEnd) {
    secondsAfterActive = currentTime + offset - lyricEnd;
  }

  return {
    isActive: isActive,
    percentage: Number(percentage.toFixed(2)),
    secondsAfterActive: secondsAfterActive,
    secondsBeforeActive: lyricStart - (currentTime + offset),
  };
}
