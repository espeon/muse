"use client";
import { useQueueStore } from "@/stores/queueStore";
import { Track } from "@/stores/queueStore";
import { TbTrash } from "react-icons/tb";
import DebugMenu from "./debugMenu";
import { BiChevronDown } from "react-icons/bi";
import { useState } from "react";

export default function Queue() {
  const {
    pastQueue,
    queue,
    currentTrack,
    addTrack,
    clearQueue,
    removeTrack,
    popTrack,
    currentContext,
  } = useQueueStore();

  const [showLastQueue, setShowLastQueue] = useState(Boolean(false));

  const handleDelete = (track: Track) => {
    removeTrack(track);
  };

  return (
    <div className="w-screen max-w-xs min-h-full max-h-full pb-2 px-2 rounded-md bg-slate-900 flex flex-col">
      <div className="flex-1 mx-2">
        <div className="flex flex-row gap-2 mt-4 items-center">
          <h1 className="flex-1">Past Queue</h1>{" "}
          <BiChevronDown
            className={`h-6 w-6 transition-all duration-300 ${
              showLastQueue ? "rotate-180" : ""
            }`}
            onClick={() => setShowLastQueue(!showLastQueue)}
          />
        </div>
        <ul
          className={`flex flex-col gap-2 transition-all duration-300 ${
            showLastQueue ? "h-min mt-2" : "h-0 overflow-hidden"
          }`}
        >
          {pastQueue.map((t) => (
            <li
              key={t.stream}
              className="flex flex-row bg-slate-800 rounded-md w-full items-center max-w-full p-2"
            >
              <div className="flex-1">
                <div className="line-clamp-1">{t.title}</div>
                <div>{t.artist}</div>
              </div>
              <button
                className="text-red-500 w-8 h-8 hover:bg-slate-900 hover:text-red-300 rounded-md transition-colors duration-300"
                onClick={(_) => handleDelete(t)}
              >
                <TbTrash className="h-6 w-6 ml-1" />
              </button>
            </li>
          ))}
        </ul>
        <h1 className="mt-4">Currently playing:</h1>
        <li className="flex flex-col bg-slate-800 rounded-md w-full max-w-full p-2">
          <div>{currentTrack?.title ?? "No track playing"}</div>
          <div>{currentTrack?.artist ?? ""}</div>
        </li>
        {queue.length > 0 && (
          <>
            <div className="flex flex-row gap-2 mt-4">
              <div className="flex-1">Next in Queue</div>
              <button
                onClick={clearQueue}
                className="text-red-500 rounded-md transition-all duration-300"
              >
                Clear queue
              </button>
            </div>
            <ul className="flex flex-col gap-2 mt-2">
              {queue.map((t) => (
                <li
                  key={t.stream}
                  className="flex flex-row bg-slate-800 rounded-md w-full items-center max-w-full p-2"
                >
                  <div className="flex-1">
                    <div className="line-clamp-1">{t.title}</div>
                    <div>{t.artist}</div>
                  </div>
                  <button
                    className="text-red-500 w-8 h-8 hover:bg-slate-900 hover:text-red-300 rounded-md transition-colors duration-300"
                    onClick={(h) => handleDelete(t)}
                  >
                    <TbTrash className="h-6 w-6 ml-1" />
                  </button>
                </li>
              ))}
            </ul>
          </>
        )}

        <div className="flex flex-row gap-2 mt-4">
          <div className="flex-1 line-clamp-1">Next from {currentContext?.display ?? "the current context"}</div>
        </div>
        <ul className="flex flex-col gap-2 mt-2">
          {currentContext?.tracks.map((t) => (
            <li
              key={t.stream}
              className="flex flex-row bg-slate-800 rounded-md w-full items-center max-w-full p-2"
            >
              <div className="flex-1">
                <div className="line-clamp-1">{t.title}</div>
                <div>{t.artist}</div>
              </div>
            </li>
          ))}
        </ul>
      </div>
      <div>
        <div className="bg-teal-950 mt-2 -mb-2 -mx-2 p-2 rounded-md">
          <DebugMenu />
        </div>
      </div>
    </div>
  );
}
