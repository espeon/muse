import { useState } from "react";
import { PiCaretLeft, PiCaretRight, PiCaretUp } from "react-icons/pi";
import {
  JapaneseOptions,
  TranslitLanguage,
} from "@/stores/lyricsSettingsStore";

export default function LyricsMenu({
  offset,
  setOffset,
  translit,
  setTranslit,
  jpOpts,
  setJpOpts,
}: {
  offset: number;
  setOffset: (offset: number) => void;
  translit: TranslitLanguage;
  setTranslit: (lang: TranslitLanguage) => void;
  jpOpts: JapaneseOptions;
  setJpOpts: (opts: JapaneseOptions) => void;
}) {
  const [menuShown, setMenuShown] = useState(false);
  return (
    <div className="sticky w-full bottom-0 mt-auto mb-2" id="nav">
      <div className="flex items-right justify-end h-full pb-2 text-center align-left items-end">
        <div
          className={`bg-black/80 border px-4 py-2 border-slate-500/50 rounded-md flex flex-col items-end justify-end gap-2 transition-all duration-300 animate-in fade-in slide-in-from-bottom-10 ${menuShown ? "" : "hidden"}`}
        >
          <div>
            <select
              className="text-slate-400 text-xl flex items-center pl-3 py-1 bg-slate-900/50 md:hover:bg-slate-800 rounded-full gap-2 transition-all duration-300 outline-none"
              onChange={(e) => setTranslit(e.target.value as TranslitLanguage)}
              value={translit}
            >
              <option value={TranslitLanguage.NONE}>None</option>
              <option value={TranslitLanguage.JAPANESE}>Japanese</option>
              <option value={TranslitLanguage.CHINESE}>Chinese</option>
              <option value={TranslitLanguage.KOREAN}>Korean</option>
            </select>
          </div>
          <div
            className={`${translit === TranslitLanguage.JAPANESE ? "" : "hidden"}`}
          >
            <select
              className="text-slate-400 text-xl flex items-center pl-3 py-1 bg-slate-900/50 md:hover:bg-slate-800 rounded-full gap-2 transition-all duration-300 outline-none"
              onChange={(e) => setJpOpts(e.target.value as JapaneseOptions)}
              value={jpOpts}
            >
              <option value={JapaneseOptions.ROMAJI}>Romaji</option>
              <option value={JapaneseOptions.HIRAGANA}>Hiragana</option>
              <option value={JapaneseOptions.KATAKANA}>Katakana</option>
              <option value={JapaneseOptions.FURIGANA_HIRAGANA}>
                Furigana (Hiragana)
              </option>
              <option value={JapaneseOptions.FURIGANA_KATAKANA}>
                Furigana (Katakana)
              </option>
            </select>
          </div>
          <div className="flex items-center align-middle justify-center gap-2">
            <button
              className="text-slate-400 text-xl flex items-center p-2 disabled:bg-transparent md:bg-slate-900/50 md:hover:bg-slate-800 disabled:text-transparent md:disabled:text-slate-600 rounded-full gap-2 transition-all duration-300"
              onClick={() => setOffset(Number((offset - 0.1).toFixed(3)))}
              disabled={offset <= -3}
            >
              <PiCaretLeft className="text-2xl" />
            </button>
            <div
              className="h-full align-middle hover:text-slate-300 cursor-alias w-12"
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
        </div>
        <button
          className="text-slate-400 text-xl flex items-center p-2 ml-2 disabled:bg-transparent md:bg-slate-900/50 md:hover:bg-slate-800 disabled:text-transparent md:disabled:text-slate-600 rounded-full gap-2 transition-all duration-300"
          onClick={() => setMenuShown(!menuShown)}
          disabled={offset >= 3}
        >
          <PiCaretUp
            className={`text-2xl transition-all duration-300 ${menuShown ? "rotate-180" : ""}`}
          />
        </button>
      </div>
    </div>
  );
}
