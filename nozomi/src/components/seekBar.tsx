import s2t from "@/helpers/s2t";
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
  const [sliderValue, setSliderValue] = useState<number>(currentTime);
  const animationRef = useRef<number | null>(null);
  const lastTimeRef = useRef<number>(currentTime);

  useEffect(() => {
    if (animationRef.current !== null) {
      cancelAnimationFrame(animationRef.current);
    }

    // Only interpolate if the playback is not paused
    if (isActivelyPlaying) {
      interpolateSlider(currentTime);
    }
    // set the slider value to the current time if the playback is paused
    else {
      setSliderValue(currentTime);
    }

    // Update the reference to the last known time
    lastTimeRef.current = currentTime;
  }, [currentTime, isActivelyPlaying]);

  const interpolateSlider = (startValue: number) => {
    const startTime = performance.now();

    const animate = (time: DOMHighResTimeStamp) => {
      if (!isActivelyPlaying) return;

      const elapsed = time - startTime;
      const predictedValue = startValue + elapsed / 1000; // Predict based on 1 second intervals

      if (predictedValue < duration) {
        setSliderValue(predictedValue);
        animationRef.current = requestAnimationFrame(animate);
      } else {
        setSliderValue(duration);
        if (animationRef.current !== null) {
          cancelAnimationFrame(animationRef.current);
        }
      }
    };

    animationRef.current = requestAnimationFrame(animate);
  };

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    console.log("handleChange", e.target.value);
    const value = Number(e.target.value);
    setSliderValue(value / 100);
    onSeek && onSeek(value / 100);
  };

  if (barType === "progress") {
    return (
      <progress
        value={sliderValue}
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
          value={sliderValue * 100}
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
