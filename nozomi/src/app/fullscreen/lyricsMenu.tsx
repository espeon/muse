import { useState } from "react";
import { PiCaretLeft, PiCaretRight, PiCaretUp } from "react-icons/pi";

export default function LyricsMenu({offset, setOffset}: {offset: number, setOffset: (offset: number) => void}) {
    const [menuShown, setMenuShown] = useState(false);
    return <div className="sticky w-full bottom-0 h-12 mb-2" id="nav">
    <div className="flex items-right justify-end h-full pb-2 text-center align-middle">
      <div className={`flex items-center gap-2 transition-all duration-300 ${menuShown ? "fade-in-100" : "hidden"}`}>
        <button
          className="text-slate-400 text-xl flex items-center p-2 disabled:bg-transparent md:bg-slate-900/50 md:hover:bg-slate-800 disabled:text-transparent md:disabled:text-slate-600 rounded-full gap-2 transition-all duration-300"
          onClick={() => setOffset(Number((offset - 0.1).toFixed(3)))}
          disabled={offset <= -3}
        >
          <PiCaretLeft className="text-2xl" />
        </button>
        <div
          className="h-full align-middle pt-2 hover:text-slate-300 cursor-alias w-12"
          onClick={() => setOffset(0)}
        >
          {offset > 0 && "+"}
          {offset} s
        </div>
        <button
          className="text-slate-400 text-xl flex items-center p-2 disabled:bg-transparent md:bg-slate-900/50 md:hover:bg-slate-800 disabled:text-transparent md:disabled:text-slate-600 rounded-full gap-2 transition-all duration-300"
          onClick={() => setOffset(Number((offset + 0.1).toFixed(3)))}
          disabled={offset >= 3}
        >
          <PiCaretRight className="text-2xl" />
        </button>
      </div>
      <button
          className="text-slate-400 text-xl flex items-center p-2 ml-2 disabled:bg-transparent md:bg-slate-900/50 md:hover:bg-slate-800 disabled:text-transparent md:disabled:text-slate-600 rounded-full gap-2 transition-all duration-300"
          onClick={() => setMenuShown(!menuShown)}
          disabled={offset >= 3}
        >
          <PiCaretUp className={`text-2xl transition-all duration-300 ${menuShown ? "rotate-180" : ""}`} />
        </button>
    </div>
  </div>
}