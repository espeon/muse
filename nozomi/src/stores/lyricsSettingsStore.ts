import { create } from "zustand";
import { persist } from "zustand/middleware";

import Kuroshiro from "@sglkc/kuroshiro";
import KuromojiAnalyzer from "@sglkc/kuroshiro-analyzer-kuromoji";

const analyzer = new KuromojiAnalyzer({
  dictPath: "https://takuyaa.github.io/kuromoji.js/demo/kuromoji/dict/",
});

const kuroshiro = new Kuroshiro();
kuroshiro.init(analyzer).then(() => {
  console.log("Kuroshiro analyzer loaded");
});

/// Enum for transliteration from
/// E.g. for Japanese to romaji, 日本語 -> nihongo
export enum TranslitLanguage {
  NONE = "none",
  JAPANESE = "ja",
  CHINESE = "zh",
  KOREAN = "ko",
}

export enum JapaneseOptions {
  ROMAJI = "romaji",
  HIRAGANA = "hiragana",
  KATAKANA = "katakana",
  FURIGANA_KATAKANA = "furigana_katakana",
  FURIGANA_HIRAGANA = "furigana_hiragana",
}

type LyricsSettingsStore = {
  lyricsLanguage: string;
  jpOptions: JapaneseOptions;
  translitLanguage: string;
  offset: number;
  setLyricsLanguage: (language: string) => void;
  setJPOptions: (options: Partial<JapaneseOptions>) => void;
  setTranslitLanguage: (language: string) => void;
  setOffset: (offset: number) => void;
  resetSettings: () => void;
  currentTranslitLanguages: TranslitLanguage[];
  detectTranslitLanguages: () => void;
};

const DEFAULT_SETTINGS: LyricsSettingsStore["lyricsLanguage"] = "english";
const DEFAULT_JPOPTIONS: JapaneseOptions = JapaneseOptions.FURIGANA_HIRAGANA;
const DEFAULT_TRANSLIT_LANGUAGE: LyricsSettingsStore["translitLanguage"] =
  "romaji";
const DEFAULT_OFFSET: LyricsSettingsStore["offset"] = 0;

export const useLyricsSettings = create<LyricsSettingsStore>((set, get) => ({
  ...get(),
  setLyricsLanguage: (language: string) =>
    set((state) => ({ ...state, lyricsLanguage: language })),
  setJPOptions: (options: Partial<JapaneseOptions>) =>
    set((state) => ({
      ...state,
      jpOptions: state.jpOptions,
    })),
  setTranslitLanguage: (language: string) =>
    set((state) => ({ ...state, translitLanguage: language })),
  setOffset: (offset: number) => set((state) => ({ ...state, offset })),
  resetSettings: () =>
    set(() => ({
      lyricsLanguage: DEFAULT_SETTINGS,
      jpOptions: DEFAULT_JPOPTIONS,
      translitLanguage: DEFAULT_TRANSLIT_LANGUAGE,
      offset: DEFAULT_OFFSET,
    })),
  currentTranslitLanguages: [
    TranslitLanguage.JAPANESE,
    TranslitLanguage.CHINESE,
    TranslitLanguage.KOREAN,
  ],
  detectTranslitLanguages: () => {
    const translitLanguages: TranslitLanguage[] = [];
    if (get().jpOptions === JapaneseOptions.FURIGANA_KATAKANA) {
      translitLanguages.push(TranslitLanguage.JAPANESE);
    }
    if (get().jpOptions === JapaneseOptions.FURIGANA_HIRAGANA) {
      translitLanguages.push(TranslitLanguage.JAPANESE);
    }
    if (get().jpOptions === JapaneseOptions.HIRAGANA) {
      translitLanguages.push(TranslitLanguage.JAPANESE);
    }
    if (get().jpOptions === JapaneseOptions.KATAKANA) {
      translitLanguages.push(TranslitLanguage.JAPANESE);
    }
    if (get().translitLanguage === TranslitLanguage.JAPANESE) {
      translitLanguages.push(TranslitLanguage.KOREAN);
    }
    return translitLanguages;
  },
}));
