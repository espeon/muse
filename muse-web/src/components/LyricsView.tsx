import { Fragment, useCallback, useLayoutEffect, useMemo, useRef } from "react";
import type { Jlf, SyncedRichLineSegment } from "@/lyrics/jlf";
import { cn } from "@/lib/utils";

// --------------------------------------------------------------- types

interface RenderToken {
  text: string;
  startMs: number;
  endMs: number;
  timed: boolean;
}

interface BgVoxRenderLine {
  startMs: number;
  endMs: number;
  text: string;
  tokens: RenderToken[];
}

interface RenderLine {
  startMs: number;
  endMs: number;
  text: string;
  tokens: RenderToken[] | null;
  agent: string;
  bgVox: BgVoxRenderLine | null;
}

// --------------------------------------------------------------- helpers

interface FlatRichLine {
  timeStart: number;
  timeEnd: number;
  text: string;
  segments: SyncedRichLineSegment[];
  agent: string;
  bgVox: BgVoxRenderLine | null;
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
          agent: line.agent,
          bgVox: line.bgVox
            ? {
                startMs: line.bgVox.timeStart,
                endMs: line.bgVox.timeEnd,
                text: line.bgVox.text,
                tokens: tokenize(line.bgVox.text, line.bgVox.segments),
              }
            : null,
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
      return {
        startMs: line.timeStart,
        endMs: end,
        text: line.text,
        tokens,
        agent: line.agent,
        bgVox: line.bgVox,
      };
    });
  }
  const plain = jlf.lines.lines;
  return plain.map((line, i) => {
    const end = plain[i + 1]?.time ?? line.time + 6000;
    return {
      startMs: line.time,
      endMs: end,
      text: line.text,
      tokens: null,
      agent: "",
      bgVox: null,
    };
  });
}

function tokenize(
  text: string,
  segments: SyncedRichLineSegment[],
): RenderToken[] {
  const tokens: RenderToken[] = [];
  let cursor = 0;
  for (const seg of segments) {
    const segText = seg.text.trim();
    if (!segText) continue;
    const idx = text.indexOf(segText, cursor);
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
      text: segText,
      startMs: seg.timeStart,
      endMs: seg.timeEnd,
      timed: true,
    });
    cursor = idx + segText.length;
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

interface TokenWord {
  /** Tokens with their original flat-array indices */
  tokens: { tok: RenderToken; idx: number }[];
  spaceAfter: boolean;
}

function isCJKText(text: string): boolean {
  for (const char of text) {
    const code = char.codePointAt(0)!;
    if (
      (code >= 0x4e00 && code <= 0x9fff) || // CJK Unified Ideographs
      (code >= 0x3400 && code <= 0x4dbf) || // CJK Extension A
      (code >= 0x3040 && code <= 0x309f) || // Hiragana
      (code >= 0x30a0 && code <= 0x30ff) || // Katakana
      (code >= 0xac00 && code <= 0xd7af) // Hangul Syllables
    ) {
      return true;
    }
  }
  return false;
}

/**
 * Group tokens into words so the browser wraps at space boundaries,
 * not mid-word between syllable tokens.
 *
 * For CJK text (no inter-word spaces), each token is its own group
 * so the browser can break between any characters — matching the
 * Swift `isCJK` path in `renderedWords`.
 */
