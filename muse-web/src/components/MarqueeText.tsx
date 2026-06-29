import { useEffect, useRef, useState } from "react";
import { cn } from "@/lib/utils";

/**
 * Marquee text — scrolls horizontally when text overflows its container,
 * stays static when it fits. Matches the iOS MarqueeText behavior:
 * only animates when needed, gradient fade on both edges, linear scroll
 * with a delay before starting.
 */
export function MarqueeText({
  text,
  className,
  startDelay = 800,
  speed = 30, // px per second
}: {
  text: string;
  className?: string;
  startDelay?: number;
  speed?: number;
}) {
  const containerRef = useRef<HTMLDivElement>(null);
  const textRef = useRef<HTMLSpanElement>(null);
  const [needsScroll, setNeedsScroll] = useState(false);

  useEffect(() => {
    const check = () => {
      const container = containerRef.current;
      const text = textRef.current;
      if (!container || !text) return;
      setNeedsScroll(text.scrollWidth > container.clientWidth + 1);
    };
    check();
    // Re-check on resize
    const ro = new ResizeObserver(check);
    if (containerRef.current) ro.observe(containerRef.current);
    return () => ro.disconnect();
  }, [text]);

  return (
    <div
      ref={containerRef}
      className={cn("relative overflow-hidden", className)}
    >
      <span
        ref={textRef}
        className={cn(
          "inline-block whitespace-nowrap",
          needsScroll && "animate-marquee",
        )}
        style={
          needsScroll
            ? {
                animationDelay: `${startDelay}ms`,
                animationDuration: `${text.length * (1000 / speed)}s`,
                maskImage:
                  "linear-gradient(to right, transparent, black 12px, black calc(100% - 12px), transparent)",
                WebkitMaskImage:
                  "linear-gradient(to right, transparent, black 12px, black calc(100% - 12px), transparent)",
              }
            : undefined
        }
      >
        {needsScroll ? `${text}   ${text}   ` : text}
      </span>
    </div>
  );
}
