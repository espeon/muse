import { cookies } from "next/headers";

export interface CookiePair {
  name: string;
  value: string;
}

export function getCookiePairServer(names: string[]): CookiePair {
  let ck = cookies();
  for (const name of names) {
    let cookie = ck.get(name);
    if (cookie) {
      return {
        name: cookie.name,
        value: cookie.value,
      };
    }
  }
  throw new Error("No cookie found");
}
