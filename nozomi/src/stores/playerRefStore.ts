import ReactPlayer from "react-player";
import { create } from "zustand";

interface PlayerRefState {
  playerRef1: React.RefObject<ReactPlayer> | null;
  playerRef2: React.RefObject<ReactPlayer> | null;
  setPlayerRef1: (ref: React.RefObject<ReactPlayer> | null) => void;
  setPlayerRef2: (ref: React.RefObject<ReactPlayer> | null) => void;
}

export const usePlayerRefStore = create<PlayerRefState>((set) => ({
  playerRef1: null,
  playerRef2: null,
  setPlayerRef1: (ref) => set({ playerRef1: ref }),
  setPlayerRef2: (ref) => set({ playerRef2: ref }),
}));
