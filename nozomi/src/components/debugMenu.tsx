"use client";
import { Track, useQueueStore } from "@/stores/queueStore";
import { usePlayerStore } from "../stores/playerStore";
import { useRouteStore } from "@/stores/routeStore";
import { useContext, useState } from "react";
import { useConfig } from "@/stores/configStore";

export default function DebugMenu() {
  const { media, scrobbled, media2, currentPlayerIs } = usePlayerStore();
  const { currentContext } = useQueueStore();
  const { externalMakiBaseURL, umiBaseURL } = useConfig();

  const { history, future } = useRouteStore();

  return (
    <>
      <div className="pb-2 text-xl">Debug menu</div>
      <div>
        <div>Currently streaming from: {currentPlayerIs}</div>
        <div>
          In player 1: <br />
          {media} {scrobbled ? "(scrobbled)" : "x"}
        </div>
        <div>In player 2: {media2 ?? "None"}</div>
        <br />
        <div>
          Player context: {currentContext?.type} {currentContext?.id} -{" "}
          {currentContext?.tracks.length}
        </div>
        {/* get last three history elements */}
        <div> Current history: {history.slice(-3).join(",")}</div>
        {future.length > 0 && <div> Current future: {future.join(",")}</div>}
      </div>
      <div>
        maki: {externalMakiBaseURL}
        <br />
        umi: {umiBaseURL}
      </div>
    </>
  );
}
