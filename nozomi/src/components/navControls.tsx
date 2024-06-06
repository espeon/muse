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
import { useEffect, useRef } from "react";

const menuItemClass =
  "text-slate-400 text-xl flex items-center hover:bg-slate-800 rounded-md transition-all duration-300";

export default function NavControls() {
  const router = useRouter();
  const { history, future } = useRouteStore();
  const { pageTitle, pageTitleVisible } = useTitleStore();
  
  return (
    <div className="sticky w-full top-4 z-10 h-0 md:h-8 flex flex-row justify-between items-center text-center" id="nav">
      <div className={`w-full flex flex-row align-middle flex-1 text-center pt-4 md:px-4 md:pt-4 md:bg-transparent bg-gradient-to-t from-transparent from-0% via-black/90 via-20% to-black/90 ${pageTitleVisible ? "-translate-y-20 " : ""} transition-all duration-300`}>
        <div className={`sticky flex flex-row items-center gap-6 pt-4 md:pt-0 pb-4 transition-all duration-300 ${pageTitleVisible ? "translate-y-20" : ""}`}>
          <button
            className="text-slate-400 text-xl flex items-center p-3 disabled:bg-transparent md:bg-slate-900/50 md:hover:bg-slate-800 disabled:text-transparent md:disabled:text-slate-600 rounded-full gap-2 transition-all duration-300"
            disabled={history.length < 2}
            onClick={() => router.back()}
          >
            <PiCaretLeft className="h-6 w-6" />
          </button>
          <button
            className="text-slate-400 text-xl hidden md:flex items-center p-3 disabled:bg-transparent md:bg-slate-900/50 md:hover:bg-slate-800 disabled:text-slate-600 rounded-full gap-2 transition-all duration-300"
            disabled={future.length === 0}
            onClick={() => router.forward()}
          >
            <PiCaretRight className="h-6 w-6" />
          </button>
          <h3 className={`hidden md:block w-full min-w-64 text-start text-lg mt-0.5 transition-all duration-300 ${pageTitleVisible ? "text-transparent translate-y-3" : "translate-y-0"}`}>{pageTitle}</h3>
        </div>
        <h3 className={`block md:invisible w-full pb-2 text-center text-lg pt-6 mt-0.5 transition-all duration-300 ${pageTitleVisible ? "text-transparent translate-y-3" : "translate-y-0"}`}>{pageTitle}</h3>
        <div className="w-6 mx-4"></div>
          <img
            className={`hidden lg:block h-10 w-10 aspect-square rounded-full border-2 border-gray-600 transition-all duration-300 ${pageTitleVisible ? "translate-y-20" : ""}`}
            src="https://images-ext-1.discordapp.net/external/Q1krVLQvG1SB8nHBvbGXzaREVat8_pOfvM7nqksidP0/%3Fsize%3D1024/https/cdn.discordapp.com/guilds/114407194971209731/users/267121875765821440/avatars/6107e4e092f5dc5f15b6ae57e630ed3c.png?format=webp&quality=lossless"
          />
      </div>
    </div>
  );
}
