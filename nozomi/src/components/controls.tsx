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

  const handleSeekChange = (c: ChangeEvent<HTMLInputElement>) => {
    let value = c.currentTarget.value;
    setCurrentLocalTime(Number(value) / 100);
  };

  const handleVolumeChange = (c: ChangeEvent<HTMLInputElement>) => {
    let value = c.currentTarget.value;
    if (value == "0") {
      setMuted(true);
    } else {
      setMuted(false);
    }
    setVolume(Number(value) / 100);
  };

  const handleMute = () => {
    setMuted(!muted);
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

  const handleVolumeDrag = (e: any) => {
    const action = e._reactName;
    if (action === "onMouseDown") {
      if (volume !== 0) {
        setLastVolume(volume);
      }
    }
    if (action === "onMouseUp") {
      if (volume == 0) {
        setVolume(lastVolume);
      }
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
      <div className="flex flex-row justify-start items-center flex-1 pl-4 py-1">
        {currentTrack && (
          <>
            <div className="block margin-auto aspect-square max-w-full h-full mr-4">
              <img
                src={
                  currentTrack?.artwork ?? "https://i.imgur.com/moGByde.jpeg"
                }
                className="mx-auto max-h-full self-center contain-content rounded-lg margin-auto ambilight"
              />
            </div>
            <div className="">
              <div className="line-clamp-1">{currentTrack?.title}</div>
              <div>{currentTrack?.artist}</div>
            </div>
            <TbHeart className="h-6 w-6 ml-6 mr-8" />
          </>
        )}
      </div>
      <div className="flex flex-col justify-center align-baseline items-center">
        <div className="flex flex-row gap-4 pb-2">
          <PiShuffle className="h-5 w-5 mt-2.5" />
          <PiCaretLineLeft
            className="h-6 w-6 mt-2 hover:text-pink-400 transition-colors duration-300"
            onClick={() => {
              if (currentLocalTime < 3) {
                popPastTrack();
              } else {
                setCurrentTime(0, true);
              }
            }}
          />
          <PlayPauseIcon
            className="h-10 w-10 hover:text-pink-400 transition-colors duration-300"
            onClick={() => togglePlaying()}
          />
          <PiCaretLineRight
            className="h-6 w-6 mt-2 hover:text-pink-400 transition-colors duration-300"
            onClick={() => popTrack()}
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
            onMouseDown={(e) => handleSeekDrag(e)}
            onMouseUp={(e) => handleSeekDrag(e)}
            onTouchStart={(e) => handleSeekDrag(e)}
            onTouchEnd={(e) => handleSeekDrag(e)}
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
          <PiMicrophoneStageDuotone className="h-6 w-6 mr-4 hover:text-pink-400 transition-colors duration-300" />
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
        <input
          type="range"
          min="0"
          max={100}
          value={muted ? 0 : volume * 100}
          className="range w-screen max-w-28 cursor-pointer"
          onChange={(c) => handleVolumeChange(c)}
          onMouseDown={(e) => handleVolumeDrag(e)}
          onMouseUp={(e) => handleVolumeDrag(e)}
        />
      </div>
    </div>
  );
}
