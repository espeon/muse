import { Play } from "lucide-react";
import { cn, displayArtists, formatTime } from "@/lib/utils";
import { LikeButton } from "@/components/LikeButton";
import type { Track } from "@/types";

export function TrackRow({
  track,
  index,
  isActive,
  onPlay,
}: {
  track: Track;
  index: number;
  isActive?: boolean;
  onPlay?: () => void;
}) {
  return (
    <div
      className={cn(
        "group flex h-11 select-none items-center gap-3 rounded-lg px-2 transition-colors hover:bg-white/5",
        isActive && "text-primary",
      )}
    >
      <button
        type="button"
        onClick={onPlay}
        className="flex h-6 w-6 items-center justify-center text-xs text-muted-foreground group-hover:text-foreground"
        aria-label={`Play ${track.name}`}
      >
        <span className="group-hover:hidden">{index + 1}</span>
        <Play size={14} className="hidden fill-current group-hover:block" />
      </button>
      <div className="min-w-0 flex-1">
        <div className={cn("truncate text-sm", isActive && "text-primary")}>
          {track.name}
        </div>
        <div className="truncate text-xs text-muted-foreground">
          {displayArtists({ artistName: track.artist_name, artists: track.artists })}
        </div>
      </div>
      <LikeButton
        trackId={track.id}
        initialLiked={track.liked}
        size={16}
        className="opacity-0 transition-opacity group-hover:opacity-100"
      />
      <span className="w-10 text-right text-xs tabular-nums text-muted-foreground">
        {formatTime(track.duration)}
      </span>
    </div>
  );
}
