import { useSyncExternalStore } from "react";
import type { NameAlias, NamePreference } from "@/types";

const KEY = "muse-web.nameDisplay";

const listeners = new Set<() => void>();
let current: NamePreference = load();

function load(): NamePreference {
  try {
    const value = localStorage.getItem(KEY);
    if (value === "original" || value === "romanised" || value === "english") return value;
  } catch {
    /* storage unavailable */
  }
  return "original";
}

export function getNamePreference(): NamePreference {
  return current;
}

export function setNamePreference(value: NamePreference) {
  current = value;
  try {
    localStorage.setItem(KEY, value);
  } catch {
    /* storage unavailable */
  }
  listeners.forEach((l) => l());
}

export function useNamePreference(): NamePreference {
  return useSyncExternalStore(
    (listener) => {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    getNamePreference,
    () => "original" as NamePreference,
  );
}

/** Pick an alternate name by preference, falling back to the canonical name. */
export function displayName(name: string, aliases: NameAlias[] | undefined, pref: NamePreference) {
  if (!aliases?.length || pref === "original") return name;
  const matches =
    pref === "romanised"
      ? aliases.filter(
          (a) =>
            a.script === "Latn" ||
            /-Latn$/i.test(a.locale ?? "") ||
            /roman/i.test(a.kind ?? ""),
        )
      : aliases.filter((a) => /^en\b/i.test(a.locale ?? "") || /english/i.test(a.kind ?? ""));
  return matches[0]?.alias ?? name;
}
