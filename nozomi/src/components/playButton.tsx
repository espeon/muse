"use client";
import { Context, Track, useQueueStore } from "@/stores/queueStore";
import { PiPlayFill } from "react-icons/pi";

export default function PlayButton({track, ctx, children, ...props}: {track: Track, ctx?: Context, children?: React.ReactNode}) {

    const handlePlay = (track: Track, context?: Context) => {
        useQueueStore.getState().playTrack(track);

        if(context !== undefined)   {
            // find the current track
            let i = context.tracks.findIndex(ftrack => ftrack.stream === track.stream)
            // get rid of everything before and including the current track
            
            let cTracks = context.tracks.slice(i + 1, context.tracks.length)
            const pastTracks = context.tracks.slice(0, i)
            useQueueStore.getState().setContext({ type: context.type, id: context.id, tracks: cTracks });
            for (let i = 0; i < pastTracks.length; ++i) {
                useQueueStore.getState().addTrackPast(pastTracks[i])
            }
        }

    };

    return (
        <button
            onClick={() => {
                handlePlay(track, ctx);
            }}
            {...props}
        >
            {children ?? <PiPlayFill />}
        </button>
    );
}