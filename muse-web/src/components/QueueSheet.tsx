import { useState } from "react";
import {
  Sheet,
  SheetContent,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import { cn, formatTime } from "@/lib/utils";
import { usePlayer } from "@/player/use-player";
import { ListMusic, X } from "lucide-react";

export function QueueSheet({ children }: { children: React.ReactNode }) {
  const [open, setOpen] = useState(false);
  const player = usePlayer();
  const current = player.currentIndex;

  return (
    <Sheet open={open} onOpenChange={setOpen}>
      <button
        type="button"
        onClick={() => setOpen(true)}
        className="inline-flex items-center justify-center"
        aria-label="Queue"
      >
        {children}
      </button>
      <SheetContent side="right" className="w-full p-0 sm:max-w-sm">
        <SheetHeader className="border-b border-border p-4 pb-3">
          <div className="flex items-center justify-between">
            <SheetTitle className="text-base font-semibold">Queue</SheetTitle>
            <button
              type="button"
              onClick={() => setOpen(false)}
              className="inline-flex h-8 w-8 items-center justify-center rounded-full text-muted-foreground transition-colors hover:bg-accent hover:text-foreground"
              aria-label="Close queue"
            >
              <X size={18} />
            </button>
          </div>
          <p className="text-xs text-muted-foreground">
            {player.length} {player.length === 1 ? "track" : "tracks"}
          </p>
        </SheetHeader>

        <div className="flex flex-col overflow-y-auto px-2 py-2">
          {player.queue.length === 0 ? (
            <div className="flex flex-col items-center justify-center gap-3 py-16 text-center text-sm text-muted-foreground">
              <ListMusic size={32} strokeWidth={1.5} />
              <p>Nothing in the queue yet.</p>
            </div>
          ) : (
            player.queue.map((track, index) => {
              const isCurrent = index === current;
              return (
                <button
                  key={`${track.id}-${index}`}
                  type="button"
                  onClick={() => {
                    player.jumpTo(index);
                    setOpen(false);
                  }}
                  className={cn(
                    "group flex items-center gap-3 rounded-lg px-2 py-2 text-left transition-colors",
                    isCurrent ? "bg-white/10" : "hover:bg-white/5",
                  )}
                >
                  <span
                    className={cn(
                      "flex h-6 w-6 items-center justify-center text-xs",
                      isCurrent ? "text-primary" : "text-muted-foreground",
                    )}
                  >
                    {isCurrent ? "▶" : index + 1}
                  </span>
                  {track.artUrl ? (
                    <img
                      src={track.artUrl}
                      alt=""
                      className="h-9 w-9 shrink-0 rounded object-cover"
                    />
                  ) : (
                    <div className="h-9 w-9 shrink-0 rounded bg-accent" />
                  )}
                  <div className="min-w-0 flex-1">
                    <div
                      className={cn(
                        "truncate text-sm",
                        isCurrent && "font-semibold text-primary",
                      )}
                    >
                      {track.title}
                    </div>
                    <div className="truncate text-xs text-muted-foreground">
                      {track.artistName}
                    </div>
                  </div>
                  <span className="text-xs tabular-nums text-muted-foreground">
                    {formatTime(track.duration)}
                  </span>
                </button>
              );
            })
          )}
        </div>
      </SheetContent>
    </Sheet>
  );
}