function groupTokensIntoWords(
  tokens: RenderToken[],
  isCJK: boolean,
): TokenWord[] {
  if (isCJK) {
    const words: TokenWord[] = [];
    for (let i = 0; i < tokens.length; i++) {
      const tok = tokens[i];
      if (!tok.timed && /^\s+$/.test(tok.text)) {
        if (words.length > 0) words[words.length - 1].spaceAfter = true;
        continue;
      }
      words.push({ tokens: [{ tok, idx: i }], spaceAfter: false });
    }
    return words;
  }

  const words: TokenWord[] = [];
  let current: { tok: RenderToken; idx: number }[] = [];

  for (let i = 0; i < tokens.length; i++) {
    const tok = tokens[i];
    if (!tok.timed && /^\s+$/.test(tok.text)) {
      if (current.length > 0) {
        words.push({ tokens: current, spaceAfter: true });
        current = [];
      }
    } else {
      current.push({ tok, idx: i });
    }
  }

  if (current.length > 0) {
    words.push({ tokens: current, spaceAfter: false });
  }

  return words;
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

/**
 * Pre-activation progress for the *next* line — smoothly transitions scale,
 * blur, and opacity so the upcoming line eases in before it becomes active.
 * Mirrors the Swift `activationProgress(for:)` priming window.
 */
function activationProgressFor(
  lines: RenderLine[],
  index: number,
  currentLineIndex: number,
  timeMs: number,
): number {
  const primeWindowMs = 1500;

  if (index === currentLineIndex) return 1.0;
  if (index === currentLineIndex + 1) {
    const line = lines[index];
    const timeUntilActive = line.startMs - timeMs;
    if (timeUntilActive > 0 && timeUntilActive <= primeWindowMs) {
      const linearProgress =
        1.0 - timeUntilActive / primeWindowMs;
      return 0.9 * linearProgress * linearProgress;
    }
  }
  return 0.0;
}

type TextAlign = "left" | "center" | "right";

/** Map Apple Music agent IDs to horizontal alignment (mirrors Swift). */
function agentAlign(agent: string): TextAlign {
  if (agent === "v0" || agent === "v1") return "left";
  if (agent === "v1000") return "center";
  return "right";
}

function lerp(a: number, b: number, t: number): number {
  return a + (b - a) * t;
}

// --------------------------------------------------------------- component

export interface LyricsViewProps {
  jlf: Jlf;
  /** current playback position in seconds */
  currentTime: number;
  onSeek?: (timeMs: number) => void;
  className?: string;
  /** Apple Music style: gradually hide completed lines */
  fadeCompletedLines?: boolean;
}

export function LyricsView({
  jlf,
  currentTime,
  onSeek,
  className,
  fadeCompletedLines = false,
}: LyricsViewProps) {
  const lines = useMemo(() => buildRenderLines(jlf), [jlf]);
  const currentTimeMs = Math.round(currentTime * 1000);
  const activeIndex = useMemo(
    () => activeIndexFor(lines, currentTimeMs),
    [lines, currentTimeMs],
  );

  const containerRef = useRef<HTMLDivElement>(null);
  const itemRefs = useRef<(HTMLDivElement | null)[]>([]);
  const scrollRafRef = useRef<number | null>(null);

  // Auto-scroll active line into view (custom rAF for controllable easing)
  useLayoutEffect(() => {
    if (activeIndex < 0) return;
    const container = containerRef.current;
    const el = itemRefs.current[activeIndex];
    if (!container || !el) return;
    const eRect = el.getBoundingClientRect();
    const target =
      el.offsetTop - container.clientHeight * 0.325 + eRect.height / 2;

    // Cancel any in-flight scroll animation
    if (scrollRafRef.current !== null) {
      cancelAnimationFrame(scrollRafRef.current);
    }

    const startTop = container.scrollTop;
    const delta = target - startTop;
    const duration = 300; // matches Swift easeInOut(duration: 0.3)
    const startTime = performance.now();

    // easeInOutCubic — same curve family as Swift's .easeInOut
    const ease = (t: number) =>
      t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;

    const tick = (now: number) => {
      const t = Math.min(1, (now - startTime) / duration);
      container.scrollTop = startTop + delta * ease(t);
      if (t < 1) {
        scrollRafRef.current = requestAnimationFrame(tick);
      } else {
        scrollRafRef.current = null;
      }
    };

    scrollRafRef.current = requestAnimationFrame(tick);

    return () => {
      if (scrollRafRef.current !== null) {
        cancelAnimationFrame(scrollRafRef.current);
        scrollRafRef.current = null;
      }
    };
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
        "mask-[linear-gradient(to_bottom,transparent_0%,black_8%,black_92%,transparent_100%)]",
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
            activationProgress={activationProgressFor(
              lines,
              i,
              activeIndex,
              currentTimeMs,
            )}
            distanceFromActive={i - activeIndex}
            smoothMs={currentTimeMs}
            onSeek={onSeek}
            fadeCompletedLines={fadeCompletedLines}
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
  activationProgress: number;
  distanceFromActive: number;
  smoothMs: number;
  onSeek?: (timeMs: number) => void;
  fadeCompletedLines: boolean;
  ref?: (el: HTMLDivElement | null) => void;
}

const SPRING_EASE = "350ms cubic-bezier(0.25, 0.1, 0.25, 1)";

const MAIN_LYRICS_CLASS = cn(
  "font-semibold leading-snug text-white",
  "text-4xl sm:text-2xl md:text-3xl lg:text-5xl xl:text-6xl",
);

function LyricRow({
  line,
  isActive,
  activationProgress: actProgress,
  distanceFromActive,
  smoothMs,
  onSeek,
  fadeCompletedLines,
  ref,
}: LyricRowProps) {
  const align = agentAlign(line.agent);
  const isPast = distanceFromActive < 0;
  const normalizedDistance = Math.abs(distanceFromActive);

  // --- Visual properties (ported from RichSyncLyricsView.swift) ---

  // Scale: uniform — no distance-based scaling
  const scale = 1.0;

  // Blur: active = 0, inactive blurs more with distance (max 1.5px)
  const inactiveBlur = Math.min(normalizedDistance * 0.4, 1.5);
  const blur = lerp(inactiveBlur, 0, actProgress);

  // Shadow: active gets a subtle glow, inactive = none
  const shadowRadius = lerp(0, 12, actProgress);

  // Y-offset: past lines drift up, future lines drift down (max ±8px)
  const direction = distanceFromActive > 0 ? 1.0 : -1.0;
  const inactiveYOffset = direction * Math.min(normalizedDistance * 2, 8);
  const yOffset = lerp(inactiveYOffset, 0, actProgress);

  // Opacity
  const opacity = isActive
    ? 1.0
    : fadeCompletedLines && isPast
      ? Math.max(0, 0.5 - 0.15 * normalizedDistance)
      : Math.max(0.25, 0.55 / (1 + 0.15 * normalizedDistance));

  const transformOrigin =
    align === "left"
      ? "left center"
      : align === "right"
        ? "right center"
        : "center center";

  // bgVox active state
  const bgVoxActive =
    line.bgVox !== null &&
    smoothMs >= line.bgVox.startMs &&
    smoothMs < line.bgVox.endMs;

  const handleClick = useCallback(() => {
    onSeek?.(line.startMs);
  }, [onSeek, line.startMs]);

  return (
    <div
      ref={ref}
      className="mb-4 cursor-pointer sm:mb-6 md:mb-8 lg:mb-12"
      style={{
        transform: `translateY(${yOffset}px) scale(${scale})`,
        transformOrigin,
        opacity,
        filter: `blur(${blur}px)`,
        textShadow: `0 ${shadowRadius * 0.4}px ${shadowRadius}px rgba(0,0,0,0.3)`,
        textAlign: align,
        transition: `transform ${SPRING_EASE}, opacity ${SPRING_EASE}, filter ${SPRING_EASE}, text-shadow ${SPRING_EASE}`,
      }}
      onClick={handleClick}
    >
      {line.text === "" ? (
        <InstrumentalDots isActive={isActive} />
      ) : (isActive || bgVoxActive) && line.tokens ? (
        <SyllabicLine
          tokens={line.tokens}
          smoothMs={smoothMs}
          align={align}
        />
      ) : (
        <p className={MAIN_LYRICS_CLASS}>
          {line.text}
        </p>
      )}

      {/* Background vocals */}
      {line.bgVox && (
        <div className="mt-2">
          <BgVoxContent
            bgVox={line.bgVox}
            currentTimeMs={smoothMs}
            align={align}
          />
        </div>
      )}
    </div>
  );
}

// --------------------------------------------------------------- background vocals

function BgVoxContent({
  bgVox,
  currentTimeMs,
  align,
}: {
  bgVox: BgVoxRenderLine;
  currentTimeMs: number;
  align: TextAlign;
}) {
  const isActive =
    currentTimeMs >= bgVox.startMs && currentTimeMs < bgVox.endMs;
  const opacity = isActive ? 0.8 : 0.3;

  if (isActive && bgVox.tokens.length > 0) {
    return (
      <div style={{ opacity }}>
        <SyllabicLine
          tokens={bgVox.tokens}
          smoothMs={currentTimeMs}
          align={align}
          variant="bgVox"
        />
      </div>
    );
  }

  return (
    <p
      className="text-2xl font-semibold leading-snug text-white lg:text-3xl"
      style={{ textAlign: align, opacity }}
    >
      {bgVox.text}
    </p>
  );
}

// --------------------------------------------------------------- syllabic

function SyllabicLine({
  tokens,
  smoothMs,
  align,
  variant = "main",
}: {
  tokens: RenderToken[];
  smoothMs: number;
  align: TextAlign;
  variant?: "main" | "bgVox";
}) {
  const activeToken = useMemo(
    () => activeTimedIndex(tokens, smoothMs),
    [tokens, smoothMs],
  );
  const cjk = useMemo(
    () => tokens.some((t) => isCJKText(t.text)),
    [tokens],
  );
  const words = useMemo(
    () => groupTokensIntoWords(tokens, cjk),
    [tokens, cjk],
  );

  const renderToken = (tok: RenderToken, idx: number) => {
    if (!tok.timed) {
      return <span className="text-white">{tok.text}</span>;
    }
    if (idx < activeToken) {
      return <span className="text-white">{tok.text}</span>;
    }
    if (idx === activeToken) {
      // Match Metal karaokeSweep shader: sweep extends beyond [0,1]
      // by gradientWidth so the mask edge starts/ends cleanly.
      const progress = tokenProgress(tok, smoothMs);
      const gw = 0.08; // gradient width as fraction of token width
      const sweep = progress * (1 + 2 * gw) - gw;
      const sungEnd = Math.max(0, (sweep - gw) * 100);
      const unsungStart = Math.min(100, (sweep + gw) * 100);

      return (
        <span
          className="relative inline-block duration-300 transition-all"
          style={{ paddingBottom: "0.15em", marginBottom: "-0.15em" }}
        >
          <span className="text-white/50 duration-300 transition-all">{tok.text}</span>
          <span
            className="absolute inset-0 text-white duration-300 transition-all"
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
    return <span className="text-white/50 duration-300 transition-all">{tok.text}</span>;
  };

  return (
    <div
      className={cn(
        variant === "bgVox"
          ? "font-semibold leading-snug text-white text-2xl lg:text-3xl"
          : MAIN_LYRICS_CLASS,
      )}
      style={{ textAlign: align }}
    >
      {words.map((word, wi) => (
        <Fragment key={wi}>
          <span className={cn("inline-block", !cjk && "whitespace-nowrap")}>
            {word.tokens.map(({ tok, idx }) => (
              <Fragment key={idx}>{renderToken(tok, idx)}</Fragment>
            ))}
          </span>
          {word.spaceAfter ? " " : ""}
        </Fragment>
      ))}
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
