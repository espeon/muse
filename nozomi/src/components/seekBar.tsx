import s2t from "@/helpers/s2t";
import { useSmoothTimer, useThrottle } from "@/stores/useSmoothTimer";
import React, { useState, useEffect, useRef } from "react";
import * as Slider from "@radix-ui/react-slider";
import { clsx } from "clsx";

interface SeekBarProps {
  duration: number;
  currentTime: number;
  onSeek?: (time: number) => void;
  onValueChange?: (value: number[]) => void;
  onValueCommit?: (value: number[]) => void;
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
  const [valueChange, setValueChange] = useState<boolean>(false);
  const [localTime, setLocalTime] = useState<number>(currentTime);
  const setThrottledLocalTime = useThrottle(setLocalTime, 5);

  const onValueChange = (e: number[]) => {
    setValueChange(true);
    setThrottledLocalTime(e[0]);
    props.onValueChange?.(e);
  };
  const onValueCommit = (e: number[]) => {
    setValueChange(false);
    smt.setLocalTime(e[0] / 100);
    props.onValueCommit?.(e);
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
        <Slider.Root
          defaultValue={[0]}
          value={[valueChange ? localTime : smt.currentTime * 100]}
          max={duration * 100}
          step={1}
          onValueChange={onValueChange}
          onValueCommit={onValueCommit}
          className={clsx(
            "group relative flex h-5 w-full items-center",
            props.className,
          )}
        >
          <Slider.Track className="relative h-1 w-full grow rounded-full bg-gray-400 dark:bg-gray-800">
            <Slider.Range className="absolute h-full rounded-full bg-purple-600 last:rounded-r-none dark:bg-wisteria-500" />
          </Slider.Track>
          <Slider.Thumb className="block h-2 group-hover:h-3 aspect-square rounded-full bg-purple-600 dark:bg-white focus:outline-none focus-visible:ring focus-visible:ring-purple-500 focus-visible:ring-opacity-75 transition-all duration-200" />
        </Slider.Root>
      </>
    );
  }
};

export default SeekBar;
