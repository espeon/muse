import { useQuery } from "@tanstack/react-query";
import { fetchMe } from "@/lib/api";
import { useSession } from "@/lib/auth";
import type { NameAlias, NamePreference } from "@/types";

export function profileQueryKey(sessionVersion: number) {
  return ["me", sessionVersion] as const;
}

export function useProfile() {
  const { isLoggedIn, sessionVersion } = useSession();
  return useQuery({
    queryKey: profileQueryKey(sessionVersion),
    queryFn: fetchMe,
    enabled: isLoggedIn,
    staleTime: 30_000,
  });
}

export function useNamePreference(): NamePreference {
  return useProfile().data?.name_display ?? "original";
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
