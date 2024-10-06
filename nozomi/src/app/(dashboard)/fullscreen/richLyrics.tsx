"use client";
import { useQueueStore } from "@/stores/queueStore";
import { usePlayerStore } from "@/stores/playerStore";
import { useState, useEffect, useRef, useMemo, useCallback } from "react";
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
  useLyricsSettings,
} from "@/stores/lyricsSettingsStore";
import { LyricText } from "@/stores/useLangAnalyzer";
import { useSmoothTimer } from "@/stores/useSmoothTimer";

export default function RichLyrics({
  rich,
  copyright,
}: {
  rich: SyncedRich | null;
  copyright: string | null;
}) {
  const { currentTime: globalCurrentTime, duration } = usePlayerStore();
  const { isPlaying, isBuffering } = usePlayerStore();
  const { richPreActiveRange, richPostActiveRange } = useLyricsSettings();
  const smt = useSmoothTimer({
    currentTime: globalCurrentTime,
    duration,
    isActivelyPlaying: !isPlaying && !isBuffering,
    onUpdate: (time) => {},
  });
  const activeLyricRef = useRef<HTMLDivElement>(null);
  const scrollToTopRef = useRef<HTMLDivElement>(null);

  const [lyricSettings, setLyricSettings] = useState({
    offset: 0,
    jpOpts: JapaneseOptions.FURIGANA_HIRAGANA,
    translit: TranslitLanguage.NONE,
    menuShown: false,
  });

  const { offset, jpOpts, translit } = lyricSettings;

  const memoizedRichSections = useMemo(() => {
    if (!rich) return [];
    return rich.sections.map((section) => ({
      ...section,
      lyricPos:
        rich.agents.find((a) => a.id === section.lines[0].agent)?.type ===
        "group"
          ? "center"
          : "left",
    }));
  }, [rich]);

  const scrollToActiveLyric = useCallback(() => {
    if (activeLyricRef.current) {
      activeLyricRef.current.scrollIntoView({
        behavior: "smooth",
        block: "center",
        inline: "center",
      });
    }
  }, []);

  useEffect(() => {
    console.log("activeLyricRef", activeLyricRef);
    scrollToActiveLyric();
  }, [scrollToActiveLyric, rich, activeLyricRef.current]);

  const handleScrollToTop = useCallback(() => {
    if (scrollToTopRef.current) {
      scrollToTopRef.current.scrollIntoView({
        behavior: "smooth",
        block: "center",
        inline: "center",
      });
    }
  }, []);

  useEffect(() => {
    handleScrollToTop();
  }, [handleScrollToTop, rich]);

  const updateLyricSettings = useCallback(
    (newSettings: Partial<typeof lyricSettings>) => {
      setLyricSettings((prev) => ({ ...prev, ...newSettings }));
    },
    [],
  );

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
          richPostActiveRange ?? 2.1,
          100,
          0,
        ) *
        // set pre-active range
        mapRange(
          segStatus.secondsBeforeActive,
          0,
          richPreActiveRange ?? 0.3,
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
    <div className="flex flex-col flex-1 w-full items-center">
      <div className="flex flex-col flex-1 px-4 xl:px-8 xl:max-w-[100rem] w-full">
        <div className="h-[43vh]" ref={scrollToTopRef} />
        {rich ? (
          <div className="flex flex-col">
            {memoizedRichSections.map((section, i) => (
              <div
                key={i + section.lines[0].text + "section"}
                className="text-3xl md:text-4xl lg:text-5xl xl:text-6xl text-gray-400"
              >
                {section.lines.map((line, j) => {
                  const { isActive, percentage } = getLyricStatus(
                    smt.currentTime,
                    line.timeStart,
                    line.timeEnd,
                    offset,
                  );

                  let currentLine = line.text;
                  let bgline = line.bgVox?.text;

                  const bgStatus =
                    line.bgVox &&
                    getLyricStatus(
                      smt.currentTime,
                      line.bgVox?.timeStart,
                      line.bgVox?.timeEnd,
                      offset,
                    );

                  return (
                    <div
                      key={i + j + line.text}
                      style={{
                        ["--lyric-line-dir" as any]: section.lyricPos,
                        textAlign: section.lyricPos as any,
                      }}
                      className={`transition-all bg-transparent duration-1000 ease-in-out mb-2 py-3 leading-tight origin-[--lyric-line-dir]
                        ${isActive ? "text-gray-200/75 scale-100" : "scale-90"}`}
                    >
                      <div
                        ref={
                          isActive ||
                          (section.lines[j - 1]?.timeEnd < smt.currentTime &&
                            activeLyricRef.current != null)
                            ? activeLyricRef
                            : null
                        }
                        className="md:top-0 top-36 h-4 w-4 absolute rounded-full transition-all duration-1000 ease-in-out"
                      />
                      {line.segments.map((seg, k) => {
                        const segStatus = getLyricStatus(
                          smt.currentTime,
                          seg.timeStart,
                          seg.timeEnd,
                          offset,
                        );

                        // check if there is a space after the text
                        let spaceAfter = currentLine[seg.text.length] === " ";
                        // remove the text
                        currentLine = currentLine.slice(
                          seg.text.length + (spaceAfter ? 1 : 0),
                        );

                        const styles = useMemo(
                          () => getLyricStyles(segStatus, seg),
                          [segStatus, seg, getLyricStyles],
                        );

                        return (
                          <span
                            key={i + j + k + seg.text}
                            className={`transition-all bg-transparent duration-100 ease-in mb-4`}
                            style={styles}
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
                      {line.bgVox && (
                        <div
                          className={`transition-all bg-transparent duration-700 text-2xl md:text-3xl lg:text-4xl xl:text-5xl origin-[--lyric-line-dir] ${
                            bgStatus?.isActive
                              ? "text-gray-200/75 scale-100"
                              : "scale-95"
                          }`}
                        >
                          {line.bgVox.segments.map((seg, k) => {
                            const segStatus = getLyricStatus(
                              smt.currentTime,
                              seg.timeStart,
                              seg.timeEnd,
                              offset,
                            );

                            // check if there is a space after the text
                            let spaceAfter =
                              bgline && bgline[seg.text.length] === " ";
                            // remove the text
                            bgline =
                              bgline &&
                              bgline.slice(
                                seg.text.length + (spaceAfter ? 1 : 0),
                              );

                            const styles = useMemo(
                              () =>
                                getLyricStyles(
                                  segStatus,
                                  seg,
                                  "oklch(0.82 0.255 350.94)",
                                ),
                              [segStatus, seg, getLyricStyles],
                            );

                            return (
                              <span
                                key={i + j + k + "bgVox-seg" + seg.text}
                                className={`transition-all bg-transparent duration-100 ease-in mb-4`}
                                style={styles}
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
                        </div>
                      )}
                    </div>
                  );
                })}
                <Ellipsis
                  currentTime={smt.currentTime}
                  start={section.timeEnd}
                  end={
                    rich.sections[i + 1]?.lines[0]?.timeStart ??
                    rich.sections[i + 1]?.timeEnd
                  }
                />
              </div>
            ))}
          </div>
        ) : null}
        {copyright && (
          <div className="text-sm font-mono text-left text-gray-500/50">
            {copyright.split("\n").map((line, i) => (
              <p key={i}>{line}</p>
            ))}
            <br />
            Timings may differ among releases, especially with fan-submitted
            lyrics.
          </div>
        )}
      </div>
      <div className="h-[33vh]" />
      <LyricsMenu
        offset={offset}
        setOffset={(newOffset) => updateLyricSettings({ offset: newOffset })}
        translit={translit}
        setTranslit={(newTranslit) =>
          updateLyricSettings({ translit: newTranslit })
        }
        jpOpts={jpOpts}
        setJpOpts={(newJpOpts) => updateLyricSettings({ jpOpts: newJpOpts })}
      />
    </div>
  );
}
