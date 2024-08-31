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
import getLyricStatus from "@/helpers/lyricStatus";
import {
  JapaneseOptions,
  TranslitLanguage,
} from "@/stores/lyricsSettingsStore";
import { LyricText } from "@/stores/useLangAnalyzer";

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
  const [jpOpts, setJpOpts] = useState(JapaneseOptions.FURIGANA_HIRAGANA);
  const [translit, setTranslit] = useState(TranslitLanguage.NONE);
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
      }, 2500);

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
                key={i + section.lines[0].text + "section"}
                className="text-3xl md:text-4xl lg:text-5xl xl:text-6xl text-gray-400"
              >
                {section.lines.map((line, j) => {
                  let currentLine = line.text;
                  let currentBGLine = line.bgVox?.text ?? "";
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
                    offset,
                  );

                  const bgStatus =
                    line.bgVox &&
                    getLyricStatus(
                      currentTime,
                      line.bgVox?.timeStart,
                      line.bgVox?.timeEnd,
                      offset,
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
                        key={i + j + line.text}
                        style={{
                          ["--lyric-line-dir" as any]: lyricPos,
                          textAlign: lyricPos as any,
                        }}
                        className={`transition-all bg-transparent duration-1000 ease-in-out mb-2 py-3 leading-tight origin-[--lyric-line-dir]
                       ${isActive ? "text-gray-200/75 scale-100" : "scale-90"}`}
                      >
                        <div
                          ref={
                            isActive ||
                            ((section.lines[j - 1]?.timeEnd < currentTime ??
                              false) &&
                              activeLyricRef.current != null)
                              ? activeLyricRef
                              : null
                          }
                          className={`md:top-32 top-36 h-4 w-4 absolute rounded-full transition-all duration-1000 ease-in-out`}
                        />
                        {line.segments.map((seg, k) => {
                          // check if there is a space after the text
                          let spaceAfter = currentLine[seg.text.length] === " ";
                          // remove the text
                          currentLine = currentLine.slice(
                            seg.text.length + (spaceAfter ? 1 : 0),
                          );

                          const segStatus = getLyricStatus(
                            currentTime,
                            seg.timeStart,
                            seg.timeEnd,
                            offset,
                          );

                          return (
                            <span
                              key={i + j + k + seg.text}
                              className={`transition-all bg-transparent duration-100 ease-in mb-4`}
                              style={{
                                ["--lyric-seg-percentage" as any]: `${
                                  mapRange(
                                    segStatus.secondsAfterActive -
                                      (seg.timeEnd - seg.timeStart),
                                    0.2,
                                    1,
                                    100,
                                    0,
                                  ) *
                                  mapRange(
                                    segStatus.secondsBeforeActive,
                                    0,
                                    0.25,
                                    1,
                                    0,
                                  )
                                }%`,
                                color: `color-mix(in sRGB, rgb(240 171 252) var(--lyric-seg-percentage), rgb(209 213 219 / 0.65))`,
                                filter:
                                  "drop-shadow(0 0px 4px rgba(249 168 212 / calc(var(--lyric-seg-percentage) * 0.35)))",
                              }}
                            >
                              <LyricText
                                text={seg.text}
                                lang={translit}
                                jpOpts={jpOpts}
                              />
                              {spaceAfter && " "}
                            </span>
                          );
                        })}
                        {line.bgVox ? (
                          <div
                            className={`transition-all bg-transparent duration-700 text-2xl md:text-3xl lg:text-4xl xl:text-5xl origin-[--lyric-line-dir] ${bgStatus?.isActive ? "text-gray-200/75 scale-100" : "scale-95"}`}
                          >
                            {line.bgVox.segments.map((seg, k) => {
                              // check if there is a space after the text
                              let spaceAfter =
                                currentBGLine[seg.text.length] === " ";
                              // remove the text
                              currentBGLine = currentBGLine.slice(
                                seg.text.length + (spaceAfter ? 1 : 0),
                              );
                              const segStatus = getLyricStatus(
                                currentTime,
                                seg.timeStart,
                                seg.timeEnd,
                                offset,
                              );
                              return (
                                <span
                                  key={i + j + k + "bgVox-seg" + seg.text}
                                  className={`transition-all bg-transparent duration-100 ease-in mb-4`}
                                  style={{
                                    ["--lyric-seg-percentage" as any]: `${
                                      mapRange(
                                        segStatus.secondsAfterActive -
                                          (seg.timeEnd - seg.timeStart),
                                        0.2,
                                        1,
                                        100,
                                        0,
                                      ) *
                                      mapRange(
                                        segStatus.secondsBeforeActive,
                                        0,
                                        0.25,
                                        1,
                                        0,
                                      )
                                    }%`,
                                    color: `color-mix(in sRGB, oklch(0.71 0.16 338.70) var(--lyric-seg-percentage), rgb(209 213 219 / 0.45))`,
                                    filter:
                                      "drop-shadow(0 0px 4px rgba(255 148 212 / calc(var(--lyric-seg-percentage) * 0.35)))",
                                  }}
                                >
                                  <LyricText
                                    text={seg.text}
                                    lang={translit}
                                    jpOpts={jpOpts}
                                  />
                                  {spaceAfter ? " " : ""}
                                </span>
                              );
                            })}
                          </div>
                        ) : null}
                      </div>
                      <div key={i + j + "ellipsis"}>
                        <div
                          ref={
                            currentTime >= line.timeStart &&
                            currentTime <=
                              section.lines[j + 1]?.timeStart - 2 &&
                            activeLyricRef.current === null
                              ? activeLyricRef
                              : null
                          }
                          className={`top-32 h-4 w-4 absolute rounded-full ${
                            currentTime >= line.timeStart &&
                            currentTime <=
                              section.lines[j + 1]?.timeStart - 2 &&
                            activeLyricRef.current === null
                              ? activeLyricRef
                              : null
                                ? "bg-pink-500"
                                : "bg-transparent"
                          } transition-all duration-1000 ease-in-out`}
                        />
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
                <div
                  key={i + "section-ellipsis"}
                  ref={
                    currentTime >= section.timeStart &&
                    currentTime <= rich.sections[i + 1]?.timeStart - 2 &&
                    activeLyricRef.current === null
                      ? activeLyricRef
                      : null
                  }
                >
                  <Ellipsis
                    currentTime={currentTime}
                    start={section.timeEnd}
                    end={
                      rich.sections[i + 1]?.lines[0]?.timeStart ??
                      rich.sections[i + 1]?.timeEnd
                    }
                  />
                </div>
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
        setOffset={setOffset}
        translit={translit}
        setTranslit={setTranslit}
        jpOpts={jpOpts}
        setJpOpts={setJpOpts}
      />
    </div>
  );
}
