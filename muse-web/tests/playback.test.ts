import { afterEach, beforeEach, expect, test } from "bun:test";
import { GaplessQueue } from "../src/player/queue";
import type { TrackInfo } from "../src/player/types";

const playing = new Set<string>();

class TestAudio extends EventTarget {
  currentTime = 0;
  src = "";

  async play() {
    playing.add(this.src);
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
    requestAnimationFrame: () => 1,
    cancelAnimationFrame: () => {},
  });
});

afterEach(() => {
  for (const name of ["Audio", "requestAnimationFrame", "cancelAnimationFrame"]) {
    const descriptor = browserGlobals[name];
    if (descriptor) Object.defineProperty(globalThis, name, descriptor);
    else Reflect.deleteProperty(globalThis, name);
  }
  playing.clear();
});

function track(id: number): TrackInfo {
  return { id, title: `Track ${id}`, artistName: "Artist", albumName: "Album", duration: 30 };
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
