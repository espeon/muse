import { create } from "zustand";
import { usePlayerStore } from "./playerStore";

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
  setScrobbled: () => void;
  // remove track from beginning of queue and set it as current
  // also return it
  popTrack: () => Track;
  popPastTrack: () => Track;
  clearQueue: () => void;
  clearPastQueue: () => void;
  setContext: (context: Context) => void;
}

export const useQueueStore = create<QueueState>((set) => ({
  pastQueue: [],
  queue: [],
  currentTrack: null,
  currentContext: null,
  addTrack: (track) => {
    set((state) => ({ queue: [...state.queue, track] }));
    if (useQueueStore.getState().currentTrack === null) {
      useQueueStore.getState().popTrack();
    }
  },
  addTrackHead: (track) => set((state) => ({ queue: [track, ...state.queue] })),
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
    if (usePlayerStore.getState().media === track.stream) {
      return;
    }
    set((state) => {
      state.addTrackHead(track);
      state.popTrack();
      return {};
    });
  },
  // set the current track as 'scrobbled'
  setScrobbled: () => {
    set((state) => {
      state.currentTrack!.scrobbled = true;
      return {};
    });
  },
  popTrack: () => {
    let track: Track = useQueueStore.getState().queue[0];
    useQueueStore.getState().removeTrack(track);
    // put the current playing track in the past queue
    let past = useQueueStore.getState().currentTrack;
    if (useQueueStore.getState().currentTrack !== null) {
      useQueueStore
        .getState()
        .addTrackPast(useQueueStore.getState().currentTrack!);
    }
    // check if there is no track
    if (track == undefined) {
      // if there is no current track, see if there is a context
      if (useQueueStore.getState().currentContext !== null) {
        // if there is a context, set the current track to the first track in the context
        let ftrack = useQueueStore.getState().currentContext!.tracks[0];
        // remove the first track from the context
        useQueueStore
          .getState()
          .currentContext!.tracks.shift();
        // check if there are any tracks in the context
        if (useQueueStore.getState().currentContext!.tracks.length === 0) {
          // if there are no tracks in the context, remove the context
          useQueueStore.getState().currentContext = null;
        }

        console.log("queueing track: ", ftrack);
        usePlayerStore.getState().setMedia(ftrack.stream);
        usePlayerStore.getState().seek(0);
        usePlayerStore.getState().setPlaying(false);
        console.log("Should be playing", usePlayerStore.getState().isPlaying);
        set((state) => ({ currentTrack: ftrack }));
        return track;
      }
    }
    // queue it in the player
    console.log("queueing track: ", track);
    usePlayerStore.getState().setMedia(track.stream);
    usePlayerStore.getState().seek(0);
    usePlayerStore.getState().setPlaying(false);
    console.log("Should be playing", usePlayerStore.getState().isPlaying);
    set((state) => ({ currentTrack: track }));
    return track;
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
}));

function uuidv4() {
  return "10000000-1000-4000-8000-100000000000".replace(/[018]/g, (c) =>
    (
      +c ^
      (crypto.getRandomValues(new Uint8Array(1))[0] & (15 >> (+c / 4)))
    ).toString(16)
  );
}
