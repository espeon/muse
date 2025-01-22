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
import { TbHeart, TbMaximize } from "react-icons/tb";
import s2t from "@/helpers/s2t";
import Ambilight from "@/helpers/ambilight";
import Link from "next/link";
import SeekBar from "./seekBar";
import * as Slider from "@radix-ui/react-slider";
import { ScrollingText } from "./scrollText";

export default function Controls() {
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
    isBuffering,
  } = usePlayerStore();
  const [lastVolume, setLastVolume] = useState(volume);
  const PlayPauseIcon = isPlaying ? PiPlayCircleFill : PiPauseCircleFill;

  const handleVolumeChange = (c: number, commit: boolean = false) => {
    let value = c;
    if (commit) {
      setLastVolume(value);
    }
    if (value == 0) {
      setMuted(true);
    } else {
      setMuted(false);
    }
    setVolume(Number(value) / 100);
  };

  const handleMute = () => {
    setMuted(!muted);
  };

  const handleSeekDrag = (e: number[], commit: boolean) => {
    if (commit) {
      setSeeking(false);
      setCurrentTime(currentLocalTime, true);
    } else {
      setCurrentLocalTime(e[0] / 100);
      setSeeking(true);
    }
  };

  // sync local time with player but only when not seeking
  useEffect(() => {
    if (!isSeeking) {
      setCurrentLocalTime(currentTime);
    }
  }, [currentTime, isSeeking]);

  // smoothify current local time
  useEffect(() => {
    if (!isSeeking) {
      setCurrentLocalTime(currentLocalTime);
    }
  }, [currentLocalTime, isSeeking]);

  return (
    <div className="h-20 w-full flex flex-rows justify-center">
      <Ambilight />
      <div className="flex flex-row justify-start items-center flex-1 pl-4 py-1 w-1/4">
        {currentTrack && (
          <>
            <div className="block margin-auto aspect-square max-w-full h-full mr-4">
              <img
                src={
                  currentTrack?.artwork ?? "https://i.imgur.com/moGByde.jpeg"
                }
                className="mx-auto max-h-full self-center contain-content rounded-lg margin-auto ambilight z-20"
              />
            </div>
            <div className="w-3/5 max-w-min">
              <ScrollingText text={currentTrack?.title} />
              <div className="line-clamp-1 w-max">{currentTrack?.artist}</div>
            </div>
            <TbHeart className="h-6 w-6 ml-4" />
          </>
        )}
      </div>
      <div className="flex flex-col justify-center align-baseline items-center">
        <div className="flex flex-row gap-4 pb-2">
          <PiShuffle className="h-5 w-5 mt-2.5" />
          <PiCaretLineLeft
            className="h-6 w-6 mt-2 hover:text-wisteria-400 transition-colors duration-150"
            onClick={() => {
              if (currentLocalTime < 3) {
                popPastTrack();
              } else {
                setCurrentTime(0, true);
              }
            }}
          />
          <button onClick={() => togglePlaying()}>
            <div className="relative grid overflow-hidden rounded-full mt-[0.0625rem] shadow-[0_1000px_0_0_hsl(0_0%_20%)_inset] transition-colors duration-200">
              <span>
                <span
                  className={`${isBuffering ? "animate-spinslow before:animate-kitrotate before:bg-[conic-gradient(from_0deg,rgb(2_6_23)_0_340deg,aliceblue_360deg)]" : "bg-slate-950"} absolute inset-0 h-[100%] w-[100%] overflow-hidden rounded-full before:absolute before:aspect-square before:w-[200%] before:rotate-[-90deg] before:content-[''] before:[translate:-50%_-15%]`}
                />
              </span>
              <span className=" absolute inset-px rounded-full bg-slate-950 transition-colors duration-200" />
              <PlayPauseIcon
                className={`h-10 w-10 z-10 -m-[0.0625rem] text-wisteria-200 hover:text-wisteria-400 transition-colors duration-150 ${isBuffering && "text-slate-400"}`}
              />
            </div>
          </button>
          <PiCaretLineRight
            className="h-6 w-6 mt-2 hover:text-wisteria-400 transition-colors duration-150"
            onClick={() => popTrack().switchToNextTrack()}
          />
          <PiRepeat className="h-5 w-5 mt-2.5" />
        </div>
        <div className="flex flex-row gap-2 items-center">
          <div
            className={`font-mono text-xs ${
              isSeeking ? "text-slate-300 scale-105" : "text-slate-400"
            } transition-all duration-150`}
          >
            {s2t(currentLocalTime)}
          </div>
          <SeekBar
            duration={duration}
            currentTime={currentLocalTime}
            onSeek={(time: number) => setCurrentLocalTime(time)}
            onValueChange={(value: number[]) => handleSeekDrag(value, false)}
            onValueCommit={(value: number[]) => handleSeekDrag(value, true)}
            isActivelyPlaying={!isPlaying && !isBuffering}
            className="range w-screen lg:max-w-xs xl:max-w-lg 2xl:max-w-prose cursor-pointer"
          />
          <div className="font-mono text-xs text-slate-400">
            {s2t(duration)}
          </div>
        </div>
      </div>
      <div className="flex flex-row justify-center items-center flex-1 mr-4">
        <div className="flex-1"></div>
        <Link href="/fullscreen">
          <PiMicrophoneStageDuotone className="h-6 w-6 mr-4 hover:text-pink-400 transition-colors duration-150" />
        </Link>
        <div onClick={() => handleMute()}>
          {muted || volume === 0 ? (
            <PiSpeakerNone className="h-6 w-6 mr-2" />
          ) : volume > 0.5 ? (
            <PiSpeakerHigh className="h-6 w-6 mr-2" />
          ) : (
            <PiSpeakerLow className="h-6 w-6 mr-2" />
          )}
        </div>
        <Slider.Root
          defaultValue={[0]}
          value={[muted ? 0 : volume * 100]}
          max={100}
          step={1}
          onValueChange={(c: number[]) => handleVolumeChange(c[0], false)}
          onValueCommit={(e: number[]) => handleVolumeChange(e[0], true)}
          className="group relative flex h-5 w-32 items-center"
        >
          <Slider.Track className="relative h-1 w-full grow rounded-full bg-gray-400 dark:bg-gray-800">
            <Slider.Range className="absolute h-full rounded-full bg-purple-600 last:rounded-r-none dark:bg-murasaki-500" />
          </Slider.Track>
          <Slider.Thumb className="block h-2 group-hover:h-3 aspect-square rounded-full bg-purple-600 dark:bg-white focus:outline-none focus-visible:ring focus-visible:ring-purple-500 focus-visible:ring-opacity-75 transition-all duration-200" />
        </Slider.Root>
      </div>
    </div>
  );
}
