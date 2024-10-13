"use client";
import {
  PiCaretLeft,
  PiCaretRight,
  PiHouseBold,
  PiMagnifyingGlassBold,
} from "react-icons/pi";
import { useRouter } from "next/navigation";
import { useRouteStore } from "@/stores/routeStore";
import { useTitleStore } from "@/stores/titleStore";

export default function NavControlsClient({
  children,
}: {
  children: React.ReactNode;
}) {
  const router = useRouter();
  const { history, future } = useRouteStore();
  const { pageTitle, pageTitleVisible } = useTitleStore();

  return (
    <div
      className="sticky w-full top-4 z-20 h-0 md:h-8 flex flex-row justify-between items-center text-center"
      id="nav"
    >
      <div
        className={`w-full flex flex-row align-middle flex-1 text-center pt-4 md:px-4 md:pt-4 md:bg-transparent bg-gradient-to-t from-transparent from-0% via-black/50 via-30% to-black/80 ${pageTitleVisible ? "-translate-y-20 " : ""} transition-all duration-300`}
      >
        <div
          className={`sticky flex flex-row items-center gap-6 pt-4 md:pt-0 pb-4 transition-all duration-300 ${pageTitleVisible ? "translate-y-20" : ""}`}
        >
          <button
            className="text-slate-400 text-xl flex items-center p-3 disabled:bg-slate-900/30 md:bg-slate-900/50 md:hover:bg-slate-800 ring-1 disabled:ring-0 ring-slate-700/30 disabled:text-transparent md:disabled:text-slate-600 rounded-full gap-2 transition-all duration-300"
            disabled={history.length < 2}
            onClick={() => router.back()}
          >
            <PiCaretLeft className="h-6 w-6" />
          </button>
          <button
            className="text-slate-400 text-xl hidden md:flex items-center p-3 disabled:bg-slate-900/30 md:bg-slate-900/50 md:hover:bg-slate-800 ring-1 disabled:ring-0 ring-slate-700/30 disabled:text-slate-600 rounded-full gap-2 transition-all duration-300"
            disabled={future.length === 0}
            onClick={() => router.forward()}
          >
            <PiCaretRight className="h-6 w-6" />
          </button>
          <h3
            className={`hidden md:block w-full min-w-64 text-start text-lg transition-all duration-300 ${pageTitleVisible ? "text-transparent translate-y-3" : "translate-y-0"}`}
          >
            {pageTitle}
          </h3>
        </div>
        <h3
          className={`block md:invisible w-full pb-2 text-center text-lg pt-6 transition-all duration-300 ${pageTitleVisible ? "text-transparent translate-y-3" : "translate-y-0"}`}
        >
          {pageTitle}
        </h3>
        <div className="w-12" />
        <div
          className={`hidden md:flex w-full h-12 flex-col justify-center items-end ${pageTitleVisible ? "translate-y-20 " : ""} transition-all duration-300`}
        >
          {children}
        </div>
      </div>
    </div>
  );
}
