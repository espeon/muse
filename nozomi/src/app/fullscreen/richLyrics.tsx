"use client";
import { useQueueStore } from "@/stores/queueStore";
import { usePlayerStore } from "@/stores/playerStore";
import { useState, useEffect, useRef } from "react";
import { JLF, SyncedLines, SyncedRich } from "./types";
import { easeInOutQuad, mapRange } from "@/helpers/animath";
import Ellipsis from "./ellipsis";
import { TbCircleChevronRight } from "react-icons/tb";
import { PiCaretLeft, PiCaretRight, PiCaretUp } from "react-icons/pi";
import LyricsMenu from "./lyricsMenu";

export default function RichLyrics({
  rich,
  copyright,
}: {
  rich: SyncedRich | null;
  copyright: string | null;
}) {
  const { currentTime } = usePlayerStore();
  const activeLyricRef = useRef<HTMLDivElement>(null);
  const [isScrolling, setIsScrolling] = useState(false);
  const scrollToTopRef = useRef<HTMLDivElement>(null);
  const [offset, setOffset] = useState(0);
  const [menuShown, setMenuShown] = useState(false);

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
    <div className="flex flex-col flex-1 w-full items-center">
      <div className="flex flex-col flex-1 px-4 xl:px-8 xl:max-w-[100rem] w-full">
        <div className="h-[43vh]" ref={scrollToTopRef} />
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
                  const {
                    isActive,
                    percentage,
                    secondsAfterActive,
                    secondsBeforeActive,
                  } = getLyricStatus(
                    currentTime,
                    line.timeStart,
                    line.timeEnd,
                    offset
                  );

                  let lyricPos =
                    line.agent == rich.agents[0].id ? "left" : "right";
                  if (
                    rich.agents.find((a) => a.id == line.agent)?.type == "group"
                  ) {
                    lyricPos = "center";
                  }
                  return (
                    <>
                      <div
                        key={i + j}
                        style={{
                          ["--lyric-line-dir" as any]: lyricPos,
                          textAlign: lyricPos as any,
                        }}
                        className={`transition-all bg-transparent duration-1000 ease-in-out mb-4 py-4 origin-[--lyric-line-dir]
                       ${
                         isActive
                           ? "text-gray-200/75 scale-100"
                           : "scale-90"
                       }`}
                        ref={
                          isActive ||
                          ((section.lines[j - 1]?.timeEnd < currentTime ??
                            false) &&
                            activeLyricRef.current != null)
                            ? activeLyricRef
                            : null
                        }
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
                            seg.timeEnd,
                            offset
                          );

                          return (
                            <span
                              key={i + j + k}
                              className={`transition-all bg-transparent duration-100 ease-in mb-4`}
                              style={{
                                ["--lyric-seg-percentage" as any]: `${
                                  mapRange(
                                    segStatus.secondsAfterActive -
                                      (seg.timeEnd - seg.timeStart),
                                    0.2,
                                    1,
                                    100,
                                    0
                                  ) *
                                  mapRange(
                                    segStatus.secondsBeforeActive,
                                    0,
                                    0.25,
                                    1,
                                    0
                                  )
                                }%`,
                                color: `color-mix(in sRGB, rgb(240 171 252) var(--lyric-seg-percentage), rgb(209 213 219 / 0.75))`,
                                filter:
                                  "drop-shadow(0 0px 4px rgba(249 168 212 / calc(var(--lyric-seg-percentage) * 0.35)))",
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
                            let spaceAfter =
                              currentLine[seg.text.length] === " ";
                            // remove the text
                            currentLine = currentLine.slice(
                              seg.text.length + (spaceAfter ? 1 : 0)
                            );

                            const segStatus = getLyricStatus(
                              currentTime,
                              seg.timeStart,
                              seg.timeEnd,
                              offset
                            );
                            return (
                              <span
                                key={i + j + k + "bgVox"}
                                className={`transition-all bg-transparent duration-100 ease-in mb-4 py-2`}
                              >
                                {seg.text}
                                {spaceAfter && " "}
                              </span>
                            );
                          })}
                      </div>
                      <div
                        key={i + j + "ellipsis"}
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
                          end={
                            section.lines[j + 1]?.timeStart ?? section.timeEnd
                          }
                        />
                      </div>
                    </>
                  );
                })}
              </div>
            ))}
          </div>
        ) : null}
        {copyright && (
          <div className="text-sm font-mono text-left text-gray-500/50">
            {copyright &&
              copyright.split("\n").map((line, i) => <p key={i}>{line}</p>)}
            <br />
            Timings may differ among releases, especially with fan-submitted
            lyrics.
          </div>
        )}
      </div>
      <div className="h-[33vh]" />
      <LyricsMenu
        offset={offset}
        setOffset={setOffset} />
    </div>
  );
}

function getLyricStatus(
  currentTime: number,
  lyricStart: number,
  lyricEnd: number,
  offset: number = 0
) {
  // default offset (animations look weird without this)
  offset = offset + 0.1;

  // add the offset to the current time
  currentTime = Number((currentTime + offset).toFixed(3))

  // Check if the lyric is active
  let isActive = currentTime > lyricStart && currentTime < lyricEnd;
  // Initialize variables for percentage and elapsed seconds
  let percentage = 0;
  let secondsAfterActive = 0;

  if (isActive) {
    let duration = lyricEnd - lyricStart;
    secondsAfterActive = currentTime - lyricStart;
    percentage = (secondsAfterActive / duration) * 100;
  } else if (currentTime > lyricEnd) {
    secondsAfterActive = currentTime  - lyricEnd;
  }

  return {
    isActive: isActive,
    percentage: Number(percentage.toFixed(2)),
    secondsAfterActive: secondsAfterActive,
    secondsBeforeActive: lyricStart - currentTime,
  };
}
