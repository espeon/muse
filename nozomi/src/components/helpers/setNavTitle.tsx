"use client";
import { useTitleStore } from "@/stores/titleStore";
import { RefObject, useEffect, useMemo, useRef, useState } from "react";

/// Set the title and visibility of the nav bar title
/// Goes under the current page title
export default function SetNavTitle({ title }: { title: string }) {
  const divRef = useRef<HTMLDivElement>(null);
  const { setPageTitle, setPageTitleVisible } = useTitleStore();
  const isVisible = useOnScreen(divRef);
  useEffect(() => {
    setPageTitle(title);
  }, [title]);

  useEffect(() => {
    console.log("visible", isVisible);
    setPageTitleVisible(isVisible);
  }, [isVisible]);

  return (
    <div className="relative h-0 w-0">
      <div className="absolute bottom-8" ref={divRef}></div>
    </div>
  );
}

export function useOnScreen(ref: RefObject<HTMLElement>) {
  const [isIntersecting, setIntersecting] = useState(false);
  if (typeof window !== "undefined") {
    const observer = useMemo(
      () =>
        new IntersectionObserver(([entry]) =>
          setIntersecting(entry.isIntersecting)
        ),
      [ref]
    );

    useEffect(() => {
      if (ref != null) {
        observer.observe(ref.current!);
        return () => observer.disconnect();
      }
    }, []);
  }

  return isIntersecting;
}
