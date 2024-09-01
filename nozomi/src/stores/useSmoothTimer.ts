import { useState, useEffect, useRef, useCallback } from "react";

interface UseSmoothTimerOptions {
  duration: number;
  currentTime: number;
  onUpdate?: (time: number) => void;
  isActivelyPlaying?: boolean;
  bounds?: [min: number, max: number];
}

export interface TimerControls {
  currentTime: number;
  resetTimer: () => void;
  setLocalTime: (time: number) => void;
}

const useThrottle = (callback: Function, delay: number) => {
  const lastCall = useRef(0);

  return useCallback(
    (...args: any[]) => {
      const now = Date.now();
      if (now - lastCall.current >= delay) {
        lastCall.current = now;
        callback(...args);
      }
    },
    [callback, delay],
  );
};

export const useSmoothTimer = ({
  duration,
  currentTime,
  onUpdate,
  isActivelyPlaying = true,
  bounds = [3, 5],
}: UseSmoothTimerOptions): TimerControls => {
  const [internalTime, setInternalTime] = useState<number>(currentTime);
  const animationRef = useRef<number | null>(null);
  const startTimeRef = useRef<number | null>(null);
  const startValueRef = useRef<number>(currentTime);

  // Use a callback to memoize onUpdate
  const memoizedOnUpdate = useCallback(
    (time: number) => {
      if (onUpdate) onUpdate(time);
    },
    [onUpdate],
  );

  const setThrottledInternalTime = useThrottle(setInternalTime, 100);

  // effect to update onUpdate
  useEffect(() => {
    if (onUpdate) onUpdate(internalTime);
  }, [internalTime]);

  useEffect(() => {
    if (!isActivelyPlaying) {
      setInternalTime(currentTime);
      startTimeRef.current = null;
      startValueRef.current = currentTime;
      return;
    }

    if (startTimeRef.current === null) {
      startTimeRef.current = performance.now();
      startValueRef.current = currentTime;
    }

    const animate = (time: DOMHighResTimeStamp) => {
      if (startTimeRef.current !== null) {
        const elapsed = (time - startTimeRef.current) / 1000; // Convert ms to seconds
        const predicted = startValueRef.current + elapsed;
        setThrottledInternalTime(Math.min(predicted, duration)); // Ensure time doesn't exceed duration

        if (predicted < duration) {
          animationRef.current = requestAnimationFrame(animate);
        }
      }
    };

    animationRef.current = requestAnimationFrame(animate);

    // if our predicted time is outside the bounds compared to the current time, reset to current time
    if (
      internalTime < bounds[0] - currentTime ||
      internalTime > bounds[1] + currentTime
    ) {
      setInternalTime(currentTime);
    }

    return () => {
      if (animationRef.current !== null) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, [isActivelyPlaying, currentTime, duration]);

  const resetTimer = () => {
    setInternalTime(currentTime);
    startTimeRef.current = null;
    startValueRef.current = currentTime;
    if (animationRef.current !== null) {
      cancelAnimationFrame(animationRef.current);
      animationRef.current = null;
    }
  };

  const setLocalTime = (time: number) => {
    setInternalTime(time);
    startTimeRef.current = null; // Reset the start time on manual set
    startValueRef.current = time;
  };

  return { currentTime: internalTime, resetTimer, setLocalTime };
};
