import { create } from "zustand";
import { usePlayerRefStore } from "./playerRefStore";
import { createJSONStorage, persist } from "zustand/middleware";
import getTrackUrl from "@/helpers/getTrackUrl";

interface PlayerState {
  isPlaying: boolean;
  isPlaying1: boolean;
  isPlaying2: boolean;
  currentTime: number;
  duration: number;
  volume: number;
  media: {
    url: string;
    track_id: string;
  };
  media2: {
    url: string;
    track_id: string;
  };
  currentPlayerIs: PlayerType;
  muted: boolean;
  isSeeking: boolean;
  scrobbled: boolean;
  setMedia: (media: string) => void;
  setUpNextMedia: (media: string) => void;
  switchToUpNext: (gapless: boolean) => void;
  setVolume: (volume: number) => void;
  setMuted: (muted: boolean) => void;
  setCurrentTime: (currentTime: number, global: boolean) => void;
  setScrobbled: () => void;
  setDuration: (duration: number) => void;
  togglePlaying: () => void;
  setPlaying: (isPlaying: boolean) => void;
  seek: (time: number) => void;
  setSeeking: (isSeeking: boolean) => void;
  isBuffering: boolean;
  setBuffering: (isBuffering: boolean) => void;
}

export enum PlayerType {
  PLAYER1 = "player1",
  PLAYER2 = "player2",
}

export const usePlayerStore = create<PlayerState>()(
  persist(
    (set, get) => ({
      isPlaying: false,
      isPlaying1: false,
      isPlaying2: false,
      currentTime: 0,
      duration: 0,
      volume: 1.0,
      media: {
        url: "",
        track_id: "",
      },
      media2: {
        url: "",
        track_id: "",
      },
      currentPlayerIs: PlayerType.PLAYER1,
      muted: false,
      isSeeking: false,
      scrobbled: false,
      // set the media for the player that is currently playing
      setMedia: (media: string) => {
        // if media to set is the same as the up next media, switch to the other player
        console.log("playerStore.setMedia: ", media);
        let otherPlayer =
          get().currentPlayerIs === PlayerType.PLAYER1
            ? PlayerType.PLAYER2
            : PlayerType.PLAYER1;
        if (
          media ===
          (otherPlayer === PlayerType.PLAYER1
            ? usePlayerStore.getState().media.track_id
            : usePlayerStore.getState().media2.track_id)
        ) {
          console.log(
            "playerStore.setMedia: switching to other player: ",
            otherPlayer,
          );
          set({ currentPlayerIs: otherPlayer });
        } else {
          // if current player is player 1, set the media for player 1
          if (
            usePlayerStore.getState().currentPlayerIs === PlayerType.PLAYER1
          ) {
            getTrackUrl(media).then((url) => {
              set({ media: { url, track_id: media } });
            });
          } else {
            // if current player is player 2 , set the media for player 2
            getTrackUrl(media).then((url) => {
              set({ media2: { url, track_id: media } });
            });
          }
        }
        // reset scrobbling status
        set({ scrobbled: false });
      },
      // set the media for the player that not currently playing, will be played next
      setUpNextMedia: (media: string) => {
        // if current player is player 1, set the media for player 2
        if (usePlayerStore.getState().currentPlayerIs === PlayerType.PLAYER1) {
          getTrackUrl(media).then((url) => {
            set({ media2: { url, track_id: media } });
          });
        } else {
          // if current player is player 2 , set the media for player 1
          getTrackUrl(media).then((url) => {
            set({ media: { url, track_id: media } });
          });
        }
      },
      switchToUpNext: (gapless: boolean = false) => {
        // Switch to the other player
        const newPlayer =
          get().currentPlayerIs === PlayerType.PLAYER1
            ? PlayerType.PLAYER2
            : PlayerType.PLAYER1;

        set({
          currentPlayerIs: newPlayer,
          // Reset scrobble status for the new track
          scrobbled: false,
        });

        console.log("Switching to new player: ", newPlayer);
        let isPlaying = false;
        if (get().currentPlayerIs === PlayerType.PLAYER1) {
          console.log("playerStore.setPlaying to player 1: ", isPlaying);
          set({ isPlaying1: isPlaying, isPlaying2: false });
          // set other playe
        } else {
          console.log("playerStore.setPlaying to player 2: ", isPlaying);
          set({ isPlaying2: isPlaying, isPlaying1: false });
        }
        set({ isPlaying });
      },
      setVolume: (volume: number) => set({ volume }),
      setMuted: (muted: boolean) => set({ muted }),
      setCurrentTime: (currentTime: number, global: boolean = false) => {
        if (global) {
          if (get().currentPlayerIs === PlayerType.PLAYER1) {
            console.log("Seeking to", currentTime, "on player 1");
            usePlayerRefStore
              .getState()
              .playerRef1?.current?.seekTo(currentTime, "seconds");
          } else {
            console.log("Seeking to", currentTime, "on player 2");
            try {
              console.log(usePlayerRefStore.getState().playerRef2);
              usePlayerRefStore
                .getState()
                .playerRef2?.current?.seekTo(currentTime);
            } catch (e) {
              console.log("Error seeking to", currentTime, "on player 2");
            }
          }
        }
        set({ currentTime });
      },
      setScrobbled: () => {
        set({ scrobbled: true });
      },
      setDuration: (duration: number) => set({ duration }),
      togglePlaying: () => get().setPlaying(!get().isPlaying),
      // set internal and external playing values based on the current player
      setPlaying: (isPlaying: boolean) => {
        if (get().currentPlayerIs === PlayerType.PLAYER1) {
          console.log("playerStore.setPlaying to player 1: ", isPlaying);
          set({ isPlaying1: isPlaying, isPlaying2: true });
        } else {
          console.log("playerStore.setPlaying to player 2: ", isPlaying);
          set({ isPlaying2: isPlaying, isPlaying1: true });
        }
        set({ isPlaying });
      },
      seek: (time: number) => set({ currentTime: time }),
      setSeeking: (isSeeking: boolean) => set({ isSeeking }),
      isBuffering: true,
      setBuffering: (isBuffering: boolean) => set({ isBuffering: isBuffering }),
    }),
    {
      name: "store-player-config-abzk2", // name of item in the storage (must be unique)
      storage: createJSONStorage(() => sessionStorage), // (optional) by default the 'localStorage' is used
      partialize: (state) => ({
        // the isPlaying is opposite of what it should be? fix this wtf
        isPlaying: true,
        isPlaying1: true,
        isPlaying2: true,
        currentTime: state.currentTime,
        duration: state.duration,
        volume: state.volume,
        // store the current playing track in the first slot of the queue
        media:
          state.currentPlayerIs === PlayerType.PLAYER1
            ? state.media
            : state.media2,
        muted: state.muted,
        isSeeking: state.isSeeking,
        scrobbled: state.scrobbled,
      }),
    },
  ),
);
