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
import { IoIosPause, IoIosPlay } from "react-icons/io";

export default function MobileControls() {
  const [currentLocalTime, setCurrentLocalTime] = useState(0);
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
  const [lastVolume, setLastVolume] = useState(volume);
  const PlayPauseIcon = isPlaying ? IoIosPlay : IoIosPause;

  // sync local time with player but only when not seeking
  useEffect(() => {
    if (!isSeeking) {
      setCurrentLocalTime(currentTime);
    }
  }, [currentTime, isSeeking]);

  return (
    <div className="h-16 w-full flex flex-col justify-center">
      <div className="h-[3.9375rem] w-full flex justify-center py-1">
        <Ambilight />
        <div className="flex flex-row justify-start items-center flex-1 pl-3 py-1">
          <div className="block margin-auto aspect-square max-w-full h-full mr-4">
            <img
              src={currentTrack?.artwork ?? "https://i.imgur.com/moGByde.jpeg"}
              className="mx-auto max-h-full self-center contain-content rounded-lg margin-auto ambilight ring-1 ring-slate-500/25"
            />
          </div>
          <div className="">
            <div className="line-clamp-1">{currentTrack?.title}</div>
            <div className="line-clamp-1 text-slate-300">{currentTrack?.artist}</div>
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
  return (<></>)
}