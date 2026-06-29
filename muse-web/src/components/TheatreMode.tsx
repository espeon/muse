import { useEffect, useState } from "react";
import {
  FastForward,
  ListMusic,
  Mic2,
  Pause,
  Play,
  Repeat,
  Shuffle,
  Volume1,
  Volume2,
  VolumeX,
  X,
} from "lucide-react";
import { QueueSheet } from "@/components/QueueSheet";
import { LyricsView } from "@/components/LyricsView";
import { LikeButton } from "@/components/LikeButton";
import { MarqueeText } from "@/components/MarqueeText";
import { fetchLyrics } from "@/lib/api";
import { cn, formatTime } from "@/lib/utils";
import { usePlayer } from "@/player/use-player";
import { Seekbar } from "@/components/Seekbar";
import type { Jlf } from "@/lyrics/jlf";

const iconBtn =
  "inline-flex h-12 w-12 items-center justify-center rounded-full text-foreground/90 transition-colors hover:bg-white/10 hover:text-foreground";

function VolumeControl({
  volume,
  onVolume,
}: {
  volume: number;
  onVolume: (v: number) => void;
}) {
  const Icon = volume === 0 ? VolumeX : volume < 0.5 ? Volume1 : Volume2;
  return (
    <div className="group/vol flex items-center gap-2">
      <button
        type="button"
        className="inline-flex h-9 w-9 items-center justify-center rounded-full text-foreground/70 transition-colors hover:bg-white/10 hover:text-foreground"
        onClick={() => onVolume(volume === 0 ? 1 : 0)}
        aria-label={volume === 0 ? "Unmute" : "Mute"}
      >
        <Icon size={18} />
      </button>
      <input
        type="range"
        min={0}
        max={1}
        step={0.01}
        value={volume}
        onChange={(e) => onVolume(Number(e.target.value))}
        className="h-1 w-28 cursor-pointer accent-primary"
        aria-label="Volume"
      />
    </div>
  );
}

