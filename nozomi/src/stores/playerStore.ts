import { create } from "zustand";
import { usePlayerRefStore } from "./playerRefStore";

interface PlayerState {
  isPlaying: boolean;
  currentTime: number;
  duration: number;
  volume: number;
  media: string;
  muted: boolean;
  isSeeking: boolean;
  scrobbled: boolean;
  setMedia: (media: string) => void;
  setVolume: (volume: number) => void;
  setMuted: (muted: boolean) => void;
  setCurrentTime: (currentTime: number, global: boolean) => void;
  setScrobbled: () => void;
  setDuration: (duration: number) => void;
  togglePlaying: () => void;
  setPlaying: (isPlaying: boolean) => void;
  seek: (time: number) => void;
  setSeeking: (isSeeking: boolean) => void;
}

export const usePlayerStore = create<PlayerState>((set) => ({
  isPlaying: false,
  currentTime: 0,
  duration: 0,
  volume: 1.0,
  media: "",
  muted: false,
  isSeeking: false,
  scrobbled: false,
  setMedia: (media: string) => set({ media }),
  setVolume: (volume: number) => set({ volume }),
  setMuted: (muted: boolean) => set({ muted }),
  setCurrentTime: (currentTime: number, global: boolean = false) => {
    // set the current time in the player
    if (global) {
      usePlayerRefStore
        .getState()
        .playerRef?.current?.seekTo(currentTime, "seconds");
    }
    set({ currentTime });
  },
  setScrobbled: () => {
    set({ scrobbled: true });
  },
  setDuration: (duration: number) => set({ duration }),
  togglePlaying: () => set((state) => ({ isPlaying: state.media ? !state.isPlaying : false })),
  setPlaying: (isPlaying: boolean) => set({ isPlaying: isPlaying }),
  seek: (time: number) => set({ currentTime: time }),
  setSeeking: (isSeeking: boolean) => set({ isSeeking }),
}));
