/** Maki base URL, no trailing slash. The SPA talks to Maki directly (permissive CORS). */
export const MAKI_URL = (import.meta.env.VITE_MAKI_URL ?? "http://localhost:3033").replace(
  /\/$/,
  "",
);

/** umi lyrics service base URL, no trailing slash. Optional — the frontend
 *  fails silently when lyrics are unavailable. */
export const UMI_URL = (import.meta.env.VITE_UMI_URL ?? "https://umi.nat.vg").replace(
  /\/$/,
  "",
);
