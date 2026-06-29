import { Play } from "lucide-react";
import { cn } from "@/lib/utils";
import type { AlbumPartial } from "@/types";

/** Large feature card for the top of Home — Apple Music style hero banner
 *  with blurred backdrop, large title, and play button. */
export function HeroBanner({
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
      className={cn(
        "relative flex aspect-[16/7] w-full flex-col justify-end overflow-hidden rounded-2xl p-5 text-left md:p-8",
        className,
      )}
    >
      {/* blurred backdrop */}
      {art && (
        <>
          <img
            src={art}
            alt=""
            aria-hidden
            className="absolute inset-0 h-full w-full scale-110 object-cover blur-2xl brightness-50"
          />
          <div className="absolute inset-0 bg-gradient-to-t from-black/80 via-black/40 to-transparent" />
        </>
      )}

      {/* foreground art */}
      <div className="relative flex items-end gap-4">
        {art && (
          <img
            src={art}
            alt=""
            className="hidden h-28 w-28 shrink-0 rounded-lg object-cover shadow-2xl sm:block md:h-36 md:w-36"
          />
        )}
        <div className="min-w-0 flex-1">
          <p className="text-xs font-semibold uppercase tracking-wide text-white/60">
            New Release
          </p>
          <h2 className="mt-1 truncate text-2xl font-bold text-white md:text-4xl">
            {album.name}
          </h2>
          <p className="mt-1 truncate text-sm text-white/70 md:text-base">
            {album.artist?.name ?? "—"}
          </p>
          {onPlay && (
            <span
              onClick={(e) => {
                e.stopPropagation();
                onPlay();
              }}
              className="mt-3 inline-flex items-center gap-2 rounded-full bg-white px-5 py-2 text-sm font-semibold text-black transition-opacity hover:opacity-90"
            >
              <Play size={16} className="fill-current" />
              Play
            </span>
          )}
        </div>
      </div>
    </button>
  );
}
