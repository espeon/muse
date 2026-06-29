import { useEffect, useRef, useState } from "react";
import { cn } from "@/lib/utils";

/** Horizontal carousel wrapper with left/right edge gradient fades. Fades fade
 *  in/out based on scroll position, so only the edges that actually overflow
 *  show gradients. */
export function HorizontalScroll({
  children,
  className,
  innerClassName,
}: {
  children: React.ReactNode;
  className?: string;
  innerClassName?: string;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const [leftVisible, setLeftVisible] = useState(false);
  const [rightVisible, setRightVisible] = useState(true);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const sync = () => {
      setLeftVisible(el.scrollLeft > 0);
      const maxScroll = el.scrollWidth - el.clientWidth;
      setRightVisible(el.scrollLeft < maxScroll - 1);
    };
    sync();
    el.addEventListener("scroll", sync, { passive: true });
    const ro = new ResizeObserver(sync);
    ro.observe(el);
    return () => {
      el.removeEventListener("scroll", sync);
      ro.disconnect();
    };
  }, []);

  return (
    <div className={cn("relative overflow-hidden", className)}>
      <div
        ref={ref}
        className={cn(
          "-mx-4 flex gap-4 overflow-x-auto px-4 pb-2 md:-mx-6 md:px-6 [scrollbar-width:none] [&::-webkit-scrollbar]:hidden",
          innerClassName,
        )}
      >
        {children}
      </div>
      {/* left fade */}
      <div
        className={cn(
          "pointer-events-none absolute inset-y-0 left-0 z-10 w-14 bg-gradient-to-r from-background to-transparent transition-opacity duration-200 md:w-24",
          leftVisible ? "opacity-100" : "opacity-0",
        )}
      />
      {/* right fade */}
      <div
        className={cn(
          "pointer-events-none absolute inset-y-0 right-0 z-10 w-14 bg-gradient-to-l from-background to-transparent transition-opacity duration-200 md:w-24",
          rightVisible ? "opacity-100" : "opacity-0",
        )}
      />
    </div>
  );
}
