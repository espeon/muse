import { create } from "zustand";
import { usePlayerStore } from "./playerStore";
import { createJSONStorage, persist } from "zustand/middleware";

export interface Track {
  title: string;
  artist: string;
  album: string;
  artwork: string | null;
  stream: string;
  context?: string;
  uuid?: string;
  scrobbled?: boolean;
}

export enum ContextType {
  Playlist = "playlist",
  Album = "album",
  Artist = "artist",
  Genre = "genre",
  Station = "station",
  Search = "search",
}

export interface Context {
  tracks: Track[];
  type: ContextType;
  id: string;
  display?: string;
}

interface QueueState {
  pastQueue: Track[];
  queue: Track[];
  currentTrack: Track | null;
  // track to play next
  trackUpNext: Track | null;
  currentContext: Context | null;
  addTrack: (track: Track) => void;
  // play track next
  addTrackHead: (track: Track) => void;
  addTrackPast: (track: Track) => void;
  addTrackPastHead: (track: Track) => void;
  // Remove track removes from both current and past queues
  removeTrack: (track: Track) => void;
  // play track *now*
  // internally, use addTrackHead and popTrack
  playTrack: (track: Track) => void;
  setScrobbled: (bool?: boolean) => void;
  // remove track from beginning of queue and set it as current
  // also return it
  popTrack: () =>
    | { track: Track; switchToNextTrack: () => void }
    | { track: null; switchToNextTrack: () => void };
  popPastTrack: () => Track;
  clearQueue: () => void;
  clearPastQueue: () => void;
  setContext: (context: Context) => void;
}

export const useQueueStore = create<QueueState>()(
  persist(
    (set) => ({
      pastQueue: [],
      queue: [],
      currentTrack: null,
      trackUpNext: null,
      currentContext: null,
      addTrack: (track) => {
        set((state) => ({ queue: [...state.queue, track] }));
        if (useQueueStore.getState().currentTrack === null) {
          const result = useQueueStore.getState().popTrack();
          if (result.track) {
            result.switchToNextTrack();
          }
        }
      },
      addTrackHead: (track) =>
        set((state) => ({ queue: [track, ...state.queue] })),
      addTrackPast: (track) =>
        set((state) => ({ pastQueue: [...state.pastQueue, track] })),
      addTrackPastHead(track) {
        set((state) => ({ pastQueue: [track, ...state.pastQueue] }));
      },
      removeTrack: (track) =>
        set((state) => ({
          queue: state.queue.filter((t) => t !== track),
          pastQueue: state.pastQueue.filter((t) => t !== track),
        })),
      playTrack: (track) => {
        // if the track is currently being played, do nothing
        if (usePlayerStore.getState().media.url === track.stream) {
          return;
        }
        set((state) => {
          state.addTrackHead(track);
          const { track: nt, switchToNextTrack } = useQueueStore
            .getState()
            .popTrack();
          if (nt) {
            // This will handle the actual track switch
            switchToNextTrack();
          }
          return {};
        });
      },
      // set the current track as 'scrobbled'
      setScrobbled: (bool = true) => {
        set((state) => {
          state.currentTrack!.scrobbled = bool;
          return {};
        });
      },
      popTrack: () => {
        const state = useQueueStore.getState();
        const playerState = usePlayerStore.getState();
        let track: Track = state.queue[0];

        // Handle current track (don't move to past queue yet since it's still playing)
        // We'll move it to past queue when the next track actually starts

        // Remove next track from queue
        if (track !== undefined) {
          state.removeTrack(track);
        } else {
          // Handle context when no track in queue
          if (state.currentContext !== null) {
            track = state.currentContext.tracks.shift()!;

            // Clear context if no more tracks
            if (state.currentContext.tracks.length === 0) {
              set((state) => ({ currentContext: null }));
            }
          }
        }

        // Prepare next track but don't start playing yet
        if (track) {
          // Load the media but keep it paused
          playerState.setUpNextMedia(track.stream);
          playerState.seek(0);

          // Store the track we're about to play
          set((state) => ({ trackUpNext: track }));

          // should we play this track gaplessly? (for now, if )
          const gapless = track.album === state.queue[0]?.album;
          // Set up a callback for when we actually switch to this track
          const switchToNextTrack = () => {
            console.log("Switching to next track");
            // Move current track to past queue
            if (state.currentTrack !== null) {
              state.addTrackPast(state.currentTrack);
            }

            // Update current track
            set((state) => ({
              currentTrack: track,
              trackUpNext: null,
            }));

            // Start playing the prepared track
            console.log(
              "Starting to play next track" + gapless ? " gapless" : "",
            );
            playerState.switchToUpNext(gapless);

            console.log(state.currentContext);

            // Prime next-next track
            const nextTrack = state.currentContext
              ? state.currentContext.tracks[0]
              : state.queue[0];

            if (nextTrack) {
              console.log("Setting up next track: ", nextTrack);
              playerState.setUpNextMedia(nextTrack.stream);
              set((state) => ({ trackUpNext: nextTrack }));
            }
          };

          // Return both the track and the switch function
          console.log("Popped track, now should switch to next track");
          return {
            track,
            switchToNextTrack,
          };
        }

        return {
          track: null,
          switchToNextTrack: () => {},
        };
      },
      popPastTrack: () => {
        const track: Track =
          useQueueStore.getState().pastQueue[
            useQueueStore.getState().pastQueue.length - 1
          ];
        useQueueStore.getState().removeTrack(track);

        // put the current playing track in the current queue
        if (useQueueStore.getState().currentTrack !== null) {
          useQueueStore
            .getState()
            .addTrackHead(useQueueStore.getState().currentTrack!);
        }
        // Queue it in the player
        usePlayerStore.getState().setMedia(track.stream);
        usePlayerStore.getState().seek(0);
        usePlayerStore.getState().setPlaying(false);

        set((state) => ({ currentTrack: track }));
        return track;
      },
      clearQueue: () => set({ queue: [] }),
      clearPastQueue: () => set({ pastQueue: [] }),
      setContext: (context) => set({ currentContext: context }),
    }),
    {
      name: "store-queue-config-abzk2", // name of item in the storage (must be unique)
      storage: createJSONStorage(() => sessionStorage), // (optional) by default the 'localStorage' is used
      // only store the queue, currentTrack, and currentContext
      partialize: (state) => ({
        queue: state.queue,
        currentTrack: state.currentTrack,
        currentContext: state.currentContext,
      }),
    },
  ),
);