export function TheatreMode({ onClose }: { onClose: () => void }) {
  const player = usePlayer();
  const cur = player.current;

  const [showLyrics, setShowLyrics] = useState(false);
  const [lyrics, setLyrics] = useState<Jlf | null>(null);
  const [lyricsLoading, setLyricsLoading] = useState(false);
  const [lyricsNotFound, setLyricsNotFound] = useState(false);

  // Fetch lyrics whenever the current track changes
  useEffect(() => {
    if (!cur) {
      setLyrics(null);
      setLyricsNotFound(false);
      return;
    }
    let cancelled = false;
    setLyrics(null);
    setLyricsNotFound(false);
    setLyricsLoading(true);
    fetchLyrics(cur.title, cur.artistName, cur.albumName).then((result) => {
      if (cancelled) return;
      setLyrics(result);
      setLyricsLoading(false);
      setLyricsNotFound(result === null);
    });
    return () => {
      cancelled = true;
    };
  }, [cur?.id]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        if (showLyrics) {
          setShowLyrics(false);
          return;
        }
        onClose();
      }
      if (e.key === " ") {
        e.preventDefault();
        player.toggle();
      }
      if (e.key === "ArrowRight") void player.next();
      if (e.key === "ArrowLeft") void player.previous();
      if (e.key === "l" || e.key === "L") setShowLyrics((s) => !s);
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [onClose, player, showLyrics]);

  if (!cur) return null;

  const canShowLyrics =
    showLyrics && (lyrics || lyricsLoading || lyricsNotFound);

  return (
    <div className="fixed inset-0 z-[60] flex flex-col overflow-hidden bg-background">
      {/* blurred background */}
      {cur.artUrl ? (
        <>
          <img
            src={cur.artUrl}
            alt=""
            aria-hidden
            className="absolute inset-0 h-full w-full scale-110 object-cover blur-3xl brightness-[0.25] saturate-150"
          />
          <div className="absolute inset-0 bg-linear-to-b from-black/30 via-black/50 to-black/80" />
        </>
      ) : (
        <div className="absolute inset-0 bg-linear-to-br from-card via-background to-accent" />
      )}

      {/* top bar */}
      <div className="relative flex items-center justify-between px-4 py-3 md:px-6">
        <button
          type="button"
          onClick={onClose}
          className="inline-flex h-10 w-10 items-center justify-center rounded-full text-foreground/80 transition-colors hover:bg-white/10"
          aria-label="Close theatre mode"
        >
          <X size={22} />
        </button>
        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={() => setShowLyrics((s) => !s)}
            className={cn(
              "inline-flex h-10 w-10 items-center justify-center rounded-full transition-colors hover:bg-white/10",
              showLyrics ? "text-primary" : "text-foreground/80",
            )}
            aria-label="Toggle lyrics"
            aria-pressed={showLyrics}
            title="Lyrics (L)"
          >
            <Mic2 size={20} />
          </button>
          <QueueSheet>
            <button
              type="button"
              className="inline-flex h-10 w-10 items-center justify-center rounded-full text-foreground/80 transition-colors hover:bg-white/10"
              aria-label="Queue"
            >
              <ListMusic size={22} />
            </button>
          </QueueSheet>
        </div>
      </div>

      {/* ── Apple Music-style lyrics view ── */}
      {canShowLyrics ? (
        <div className="relative flex flex-1 flex-col overflow-hidden px-4 pb-4 md:px-6">
          {/* art + info  |  lyrics */}
          <div className="flex flex-1 flex-col gap-4 overflow-hidden lg:flex-row lg:gap-10">
            {/* left: art + track info */}
            <div className="flex flex-col items-center justify-center gap-3 lg:w-2/5 lg:shrink-0 lg:items-end lg:justify-center lg:pr-12 lg:gap-4">
              {cur.artUrl ? (
                <img
                  src={cur.artUrl}
                  alt=""
                  className="aspect-square w-full max-w-32 rounded-xl object-cover shadow-2xl shadow-black/50 lg:max-w-140"
                />
              ) : (
                <div className="aspect-square w-full max-w-32 rounded-xl bg-accent lg:max-w-140" />
              )}
              <div className="flex w-full max-w-80 flex-col items-center gap-1 lg:max-w-140 lg:items-end">
                <MarqueeText
                  text={cur.title}
                  className="w-full text-center text-lg font-bold lg:text-left lg:text-4xl"
                />
                <MarqueeText
                  text={cur.artistName}
                  className="w-full text-center text-sm text-white/60 lg:text-left lg:text-3xl"
                />
                <MarqueeText
                  text={cur.albumName}
                  className="w-full text-center text-xs text-white/40 lg:text-left lg:text-3xl"
                />
              </div>
            </div>

            {/* right: lyrics */}
            <div className="flex min-h-0 flex-1 overflow-hidden lg:w-3/5">
              {lyricsLoading ? (
                <div className="flex h-full items-center justify-center">
                  <p className="animate-pulse text-white/50">Loading lyrics…</p>
                </div>
              ) : lyricsNotFound ? (
                <div className="flex h-full items-center justify-center">
                  <p className="text-white/60">No lyrics found</p>
                </div>
              ) : lyrics ? (
                <LyricsView
                  jlf={lyrics}
                  currentTime={player.currentTime}
                  onSeek={(timeMs) => player.seek(timeMs / 1000)}
                />
              ) : null}
            </div>
          </div>

          {/* bottom: minimal controls */}
          <div className="flex flex-col gap-2 pt-2">
            <Seekbar
              currentTime={player.currentTime}
              duration={player.duration}
              bufferedTime={player.bufferedTime}
              onSeek={(t) => player.seek(t)}
            />
            <div className="flex items-center justify-between px-2 lg:px-4">
              <span className="w-20 text-xs font-mono tabular-nums text-white/50 lg:w-28 lg:text-sm">
                {formatTime(player.currentTime)} / {formatTime(player.duration)}
              </span>
              <div className="flex items-center gap-4 lg:gap-6">
                <button
                  type="button"
                  className={cn(iconBtn, "h-10 w-10")}
                  onClick={() => void player.previous()}
                  aria-label="Previous"
                >
                  <FastForward
                    size={20}
                    fill="currentColor"
                    className="rotate-180"
                  />
                </button>
                <button
                  type="button"
                  className={cn(iconBtn, "h-14 w-14")}
                  onClick={() => player.toggle()}
                  aria-label={player.isPlaying ? "Pause" : "Play"}
                >
                  {player.isPlaying ? (
                    <Pause size={28} fill="currentColor" />
                  ) : (
                    <Play size={28} className="ml-0.5" fill="currentColor" />
                  )}
                </button>
                <button
                  type="button"
                  className={cn(iconBtn, "h-10 w-10")}
                  onClick={() => void player.next()}
                  aria-label="Next"
                >
                  <FastForward size={20} fill="currentColor" />
                </button>
              </div>
              <div className="flex w-12 justify-end">
                <LikeButton trackId={cur.id} size={18} />
              </div>
            </div>
          </div>
        </div>
      ) : (
        /* ── default player view (no lyrics) ── */
        <div className="relative flex flex-1 flex-col items-center justify-center gap-6 px-6 pb-8 md:gap-8 md:pb-12">
          {/* large art */}
          {cur.artUrl ? (
            <img
              src={cur.artUrl}
              alt=""
              className="aspect-square w-auto max-h-[45vh] rounded-2xl object-cover shadow-2xl shadow-black/50 md:max-h-[50vh]"
            />
          ) : (
            <div className="aspect-square w-auto max-h-[45vh] rounded-2xl bg-accent md:max-h-[50vh]" />
          )}

          {/* info */}
          <div className="flex w-full max-w-md flex-col items-center text-center">
            <MarqueeText
              text={cur.title}
              className="w-full text-2xl font-bold md:text-3xl"
            />
            <MarqueeText
              text={cur.artistName}
              className="mt-1 w-full text-base text-foreground/75 md:text-lg"
            />
            <MarqueeText
              text={cur.albumName}
              className="w-full text-sm text-muted-foreground md:text-base"
            />
          </div>

          {/* seekbar */}
          <div className="flex w-full max-w-xl flex-col gap-2">
            <Seekbar
              currentTime={player.currentTime}
              duration={player.duration}
              bufferedTime={player.bufferedTime}
              onSeek={(t) => player.seek(t)}
            />
            <div className="flex justify-between text-xs tabular-nums text-muted-foreground">
              <span>{formatTime(player.currentTime)}</span>
              <span>{formatTime(player.duration)}</span>
            </div>
          </div>

          {/* controls */}
          <div className="flex items-center gap-4 md:gap-6">
            <button
              type="button"
              className={cn(
                iconBtn,
                "h-10 w-10",
                player.shuffle && "text-primary",
              )}
              onClick={() => player.setShuffle(!player.shuffle)}
              aria-label="Shuffle"
            >
              <Shuffle size={18} />
            </button>
            <button
              type="button"
              className={iconBtn}
              onClick={() => void player.previous()}
              aria-label="Previous"
            >
              <FastForward
                size={22}
                fill="currentColor"
                className="rotate-180"
              />
            </button>
            <button
              type="button"
              className={cn(iconBtn, "h-16 w-16")}
              onClick={() => player.toggle()}
              aria-label={player.isPlaying ? "Pause" : "Play"}
            >
              {player.isPlaying ? (
                <Pause size={32} fill="currentColor" />
              ) : (
                <Play size={32} className="ml-1" fill="currentColor" />
              )}
            </button>
            <button
              type="button"
              className={iconBtn}
              onClick={() => void player.next()}
              aria-label="Next"
            >
              <FastForward size={22} fill="currentColor" />
            </button>
            <button
              type="button"
              className={cn(iconBtn, "h-10 w-10", false && "text-primary")}
              aria-label="Repeat"
            >
              <Repeat size={18} />
            </button>
            <div className="flex h-12 w-12 items-center justify-center">
              <LikeButton trackId={cur.id} size={20} />
            </div>
          </div>

          {/* volume */}
          <VolumeControl
            volume={player.volume}
            onVolume={(v) => player.setVolume(v)}
          />
        </div>
      )}
    </div>
  );
}
