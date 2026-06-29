import { Play } from "lucide-react";
import { cn } from "@/lib/utils";
import type { AlbumPartial } from "@/types";

export function AlbumCard({
  album,
  onClick,
  onPlay,
  className,
}: {
  album: AlbumPartial;
  onClick?: () => void;
  onPlay?: () => void;
  className?: string;
}) {
  const art = album.art?.[0];
  return (
    <button
      type="button"
      onClick={onClick}
      title={album.name}
      className={cn(
        "group flex w-full flex-col items-start gap-2 text-left",
        className,
      )}
    >
      <div className="relative aspect-square w-full overflow-hidden rounded-xl bg-accent shadow-lg shadow-black/20 transition-shadow group-hover:shadow-xl group-hover:shadow-black/30">
        {art ? (
          <img
            src={art}
            alt=""
            loading="lazy"
            className="h-full w-full object-cover transition-transform duration-300 group-hover:scale-105"
          />
        ) : null}
        {onPlay && (
          <span
            className="absolute bottom-2 right-2 flex h-10 w-10 items-center justify-center rounded-full bg-primary text-primary-foreground opacity-0 shadow-lg transition-all duration-200 group-hover:translate-y-0 group-hover:opacity-100 translate-y-1"
            onClick={(e) => {
              e.stopPropagation();
              onPlay();
            }}
          >
            <Play size={16} className="ml-0.5 fill-current" />
          </span>
        )}
      </div>
      <div className="w-full min-w-0">
        <div className="truncate text-sm font-semibold">{album.name}</div>
        <div className="truncate text-xs text-muted-foreground">
          {album.artist?.name ?? "—"}
        </div>
      </div>
    </button>
  );
}
