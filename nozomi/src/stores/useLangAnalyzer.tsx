"use client";
import Kuroshiro from "@sglkc/kuroshiro";
import KuromojiAnalyzer from "@sglkc/kuroshiro-analyzer-kuromoji";
import { JapaneseOptions, TranslitLanguage } from "./lyricsSettingsStore";
import { useEffect, useRef } from "react";

const analyzer = new KuromojiAnalyzer({
  dictPath: "https://takuyaa.github.io/kuromoji.js/demo/kuromoji/dict/",
});

const kuroshiro = new Kuroshiro();
kuroshiro.init(analyzer).then(() => {
  console.log("Kuroshiro analyzer loaded");
});

export function getKuroshiro(language: string): Kuroshiro {
  return kuroshiro;
}

export async function transliterate(
  text: string,
  lang: TranslitLanguage,
  jpOpts: JapaneseOptions,
): Promise<string | null> {
  if (!analyzer) return null;

  try {
    if (lang === TranslitLanguage.JAPANESE) {
      let opts =
        jpOpts === JapaneseOptions.FURIGANA_KATAKANA ||
        jpOpts === JapaneseOptions.FURIGANA_HIRAGANA
          ? { mode: "furigana", to: jpOpts.replace("furigana_", "") }
          : { to: jpOpts, romajiSystem: "hepburn" };

      kuroshiro.convert(text, opts).then((res) => {
        return res;
      });
    } else {
      // undefined
      return null;
    }
  } catch (error) {
    console.error("Error transliterating:", error);
    return null;
  }
  return null;
}

export function isLanguageDetected(
  text: string,
  onlyTest: TranslitLanguage[] = [
    TranslitLanguage.JAPANESE,
    TranslitLanguage.KOREAN,
    TranslitLanguage.CHINESE,
  ],
): TranslitLanguage[] {
  const detectedLanguages: TranslitLanguage[] = [];
  if (
    kuroshiro.Util.hasJapanese(text) &&
    onlyTest.includes(TranslitLanguage.JAPANESE)
  )
    detectedLanguages.push(TranslitLanguage.JAPANESE);
  return [];
}

export function LyricText({
  text,
  lang,
  jpOpts,
}: {
  text: string;
  lang: TranslitLanguage;
  jpOpts?: JapaneseOptions;
}) {
  const containerRef = useRef<HTMLSpanElement>(null);

  useEffect(() => {
    if (
      lang === TranslitLanguage.JAPANESE &&
      kuroshiro.Util.hasJapanese(text)
    ) {
      let opts =
        jpOpts === JapaneseOptions.FURIGANA_KATAKANA ||
        jpOpts === JapaneseOptions.FURIGANA_HIRAGANA
          ? { mode: "furigana", to: jpOpts.replace("furigana_", "") }
          : { to: jpOpts, romajiSystem: "hepburn" };

      kuroshiro.convert(text, opts).then((res) => {
        if (containerRef.current) {
          containerRef.current.innerHTML = res;
        }
      });
    } else {
      if (containerRef.current) {
        containerRef.current.textContent = text;
      }
    }
  }, [lang, jpOpts, text]);

  return <span ref={containerRef} />;
}
