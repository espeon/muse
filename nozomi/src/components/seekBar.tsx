import s2t from "@/helpers/s2t";
import { useSmoothTimer } from "@/stores/useSmoothTimer";
import React, { useState, useEffect, useRef } from "react";

interface SeekBarProps {
  duration: number;
  currentTime: number;
  onSeek?: (time: number) => void;
  onMouseDown?: (e: React.MouseEvent<HTMLInputElement, MouseEvent>) => void;
  onMouseUp?: (e: React.MouseEvent<HTMLInputElement, MouseEvent>) => void;
  onTouchStart?: (e: React.TouchEvent<HTMLInputElement>) => void;
  onTouchEnd?: (e: React.TouchEvent<HTMLInputElement>) => void;
  isActivelyPlaying?: boolean;
  className?: string;
  barType?: "range" | "progress";
}

const SeekBar: React.FC<SeekBarProps> = ({
  duration,
  currentTime,
  onSeek,
  isActivelyPlaying = true,
  barType = "range",
  ...props
}) => {
  const smt = useSmoothTimer({ currentTime, duration, isActivelyPlaying });

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    console.log("handleChange", e.target.value);
    const value = Number(e.target.value);
    smt.setLocalTime(value / 100);
    onSeek && onSeek(value / 100);
  };

  if (barType === "progress") {
    return (
      <progress
        value={smt.currentTime}
        max={duration}
        className={props.className}
      />
    );
  } else {
    return (
      <>
        <input
          type="range"
          min="0"
          max={duration * 100}
          value={smt.currentTime * 100}
          onChange={handleChange}
          onMouseDown={props.onMouseDown}
          onMouseUp={props.onMouseUp}
          onTouchStart={props.onTouchStart}
          onTouchEnd={props.onTouchEnd}
          className={props.className}
        />
      </>
    );
  }
};

export default SeekBar;
