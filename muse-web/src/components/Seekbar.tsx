import { useRef, useState } from "react";
import { cn } from "@/lib/utils";

/** progressive blur config. layers overlap heavily (each trapezoid is offset
 *  by one STEP but spans three), so neighbouring blur radii interpolate
 *  smoothly instead of butting into a visible seam. */
const BLUR_LAYERS = 24;
const STEP = 100 / (BLUR_LAYERS + 2); // mask offset per layer, in %
const MAX_BLUR = 4; // px, heaviest layer (nearest the bar)

const blurLayers = Array.from({ length: BLUR_LAYERS }, (_, i) => {
  // cos-eased ramp: gentle near the bar, so the strong-blur band reads smooth
  const t = (i + 1) / BLUR_LAYERS;
  const blur = +(Math.sin(t * Math.PI) * MAX_BLUR).toFixed(2);
  // overlapping trapezoid: transparent -> opaque -> opaque -> transparent,
  // shifted down one STEP per layer. the overlap is what kills the seam.
  const mask =
    `linear-gradient(to bottom,` +
    ` transparent ${i * STEP}%,` +
    ` #000 ${(i + 1) * STEP}%,` +
    ` #000 ${(i + 2) * STEP}%,` +
    ` transparent ${(i + 3) * STEP}%)`;
  return { blur, mask };
});

/** Apple Music-style seekbar: a thin progress line that doubles as the bottom
 *  border of the center item. On hover/drag it expands upward into a blur-backed
 *  bar with a draggable thumb. */
export function Seekbar({
  currentTime,
  duration,
  onSeek,
  bufferedTime,
}: {
  currentTime: number;
  duration: number;
  onSeek: (time: number) => void;
  bufferedTime?: number;
}) {
  const pct = duration > 0 ? Math.min(currentTime / duration, 1) : 0;
  const bufferedPct =
    bufferedTime != null && duration > 0
      ? Math.min(bufferedTime / duration, 1)
      : 0;
  // Only show the buffered fill when loading is still in progress —
  // once fully loaded (buffered >= duration), the grey fill is meaningless.
  const showBuffered = bufferedPct > 0 && bufferedPct < 0.99;
  const barRef = useRef<HTMLDivElement>(null);
  const [hovered, setHovered] = useState(false);
  const [dragging, setDragging] = useState(false);
  const active = hovered || dragging;

  const formattedCurrentTime = new Date(currentTime * 1000)
    .toISOString()
    .substring(14, 19);
  const formattedDuration = new Date(duration * 1000)
    .toISOString()
    .substring(14, 19);

  const seekFromX = (clientX: number) => {
    const el = barRef.current;
    if (!el || duration <= 0) return;
    const rect = el.getBoundingClientRect();
    const ratio = Math.max(0, Math.min(1, (clientX - rect.left) / rect.width));
    onSeek(ratio * duration);
  };

  return (
    <div className="relative w-full">
      <div
        ref={barRef}
        className={cn(
          "absolute inset-0 cursor-pointer touch-none transition-all duration-200 z-10",
          active ? "h-6 -top-6" : "h-1  -top-1",
        )}
        onPointerDown={(e) => {
          e.currentTarget.setPointerCapture(e.pointerId);
          setDragging(true);
          seekFromX(e.clientX);
        }}
        onPointerMove={(e) => {
          if (dragging) seekFromX(e.clientX);
        }}
        onPointerUp={(e) => {
          e.currentTarget.releasePointerCapture(e.pointerId);
          setDragging(false);
        }}
        onPointerCancel={() => setDragging(false)}
        onMouseEnter={() => setHovered(true)}
        onMouseLeave={() => setHovered(false)}
      ></div>
      {/* progressive blur backdrop. rendered first so the track paints on top
       *  and stays crisp. each layer cross-fades its own opacity (animating an
       *  ancestor's opacity would flatten these into one layer and break
       *  backdrop-filter mid-transition). pointer-events-none so it never
       *  steals the drag. */}
      <div
        aria-hidden
        className="pointer-events-none absolute inset-x-0 bottom-0 h-10 -mx-4"
      >
        {blurLayers.map((l, i) => (
          <div
            key={i}
            className="absolute inset-0 transition-opacity duration-200"
            style={{
              opacity: active ? 1 : 0,
              backdropFilter: `blur(${l.blur}px)`,
              WebkitBackdropFilter: `blur(${l.blur}px)`,
              maskImage: l.mask,
              WebkitMaskImage: l.mask,
            }}
          />
        ))}
      </div>

      <p
        className={cn(
          "absolute left-0 bottom-full text-xs tabular-nums duration-200 drop-shadow-sm drop-shadow-black",
          active ? "opacity-100 mb-2" : "opacity-0 -mb-1",
        )}
      >
        {formattedCurrentTime} / {formattedDuration}
      </p>

      {/* track: thin line by default, expands into a tinted bar on hover.
       *  Three tiers: white (played), grey (loaded/buffered), dark grey (rest). */}
      <div
        className={cn(
          "absolute inset-x-0 bottom-0 flex items-center overflow-visible rounded-full transition-all duration-200",
          active ? "h-1.5 bg-white/10" : "h-0.5 bg-white/10",
        )}
      >
        {/* buffered fill (loaded/cached amount) — grey, only while loading */}
        {showBuffered && (
          <div
            className="absolute left-0 top-0 h-full rounded-full bg-white/30 transition-[width] duration-300"
            style={{ width: `${bufferedPct * 100}%` }}
            aria-hidden
          />
        )}
        {/* progress fill — white, sits on top of buffered */}
        <div
          className="relative h-full rounded-full bg-white"
          style={{ width: `${pct * 100}%` }}
        >
          {/* thumb */}
          <span
            className={cn(
              "absolute right-0 top-1/2 -translate-y-1/2 translate-x-1/2 rounded-full bg-white shadow-md transition-all duration-200",
              active ? "h-3 w-3 opacity-100" : "h-0 w-0 opacity-0",
            )}
          />
        </div>
      </div>
    </div>
  );
}
