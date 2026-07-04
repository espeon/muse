import {
  FastForward,
  ListMusic,
  Maximize2,
  Pause,
  Play,
  Repeat,
  Shuffle,
  Volume1,
  Volume2,
  VolumeX,
} from "lucide-react";
import { Glass } from "@samasante/liquid-glass";
import { QueueSheet } from "@/components/QueueSheet";
import { usePlayer } from "@/player/use-player";
import { cn } from "@/lib/utils";
import { useLayoutEffect, useRef, useState } from "react";
import { MarqueeText } from "@/components/MarqueeText";
import { Seekbar } from "@/components/Seekbar";
import { useNavigate } from "@tanstack/react-router";

const iconBtn =
  "inline-flex h-7 w-7 items-center justify-center rounded-full text-foreground/80 transition-colors hover:bg-accent hover:text-foreground disabled:opacity-40";

function VolumeControl({
  volume,
  onVolume,
}: {
  volume: number;
  onVolume: (v: number) => void;
}) {
  const Icon = volume === 0 ? VolumeX : volume < 0.5 ? Volume1 : Volume2;
  return (
    <div className="group/vol relative z-10 flex items-center lg:flex">
      {/* slider: absolute so it overlays the buttons to the left without
          pushing layout. Still a child of group/vol so hover is continuous
          from button → slider. */}
      <div className="pointer-events-none absolute right-6 top-1/2 flex h-6 -translate-y-1/2 items-center justify-end overflow-hidden rounded-l-full bg-card/90 pl-2.5 pr-1 opacity-0 shadow-lg shadow-black/30 backdrop-blur-sm transition-all duration-200 group-hover/vol:pointer-events-auto group-hover/vol:opacity-100 w-0 group-hover/vol:w-24">
        <input
          type="range"
          min={0}
          max={1}
          step={0.01}
          value={volume}
          onChange={(e) => onVolume(Number(e.target.value))}
          className="h-1 w-full cursor-pointer accent-primary"
          aria-label="Volume"
        />
      </div>
      <button
        type="button"
        className={cn(
          iconBtn,
          "h-6 w-6 shrink-0 rounded-r-full rounded-l-none group-hover/vol:bg-card/90",
        )}
        onClick={() => onVolume(volume === 0 ? 1 : 0)}
        aria-label={volume === 0 ? "Unmute" : "Mute"}
      >
        <Icon size={14} />
      </button>
    </div>
  );
}

