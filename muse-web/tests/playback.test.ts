import { afterEach, beforeEach, expect, test } from "bun:test";
import { GaplessQueue } from "../src/player/queue";
import type { TrackInfo } from "../src/player/types";

const playing = new Set<string>();
const audioBySource = new Map<string, TestAudio>();
const frames = new Map<number, () => void>();
let nextFrameId = 0;

class TestAudio extends EventTarget {
  currentTime = 0;
  src = "";

  async play() {
    playing.add(this.src);
    audioBySource.set(this.src, this);
  }

  pause() {
    playing.delete(this.src);
  }

  removeAttribute(name: string) {
    if (name === "src") this.src = "";
  }

  load() {}
}

let browserGlobals: PropertyDescriptorMap;

beforeEach(() => {
  browserGlobals = Object.getOwnPropertyDescriptors(globalThis);
  Object.assign(globalThis, {
    Audio: TestAudio,
    requestAnimationFrame: (callback: () => void) => {
      frames.set(++nextFrameId, callback);
      return nextFrameId;
    },
    cancelAnimationFrame: (id: number) => frames.delete(id),
  });
});

afterEach(() => {
  for (const name of ["Audio", "requestAnimationFrame", "cancelAnimationFrame"]) {
    const descriptor = browserGlobals[name];
    if (descriptor) Object.defineProperty(globalThis, name, descriptor);
    else Reflect.deleteProperty(globalThis, name);
  }
  playing.clear();
  audioBySource.clear();
  frames.clear();
});

function track(id: number): TrackInfo {
  return { id, title: `Track ${id}`, artistName: "Artist", albumName: "Album", duration: 30 };
}

function progress(id: number, seconds: number) {
  audioBySource.get(`track-${id}`)!.currentTime = seconds;
  const callbacks = [...frames.values()];
  frames.clear();
  callbacks.forEach((callback) => callback());
}

test("replacing the queue cancels the previous pending start", async () => {
  const signing = Promise.withResolvers<string>();
  const nowPlaying: number[] = [];
  const queue = new GaplessQueue({
    resolveUrl: (id) => id === 1 ? signing.promise : Promise.resolve(`track-${id}`),
    callbacks: { onTrackChange: (_, info) => nowPlaying.push(info.id) },
  });

  const oldStart = queue.setQueue([track(1)]);
  await queue.setQueue([track(2)]);
  signing.resolve("track-1");
  await oldStart;

  expect(queue.currentInfo?.id).toBe(2);
  expect([...playing]).toEqual(["track-2"]);
  expect(nowPlaying).toEqual([2]);
  queue.dispose();
});

test("skipping cancels a pending start without preventing a later retry", async () => {
  const signing = Promise.withResolvers<string>();
  const queue = new GaplessQueue({
    resolveUrl: (id) => id === 1 ? signing.promise : Promise.resolve(`track-${id}`),
  });

  const oldStart = queue.setQueue([track(1), track(2)]);
  await queue.next();
  signing.resolve("track-1");
  await oldStart;
  expect([...playing]).toEqual(["track-2"]);

  await queue.previous();
  expect([...playing]).toEqual(["track-1"]);
  queue.dispose();
});

test("a signing failure for the current track rejects the start", async () => {
  const queue = new GaplessQueue({
    resolveUrl: () => Promise.reject(new Error("Signing failed")),
  });

  await expect(queue.setQueue([track(1)])).rejects.toThrow("Signing failed");
  expect(queue.isPlaying).toBe(false);
  queue.dispose();
});

test("pause and resume publish the playback state without waiting for a frame", async () => {
  const states: boolean[] = [];
  const queue = new GaplessQueue({
    resolveUrl: async () => "track-1",
    callbacks: { onStateChange: () => states.push(queue.isPlaying) },
  });
  await queue.setQueue([track(1)]);
  progress(1, 8);

  queue.pause();
  expect(states.at(-1)).toBe(false);
  expect(queue.currentTime).toBe(8);

  await queue.play();
  expect(states.at(-1)).toBe(true);
  expect(queue.currentTime).toBe(8);
  queue.dispose();
});

test("a pause while signing stops the pending start and allows playback later", async () => {
  const signing = Promise.withResolvers<string>();
  const queue = new GaplessQueue({ resolveUrl: () => signing.promise });
  const start = queue.setQueue([track(1)]);
  queue.pause();
  signing.resolve("track-1");
  await start;
  expect(queue.isPlaying).toBe(false);
  expect([...playing]).toEqual([]);

  await queue.play();
  expect(queue.isPlaying).toBe(true);
  queue.dispose();
});

test("the last track stops publishing frames and can be replayed from the beginning", async () => {
  const statesAtEnd: boolean[] = [];
  const queue = new GaplessQueue({
    resolveUrl: async () => "track-1",
    callbacks: { onQueueEnd: () => statesAtEnd.push(queue.isPlaying) },
  });
  await queue.setQueue([track(1)]);
  const audio = audioBySource.get("track-1")!;
  audio.currentTime = 30;
  audio.dispatchEvent(new Event("ended"));

  expect(statesAtEnd).toEqual([false]);
  expect(frames.size).toBe(0);
  await queue.play();
  expect(queue.isPlaying).toBe(true);
  expect(queue.currentTime).toBe(0);
  queue.dispose();
});

test("scrobbles after actual listening across pauses and seeks, once per queue entry", async () => {
  const scrobbles: number[] = [];
  const queue = new GaplessQueue({
    resolveUrl: async () => "track-1",
    callbacks: { onScrobble: (info: TrackInfo) => scrobbles.push(info.id) },
  });
  await queue.setQueue([track(1), track(1)]);
  progress(1, 4);
  queue.seekTo(20);
  progress(1, 23);
  queue.pause();
  queue.seekTo(2);
  expect(scrobbles).toEqual([]);

  await queue.play();
  progress(1, 10);
  expect(scrobbles).toEqual([1]);
  queue.seekTo(0);
  progress(1, 16);
  expect(scrobbles).toEqual([1]);

  await queue.next();
  progress(1, 15);
  expect(scrobbles).toEqual([1, 1]);
  queue.dispose();
});

test("long tracks scrobble at four minutes of listening", async () => {
  const scrobbles: number[] = [];
  const queue = new GaplessQueue({
    resolveUrl: async () => "track-1",
    callbacks: { onScrobble: (info) => scrobbles.push(info.id) },
  });
  await queue.setQueue([{ ...track(1), duration: 600 }]);
  progress(1, 239);
  expect(scrobbles).toEqual([]);
  progress(1, 240);
  expect(scrobbles).toEqual([1]);
  queue.dispose();
});

test("repeat starts the queue again when the last track ends", async () => {
  const started = Promise.withResolvers<void>();
  const queue = new GaplessQueue({
    resolveUrl: async (id) => `track-${id}`,
    callbacks: { onTrackChange: (_, info) => {
      if (info.id === 1) started.resolve();
    } },
  });
  await queue.setQueue([track(1), track(2)], 1);
  queue.setRepeat(true);

  audioBySource.get("track-2")!.dispatchEvent(new Event("ended"));

  await started.promise;
  expect(queue.currentInfo?.id).toBe(1);
  expect(queue.currentTime).toBe(0);
  expect([...playing]).toEqual(["track-1"]);
  queue.dispose();
});
