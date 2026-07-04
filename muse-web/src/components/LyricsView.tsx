import { useCallback, useLayoutEffect, useMemo, useRef } from "react";
import type { Jlf, SyncedRichLineSegment } from "@/lyrics/jlf";
import { cn } from "@/lib/utils";

// --------------------------------------------------------------- types

interface RenderToken {
  text: string;
  startMs: number;
  endMs: number;
  timed: boolean;
}

interface RenderLine {
  startMs: number;
  endMs: number;
  text: string;
  tokens: RenderToken[] | null;
}

// --------------------------------------------------------------- helpers

interface FlatRichLine {
  timeStart: number;
  timeEnd: number;
  text: string;
  segments: SyncedRichLineSegment[];
}

/** Flatten richsync sections into a flat line list, or fall back to plain lines. */
function buildRenderLines(jlf: Jlf): RenderLine[] {
  if (jlf.richsync) {
    const flat: FlatRichLine[] = [];
    for (const section of jlf.richsync.sections) {
      for (const line of section.lines) {
        flat.push({
          timeStart: line.timeStart,
          timeEnd: line.timeEnd,
          text: line.text,
          segments: line.segments,
        });
      }
    }
    return flat.map((line, i) => {
      const end =
        line.timeEnd > 0
          ? line.timeEnd
          : (flat[i + 1]?.timeStart ?? line.timeStart + 4000);
      const tokens =
        line.segments.length > 0 ? tokenize(line.text, line.segments) : null;
      return { startMs: line.timeStart, endMs: end, text: line.text, tokens };
    });
  }
  const plain = jlf.lines.lines;
  return plain.map((line, i) => {
    const end = plain[i + 1]?.time ?? line.time + 6000;
    return { startMs: line.time, endMs: end, text: line.text, tokens: null };
  });
}

function tokenize(
  text: string,
  segments: SyncedRichLineSegment[],
): RenderToken[] {
  const tokens: RenderToken[] = [];
  let cursor = 0;
  for (const seg of segments) {
    const idx = text.indexOf(seg.text, cursor);
    if (idx < 0) continue;
    if (idx > cursor) {
      tokens.push({
        text: text.slice(cursor, idx),
        startMs: 0,
        endMs: 0,
        timed: false,
      });
    }
    tokens.push({
      text: seg.text,
      startMs: seg.timeStart,
      endMs: seg.timeEnd,
      timed: true,
    });
    cursor = idx + seg.text.length;
  }
  if (cursor < text.length) {
    tokens.push({
      text: text.slice(cursor),
      startMs: 0,
      endMs: 0,
      timed: false,
    });
  }
  return tokens;
}

function activeIndexFor(lines: RenderLine[], timeMs: number): number {
  if (lines.length === 0) return -1;
  let lo = 0;
  let hi = lines.length;
  while (lo < hi) {
    const mid = (lo + hi) >> 1;
    if (lines[mid].startMs <= timeMs) lo = mid + 1;
    else hi = mid;
  }
  return Math.max(0, lo - 1);
}

function tokenProgress(tok: RenderToken, ms: number): number {
  const dur = Math.max(1, tok.endMs - tok.startMs);
  return Math.min(1, Math.max(0, (ms - tok.startMs) / dur));
}

function activeTimedIndex(tokens: RenderToken[], ms: number): number {
  let result = -1;
  for (let i = 0; i < tokens.length; i++) {
    if (tokens[i].timed && tokens[i].startMs <= ms) result = i;
    else if (tokens[i].timed) break;
  }
  return result;
}

// --------------------------------------------------------------- component

export interface LyricsViewProps {
  jlf: Jlf;
  /** current playback position in seconds */
  currentTime: number;
  onSeek?: (timeMs: number) => void;
  className?: string;
}

export function LyricsView({
  jlf,
  currentTime,
  onSeek,
  className,
}: LyricsViewProps) {
  const lines = useMemo(() => buildRenderLines(jlf), [jlf]);
  const currentTimeMs = Math.round(currentTime * 1000);
  const activeIndex = useMemo(
    () => activeIndexFor(lines, currentTimeMs),
    [lines, currentTimeMs],
  );

  const containerRef = useRef<HTMLDivElement>(null);
  const itemRefs = useRef<(HTMLDivElement | null)[]>([]);

  // Auto-scroll active line into view
  useLayoutEffect(() => {
    if (activeIndex < 0) return;
    const container = containerRef.current;
    const el = itemRefs.current[activeIndex];
    if (!container || !el) return;
    const eRect = el.getBoundingClientRect();
    const target =
      el.offsetTop - container.clientHeight * 0.325 + eRect.height / 2;
    container.scrollTo({ top: target, behavior: "smooth" });
  }, [activeIndex]);

  if (lines.length === 0) {
    return (
      <div className="flex h-full items-center justify-center">
        <p className="text-white/60">No lyrics available</p>
      </div>
    );
  }

  return (
    <div
      ref={containerRef}
      className={cn(
        "relative h-full overflow-y-auto overflow-x-hidden no-scrollbar",
        "mask-[linear-gradient(to_bottom,transparent_0%,black_12%,black_85%,transparent_100%)]",
        className,
      )}
    >
      <div className="px-6 pb-[40vh] pt-[35vh]">
        {lines.map((line, i) => (
          <LyricRow
            key={`${line.startMs}-${i}`}
            ref={(el: HTMLDivElement | null) => {
              itemRefs.current[i] = el;
            }}
            line={line}
            isActive={i === activeIndex}
            distanceFromActive={i - activeIndex}
            smoothMs={currentTimeMs}
            onSeek={onSeek}
          />
        ))}
      </div>
    </div>
  );
}