export function PlayerBar({ onOpenTheatre }: { onOpenTheatre?: () => void }) {
  const player = usePlayer();
  const cur = player.current;
  const navigate = useNavigate();
  const [repeat, setRepeat] = useState(false);

  // in PlayerBar, near the top
  const midRef = useRef<HTMLButtonElement>(null);
  const [mid, setMid] = useState<{ left: number; width: number } | null>(null);

  useLayoutEffect(() => {
    const el = midRef.current;
    if (!el) return;
    const sync = () => {
      const wrap = el.offsetParent as HTMLElement | null; // the relative wrapper
      if (!wrap) return;
      const a = el.getBoundingClientRect();
      const b = wrap.getBoundingClientRect();
      setMid({ left: a.left - b.left, width: a.width });
    };
    sync();
    const ro = new ResizeObserver(sync);
    ro.observe(el);
    window.addEventListener("resize", sync);
    return () => {
      ro.disconnect();
      window.removeEventListener("resize", sync);
    };
  }, [midRef.current]);

  if (!cur) return null;

  return (
    <div className="fixed bottom-0 right-0 z-50 px-2 pb-2 md:left-60">
      <div className="relative mx-auto max-w-2xl">
        <Glass
          className="flex! items-center gap-2 rounded-2xl border border-white/10 px-3 py-1.5"
          style={{
            borderRadius: 999999,
            backgroundColor: "rgba(80,80,80, 0.25)",
          }}
          optics={{
            mapSize: 512,
            clipToShape: true,
            softEdge: true,
            strength: 0.18,
            depth: 0.2,
            curvature: 0.55,
            bend: 0.25,
            bendWidth: 0.08,
            dispersion: 0.15,
            specular: 0.8,
            sheenAngle: 50,
            glow: 0.15,
            glowSpread: 1,
            glowFalloff: 1.5,
            sheen: 0.95,
            sheenWidth: 2,
            sheenFalloff: 1.5,
            frost: 6,
            brightness: 0.15,
          }}
        >
          {/* transport: shuffle · prev · play/pause · next · repeat */}
          <div className="flex items-center gap-0.5">
            <button
              type="button"
              className={cn(iconBtn, player.shuffle && "text-primary")}
              onClick={() => player.setShuffle(!player.shuffle)}
              aria-label="Shuffle"
              aria-pressed={player.shuffle}
            >
              <Shuffle size={13} />
            </button>
            <button
              type="button"
              className={iconBtn}
              onClick={() => void player.previous()}
              aria-label="Previous"
            >
              <FastForward
                size={18}
                fill="currentColor"
                className="rotate-180"
              />
            </button>
            <button
              type="button"
              className={iconBtn}
              onClick={() => player.toggle()}
              aria-label={player.isPlaying ? "Pause" : "Play"}
            >
              {player.isPlaying ? (
                <Pause size={24} fill="currentColor" />
              ) : (
                <Play size={24} className="ml-0.5" fill="currentColor" />
              )}
            </button>
            <button
              type="button"
              className={iconBtn}
              onClick={() => void player.next()}
              aria-label="Next"
            >
              <FastForward size={18} fill="currentColor" />
            </button>
            <button
              type="button"
              className={cn(iconBtn, repeat && "text-primary")}
              onClick={() => setRepeat((r) => !r)}
              aria-label="Repeat"
              aria-pressed={repeat}
            >
              <Repeat size={13} />
            </button>
          </div>

          {/* album info + seekbar (seekbar sits as the bottom border of this item) */}
          <button
            type="button"
            ref={midRef}
            className="relative min-w-0 flex-1 text-left"
          >
            <div className="flex items-center gap-2 pb-1">
              <div
                className="flex h-8 w-8 items-center justify-center rounded-md bg-accent shadow-lg shadow-black/20 group cursor-pointer "
                onClick={onOpenTheatre}
              >
                {cur.artUrl ? (
                  <img
                    src={cur.artUrl}
                    alt=""
                    className="h-8 w-8 shrink-0 rounded-md object-cover shadow-lg"
                  />
                ) : (
                  <div className="h-8 w-8 shrink-0 rounded-md bg-accent" />
                )}
                <Maximize2 className="absolute mix-blend-difference shadow-sm shadow-neutral-500/50 opacity-0 group-hover:opacity-100 duration-250 transition-opacity" />
              </div>
              <div className="min-w-0 flex-1">
                <div className="flex items-baseline justify-between gap-2">
                  <div className="min-w-0">
                    <MarqueeText
                      text={cur.title}
                      className="text-xs font-semibold"
                    />
                    <div className="truncate text-[11px] text-foreground/75">
                      {cur.artists && cur.artists.length > 1 ? (
                        <>
                          {cur.artists.map((artist, i) => (
                            <span key={artist.id}>
                              <button
                                type="button"
                                onClick={(e) => {
                                  e.stopPropagation();
                                  void navigate({ to: "/artist/$artistId", params: { artistId: String(artist.id) } });
                                }}
                                className="hover:text-foreground hover:underline"
                              >
                                {artist.name}
                              </button>
                              {i < cur.artists!.length - 1 && <span>, </span>}
                            </span>
                          ))}
                        </>
                      ) : (
                        <button
                          type="button"
                          onClick={(e) => {
                            e.stopPropagation();
                            const artistId = cur.artists?.[0]?.id ?? cur.albumArtistId;
                            if (artistId) void navigate({ to: "/artist/$artistId", params: { artistId: String(artistId) } });
                          }}
                          className="hover:text-foreground hover:underline"
                        >
                          {cur.artistName}
                        </button>
                      )}
                      <span> — </span>
                      <button
                        type="button"
                        onClick={(e) => {
                          e.stopPropagation();
                          if (cur.albumId) void navigate({ to: "/album/$albumId", params: { albumId: String(cur.albumId) } });
                        }}
                        className="hover:text-foreground hover:underline"
                      >
                        {cur.albumName}
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </button>

          {/* kebab · queue · volume */}
          <div className="flex items-center gap-0.5">
            <QueueSheet>
              <button type="button" className={iconBtn} aria-label="Queue">
                <ListMusic size={15} />
              </button>
            </QueueSheet>
            <VolumeControl
              volume={player.volume}
              onVolume={(v) => player.setVolume(v)}
            />
          </div>
        </Glass>
        {/* overlay — OUTSIDE Glass, inside the shared relative wrapper */}
        {mid && (
          <div
            className="absolute bottom-0 h-1.5"
            style={{ left: mid.left, width: mid.width }}
          >
            <Seekbar
              currentTime={player.currentTime}
              duration={player.duration}
              bufferedTime={player.bufferedTime}
              onSeek={(t) => player.seek(t)}
            />
          </div>
        )}
      </div>
    </div>
  );
}
