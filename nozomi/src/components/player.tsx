"use client";
import React from "react";
import { PlayerType, usePlayerStore } from "../stores/playerStore";
import ReactPlayer from "react-player";
import { useQueueStore } from "@/stores/queueStore";
import { usePlayerRefStore } from "@/stores/playerRefStore";
import getTrackUrl from "@/helpers/getTrackUrl";

export default function Player() {
  // ref of player
  const playerRef = React.useRef<ReactPlayer>(null);
  const playerRef2 = React.useRef<ReactPlayer>(null);
  const {
    currentPlayerIs,
    isPlaying,
    isPlaying1,
    isPlaying2,
    media,
    media2,
    seek,
    isSeeking,
    volume,
    scrobbled,
    duration,
    muted,
    currentTime,
    setCurrentTime,
    setDuration,
    setMedia,
    setUpNextMedia,
    setPlaying,
    setScrobbled,
    isBuffering,
    setBuffering,
  } = usePlayerStore();

  const { popTrack, popPastTrack, currentTrack } = useQueueStore();

  const { setPlayerRef1, setPlayerRef2 } = usePlayerRefStore();

  React.useEffect(() => {
    if ("mediaSession" in navigator) {
      navigator.mediaSession.metadata = new window.MediaMetadata({
        title: currentTrack?.title ?? "Not playing",
        artist: currentTrack?.artist ?? "",
        album: currentTrack?.album ?? "",
        artwork: [
          {
            src: currentTrack?.artwork ?? "https://i.imgur.com/7vNnD9q.png",
            sizes: "512x512",
            type: "image/webp",
          },
        ],
      });
      navigator.mediaSession.setActionHandler("previoustrack", () => {
        {
          if (currentTime < 3) {
            popPastTrack();
          } else {
            setCurrentTime(0, true);
          }
        }
      });
      navigator.mediaSession.setActionHandler("nexttrack", () => {
        popTrack();
      });
      navigator.mediaSession.setActionHandler(
        "seekto",
        (a: MediaSessionActionDetails) => {
          if (a.seekTime) {
            setCurrentTime(a.seekTime, true);
          }
        },
      );
      navigator.mediaSession.setActionHandler("seekbackward", () => {
        setCurrentTime(currentTime - 10, true);
      });
      navigator.mediaSession.setActionHandler("seekforward", () => {
        setCurrentTime(currentTime + 10, true);
      });
      navigator.mediaSession.setActionHandler("play", () => {
        handlePlay();
      });
      navigator.mediaSession.setActionHandler("pause", () => {
        handlePause();
      });
    }
  }, [currentTrack]);

  React.useEffect(() => {
    setPlayerRef1(playerRef);

    return () => {
      setPlayerRef1(null);
    };
  }, [playerRef, setPlayerRef1]);

  React.useEffect(() => {
    setPlayerRef2(playerRef);

    return () => {
      setPlayerRef2(null);
    };
  }, [playerRef, setPlayerRef2]);

  const handleProgress = (
    state: { playedSeconds: number },
    player: PlayerType,
  ) => {
    if (player !== currentPlayerIs) {
      return;
    }
    // We only want to update time slider if we are not currently seeking
    if (!isSeeking) {
      setCurrentTime(state.playedSeconds, false);
    }
    // get 80% of duration for scrobbling
    if (state.playedSeconds > duration * 0.8 && !scrobbled) {
      console.log("scrobbling track");
      // api call to media url, get the track url and extract the id
      // url is in the form of "https://maki.lol/track/{id}/stream?blabla"
      let id =
        currentPlayerIs === PlayerType.PLAYER1
          ? media.url.split("/")[media.url.split("/").length - 2]
          : media2.url.split("/")[media2.url.split("/").length - 2];

      // api/authedReq?route with id
      fetch(`/api/authedReq?route=track/${id}/scrobble`)
        .then((res) => {
          console.log("scrobble response", res);
          setScrobbled();
        })
        .catch((err) => {
          console.log("scrobble error", err);
        });
    }
  };

  const handleDuration = (duration: number, player: PlayerType) => {
    console.log("onDuration", duration, "player: ", player);

    // if player is currently playing, set the duration
    if (currentPlayerIs === player) {
      setDuration(duration);
    }
  };

  const handleEndedTrack = () => {
    console.log("Track ended, queueing next track");
    let track = popTrack();
  };

  const handleReady = () => {
    // if no track is playing, check if there is a track in the queue
    if (!isPlaying && media.url === "") {
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

  const handleBuffer = (player: PlayerType) => {
    if (player === currentPlayerIs) {
      setBuffering(true);
    }
  };

  const handleBufferEnd = (player: PlayerType) => {
    if (player === currentPlayerIs) {
      setBuffering(false);
    }
  };

  const handleError = (err: Error, player: PlayerType) => {
    console.log("Error: ", err);
    if (player === currentPlayerIs) {
      setBuffering(false);
    }
    // Assume that the error is due to expired signed url so we should try getting a new one
    console.log("Error expired signed url, getting new url");
    let track = currentPlayerIs === PlayerType.PLAYER1 ? media : media2;
    getTrackUrl(track.track_id).then((url) => {
      if (player === PlayerType.PLAYER1) {
        usePlayerStore().media.url = url;
      } else {
        usePlayerStore().media2.url = url;
      }
    });
  };

  // other media keys

  return (
    <div>
      <ReactPlayer
        ref={playerRef}
        key={media.url}
        className="hidden"
        url={media.url}
        playing={!isPlaying1}
        onBuffer={() => handleBuffer(PlayerType.PLAYER1)}
        onBufferEnd={() => handleBufferEnd(PlayerType.PLAYER1)}
        volume={volume}
        muted={muted}
        onEnded={handleEndedTrack}
        onError={(err) => handleError(err, PlayerType.PLAYER1)}
        onProgress={(state) => handleProgress(state, PlayerType.PLAYER1)}
        onReady={handleReady}
        onDuration={(dur) => handleDuration(dur, PlayerType.PLAYER1)}
        onPause={handlePause}
        onPlay={handlePlay}
        progressInterval={1000}
      />

      <ReactPlayer
        ref={playerRef2}
        key={media2.url}
        className="hidden"
        url={media2.url}
        playing={!isPlaying2}
        onBuffer={() => handleBuffer(PlayerType.PLAYER2)}
        onBufferEnd={() => handleBufferEnd(PlayerType.PLAYER2)}
        volume={volume}
        muted={muted}
        onEnded={handleEndedTrack}
        onError={(err) => handleError(err, PlayerType.PLAYER2)}
        onProgress={(state) => handleProgress(state, PlayerType.PLAYER2)}
        onReady={handleReady}
        onDuration={(dur) => handleDuration(dur, PlayerType.PLAYER2)}
        onPause={handlePause}
        onPlay={handlePlay}
        progressInterval={1000}
      />
    </div>
  );
}