// --------------------------------------------------------------- row

interface LyricRowProps {
  line: RenderLine;
  isActive: boolean;
  distanceFromActive: number;
  smoothMs: number;
  onSeek?: (timeMs: number) => void;
  ref?: (el: HTMLDivElement | null) => void;
}

function LyricRow({
  line,
  isActive,
  distanceFromActive,
  smoothMs,
  onSeek,
  ref,
}: LyricRowProps) {
  const isPast = distanceFromActive < 0;
  // Apple Music-style asymmetry: past lines fade faster than future lines.
  const dimAlpha = isActive
    ? 1
    : isPast
      ? // TODO: toggle apple music mode (fade out on past lines) else keep past behaviour (commented out)
        0 //Math.max(0.1, 0.45 / (1 + 0.45 * Math.abs(distanceFromActive)))
      : Math.max(0.2, 0.65 / (1 + 0.3 * Math.abs(distanceFromActive)));
  const scale = isActive ? 1 : 0.95;
  const shadowOpacity = isActive ? 1 : 0;

  const handleClick = useCallback(() => {
    onSeek?.(line.startMs);
  }, [onSeek, line.startMs]);

  return (
    <div
      ref={ref}
      className="mb-4 cursor-pointer origin-left pl-2 text-left transition-all duration-250 ease-out sm:mb-6 md:mb-8 lg:mb-12"
      style={{
        transform: `scale(${scale})`,
        opacity: dimAlpha,
        textShadow: `0 1px 3px rgba(50,50,50,${0.5 * shadowOpacity}), 0 2px 6px rgba(100,100,100,${0.7 * shadowOpacity}), 0 4px 12px rgba(110,110,110,${0.5 * shadowOpacity})`,
      }}
      onClick={handleClick}
    >
      {line.text === "" ? (
        <InstrumentalDots isActive={isActive} />
      ) : isActive && line.tokens ? (
        <SyllabicLine tokens={line.tokens} smoothMs={smoothMs} />
      ) : (
        <p className="text-4xl font-semibold leading-snug text-white lg:text-6xl">
          {line.text}
        </p>
      )}
    </div>
  );
}

// --------------------------------------------------------------- syllabic

function SyllabicLine({
  tokens,
  smoothMs,
}: {
  tokens: RenderToken[];
  smoothMs: number;
}) {
  const activeToken = useMemo(
    () => activeTimedIndex(tokens, smoothMs),
    [tokens, smoothMs],
  );

  return (
    <div className="text-4xl font-semibold leading-snug text-white sm:text-2xl md:text-3xl lg:text-5xl xl:text-6xl">
      {tokens.map((tok, i) => {
        // Check if there's a trailing space in the original text after this token.
        const spaceAfter = i < tokens.length - 1 && tokens[i + 1].text.startsWith(" ");

        const content = (() => {
          if (!tok.timed) {
            return (
              <span className="text-white">{tok.text}</span>
            );
          }
          if (i < activeToken) {
            return (
              <span className="text-white">{tok.text}</span>
            );
          }
          if (i === activeToken) {
            const progress = tokenProgress(tok, smoothMs);
            // Overshoot: extend the sung portion slightly past actual progress
            // so the highlight leads the audio slightly (matches the iOS shader feel).
            const overshoot = 0.03;
            const sweepPct = Math.min(100, (progress + overshoot) * 100);
            // Gradient edge width as a percentage of the token width.
            // Mirrors the Metal shader's smoothstep(sweep - gradientWidth, sweep + gradientWidth, uv_x).
            const edgePct = 8;
            // Clamp so the mask is fully hidden at progress 0 and fully shown
            // near the end — no white peeking in before the token starts.
            const sungEnd =
              progress < 0.01 ? 0 : Math.max(0, sweepPct - edgePct);
            const unsungStart =
              progress > 0.97 ? 100 : Math.min(100, sweepPct + edgePct);

            return (
              <span className="relative inline-block">
                <span className="text-white/30">{tok.text}</span>
                <span
                  className="absolute inset-0 text-white"
                  style={{
                    maskImage: `linear-gradient(to right, black ${sungEnd}%, transparent ${unsungStart}%)`,
                    WebkitMaskImage: `linear-gradient(to right, black ${sungEnd}%, transparent ${unsungStart}%)`,
                  }}
                >
                  {tok.text}
                </span>
              </span>
            );
          }
          return (
            <span className="text-white/30">{tok.text}</span>
          );
        })();

        return (
          <span key={i} className="transition-all duration-100 ease-in">
            {content}
            {spaceAfter ? " " : ""}
          </span>
        );
      })}
    </div>
  );
}

// --------------------------------------------------------------- instrumental dots

function InstrumentalDots({ isActive }: { isActive: boolean }) {
  return (
    <div className="flex items-center gap-2 py-2">
      {[0, 1, 2].map((i) => (
        <span
          key={i}
          className={cn(
            "rounded-full transition-all duration-300",
            isActive ? "h-3.5 w-3.5 bg-white/90" : "h-2.5 w-2.5 bg-white/30",
          )}
        />
      ))}
    </div>
  );
}
