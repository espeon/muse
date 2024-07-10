"use client";
import { Track, useQueueStore } from "@/stores/queueStore";
import { usePlayerStore } from "../stores/playerStore";
import { useRouteStore } from "@/stores/routeStore";

export default function DebugMenu() {
    const {setMedia, togglePlaying, isPlaying, media} = usePlayerStore();
    const {addTrack, currentTrack, queue, currentContext} = useQueueStore();

    const { history, future } = useRouteStore();
    const setPlaying = () => {
      // random number between 0 and 999
      const randomTrack = Math.floor(Math.random() * 999)
      // get track from api
      fetch(process.env.NEXT_PUBLIC_MAKI_BASE_URL + "/track/" + randomTrack)
        .then((response) => response.json())
        .then((data) => {
            // get album 
            fetch(process.env.NEXT_PUBLIC_MAKI_BASE_URL + "/album/" + data[0].album)
            .then((response) => response.json())
            .then((adata) => {
                const toAdd: Track = {
                  title: data[0].name,
                  artist: data[0].artist_name,
                  album: data[0].album_name,
                  artwork: process.env.NEXT_PUBLIC_MAKI_BASE_URL + "/art/" + adata.art[0],
                  stream: process.env.NEXT_PUBLIC_MAKI_BASE_URL + "/track/" + randomTrack + "/stream",
                }
                addTrack(toAdd)
                console.log("queued track: ", toAdd, data)
            });
        });

    };
  
    return (
        <>
        <div className="pb-2 text-xl">Debug menu</div>
        <div>
        <button className="bg-slate-800 px-2 py-1 hover:bg-slate-900 rounded-md transition-colors duration-300" onClick={() => setPlaying()}>Queue random track</button>
        <br />
        <div>Currently streaming from: <br />{media}</div>
        <br />
        <div>Current context: {currentContext?.type} {currentContext?.id} - {currentContext?.tracks.length}</div>
        <div> Current history: {history.join(",")}</div>
        <div> Current future: {future.join(",")}</div>
        </div>
      </>
    )
  }
  