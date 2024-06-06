"use client";
import React from "react";
import { usePlayerStore } from "../stores/playerStore";
import ReactPlayer from "react-player";
import { useQueueStore } from "@/stores/queueStore";
import { usePlayerRefStore } from "@/stores/playerRefStore";

export default function Player() {
  // ref of player
  const playerRef = React.useRef<ReactPlayer>(null);
  const {
    isPlaying,
    media,
    seek,
    isSeeking,
    volume,
    scrobbled,
    duration,
    muted,
    setCurrentTime,
    setDuration,
    setMedia,
    setPlaying,
    setScrobbled,
  } = usePlayerStore();

  const { popTrack, currentTrack } = useQueueStore();

  const { setPlayerRef } = usePlayerRefStore();

  React.useEffect(() => {
    console.log("Player has initialised");
  }, []);

  React.useEffect(() => {
    
    setPlayerRef(playerRef);

    return () => {
      setPlayerRef(null);
    };
  }, [playerRef, setPlayerRef]);

  const handleProgress = (state: { playedSeconds: number }) => {
    // We only want to update time slider if we are not currently seeking
    if (!isSeeking) {
      setCurrentTime(state.playedSeconds, false);
    }
    // get 80% of duration for scrobbling
    if (state.playedSeconds > duration * 0.8 && !scrobbled) {
      console.log("scrobbling track");
      // api call to media url, replace "stream" with "scrobble"
      fetch(media.replace("stream", "scrobble")).then((res) => {
        console.log("scrobble response", res);
        setScrobbled();
      }).catch((err) => {
        console.log("scrobble error", err);
      })
    }
  };

  const handleDuration = (duration: number) => {
    console.log("onDuration", duration);

    setDuration(duration);
  };

  const handleEndedTrack = () => {
    console.log("Track ended, queueing next track");
    let track = popTrack();
  };

  const handleReady = () => {
    // if no track is playing, check if there is a track in the queue
    if (!isPlaying && media === "") {
      let track = popTrack();
      if (track) {
        setMedia(track.stream);
      }
    }
  };

  const handlePause = () => {
    setPlaying(true);
  };

  const handlePlay = () => {
    setPlaying(false);
  };

  return (
    <div>
      <ReactPlayer
        ref={playerRef}
        className="hidden"
        url={media}
        playing={!isPlaying}
        volume={volume}
        muted={muted}
        onEnded={handleEndedTrack}
        onProgress={handleProgress}
        onReady={handleReady}
        onDuration={handleDuration}
        onPause={handlePause}
        onPlay={handlePlay}
        progressInterval={100}
      />
    </div>
  );
}
