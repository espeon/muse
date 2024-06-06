import ReactPlayer from "react-player";
import { create } from "zustand";

interface PlayerRefState {
  playerRef: React.RefObject<ReactPlayer> | null;
  setPlayerRef: (ref: React.RefObject<ReactPlayer> | null) => void;
}

export const usePlayerRefStore = create<PlayerRefState>((set) => ({
  playerRef: null,
  setPlayerRef: (ref) => set({ playerRef: ref }),
}));
