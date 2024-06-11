"use client";
import { useQueueStore } from "@/stores/queueStore";
import { usePlayerStore } from "@/stores/playerStore";
import {
  PiCaretLineLeft,
  PiCaretLineRight,
  PiPlayCircleFill,
  PiPauseCircleFill,
  PiShuffle,
  PiRepeat,
  PiSpeakerHifi,
  PiSpeakerHigh,
  PiSpeakerLow,
  PiSpeakerNone,
  PiPlayFill,
  PiPauseFill,
  PiMicrophoneStageDuotone,
} from "react-icons/pi";
import {
  ChangeEvent,
  DragEvent,
  MouseEvent,
  SyntheticEvent,
  useEffect,
  useState,
} from "react";
import React from "react";
import { TbHeart } from "react-icons/tb";
import s2t from "@/helpers/s2t";
import Ambilight from "@/helpers/ambilight";
import { IoIosPause, IoIosPlay, IoIosRedo, IoIosRepeat, IoIosShuffle } from "react-icons/io";
import { BottomSheet } from "react-spring-bottom-sheet";
import { useMobileSheetStore } from "@/stores/mobileSheetStore";
import { BiVolume } from "react-icons/bi";
import { useRouter } from "next/navigation";

export default function MobileControls() {
  const [currentLocalTime, setCurrentLocalTime] = useState(0);
  const { currentTrack, popPastTrack, popTrack } = useQueueStore();
  const { toggle, isOpen } = useMobileSheetStore();
  const {
    setCurrentTime,
    currentTime,
    duration,
    isPlaying,
    togglePlaying,
    setSeeking,
    isSeeking,
    volume,
    setVolume,
    muted,
    setMuted,
  } = usePlayerStore();
  const [lastVolume, setLastVolume] = useState(volume);
  const PlayPauseIcon = isPlaying ? IoIosPlay : IoIosPause;

  // sync local time with player but only when not seeking
  useEffect(() => {
    if (!isSeeking) {
      setCurrentLocalTime(currentTime);
    }
  }, [currentTime, isSeeking]);

  const handleToggleSheet = (e: any) => {
    console.log("Toggling sheet");
    toggle();
  };

  return (
    <div className="h-16 w-full flex flex-col justify-center">
      <div className="h-[3.9375rem] w-full flex justify-center py-1">
        <Ambilight />
        <div
          className="flex flex-row justify-start items-center flex-1 pl-3 py-1"
          onTouchEnd={handleToggleSheet}
        >
          <div className="block margin-auto aspect-square max-w-full h-full mr-4">
            <img
              src={currentTrack?.artwork ?? "https://i.imgur.com/moGByde.jpeg"}
              className="mx-auto max-h-full self-center contain-content rounded-lg margin-auto ambilight ring-1 ring-slate-500/25"
            />
          </div>
          <div className="">
            <div className="line-clamp-1">{currentTrack?.title}</div>
            <div className="line-clamp-1 text-slate-300">
              {currentTrack?.artist}
            </div>
          </div>
        </div>
        <div className="flex flex-row justify-end items-center pr-4 py-1">
          <PlayPauseIcon
            className="ml-2 h-8 w-8 hover:text-gray-300 transition-colors duration-300"
            onClick={() => togglePlaying()}
          />
        </div>
      </div>
      <progress
        value={currentLocalTime}
        max={duration}
        className="w-full h-[0.0625rem]"
      />
    </div>
  );
}
export function MobileSheetControls() {
  const { isOpen, toggle } = useMobileSheetStore();
  const { currentTrack, popPastTrack, popTrack } = useQueueStore();
  const {
    setCurrentTime,
    currentTime,
    duration,
    isPlaying,
    togglePlaying,
    setSeeking,
    isSeeking,
    volume,
    setVolume,
    muted,
    setMuted,
  } = usePlayerStore();

  const PlayPauseIcon = isPlaying ? IoIosPlay : IoIosPause;

  const [currentLocalTime, setCurrentLocalTime] = useState(0);

  const router = useRouter();

  const handleSeekChange = (c: ChangeEvent<HTMLInputElement>) => {
    let value = c.currentTarget.value;
    setCurrentLocalTime(Number(value) / 100);
  };

  const handleSeekDrag = (e: any) => {
    const action = e._reactName;
    if (action === "onMouseDown" || action === "onTouchStart") {
      setSeeking(true);
    }
    if (action === "onMouseUp" || action === "onTouchEnd") {
      setSeeking(false);
      setCurrentTime(currentLocalTime, true);
    }
  };

  const handleNavigateToLyrics = () => {
    toggle();
    router.push("/fullscreen");
  };

  useEffect(() => {
    if (!isSeeking) {
      setCurrentLocalTime(currentTime);
    }
  }, [currentTime, isSeeking]);

  return (
    <BottomSheet open={isOpen} onDismiss={toggle}>
      <div className="h-[95vh] w-full flex flex-col items-center text-white bg-slate-950 shadow-2xl shadow-gray-300 rounded-t-2xl overflow-auto">
        <div className="flex flex-col md:flex-row mt-16 md:mt-8 lg:mt-24 xl:mt-44 max-w-7xl w-full items-center md:items-end ">
          <div className="flex flex-col items-center max-h-full mx-8 md:h-64 lg:h-48 xl:h-64 md:w-fit">
            <img
              className="max-w-full h-fit self-center rounded-xl ambilight transition-all duration-700 ring-2 ring-slate-500/10"
              src={currentTrack?.artwork ?? "https://i.imgur.com/moGByde.jpeg"}
            />
          </div>
          <div className="flex-1 w-full flex flex-col items-start z-10 px-8 pt-8">
            <div className="text-3xl">{currentTrack?.title}</div>
            <div className="text-lg">{currentTrack?.artist}</div>
          </div>

          <div className="mt-2 w-full px-8">
            <input
              type="range"
              min="0"
              max={duration * 100}
              value={currentLocalTime * 100}
              className="range w-full md:max-w-sm xl:max-w-prose max-w-sm cursor-pointer"
              onChange={(c) => handleSeekChange(c)}
              onTouchStart={(e) => handleSeekDrag(e)}
              onTouchEnd={(e) => handleSeekDrag(e)}
            />
            <div className="flex items-start justify-between w-full">
              <div
                className={`font-mono text-xs ${
                  isSeeking ? "text-slate-300 scale-105" : "text-slate-400"
                } transition-all duration-150`}
              >
                {s2t(currentLocalTime)}
              </div>
              <div className="font-mono text-xs text-slate-400">
                {s2t(duration)}
              </div>
            </div>

        <div className="flex flex-row justify-between items-center pb-2 mt-12">
          <PiShuffle className="h-7 w-7" />
          <PiCaretLineLeft
            className="h-9 w-9  hover:text-gray-300 transition-colors duration-300"
            onClick={() => {
              if (currentLocalTime < 3) {
                popPastTrack();
              } else {
                setCurrentTime(0, true);
              }
            }}
          />
          <PlayPauseIcon
            className="h-14 w-14 hover:text-gray-300 transition-colors duration-300"
            onClick={() => togglePlaying()}
          />
          <PiCaretLineRight
            className="h-9 w-9 hover:text-gray-300 transition-colors duration-300"
            onClick={() => popTrack()}
          />
          <IoIosRepeat className="h-10 w-8 -mx-1" />
        </div>
          </div>
        </div>
        <div className="flex-1" />
        <div className="flex flex-row justify-between items-center w-full mb-8 px-8">
          <div className="flex flex-row items-center gap-2 w-full">
          <div className="flex-1" />
            <PiMicrophoneStageDuotone className="h-6 w-6" onClick={handleNavigateToLyrics} />
          </div>
        </div>
      </div>
    </BottomSheet>
  );
}
