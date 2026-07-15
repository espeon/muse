/** Maki base URL, no trailing slash.
 *
 *  When muse-web is served from maki (embedded mode), the SPA is same-origin
 *  with the API and MAKI_URL should be empty. When running standalone (e.g.
 *  `vite dev`), set VITE_MAKI_URL to point at the maki server. */
export const MAKI_URL = (import.meta.env.VITE_MAKI_URL ?? "").replace(/\/$/, "");

/** umi lyrics service base URL, no trailing slash. Optional — the frontend
 *  fails silently when lyrics are unavailable. */
export const UMI_URL = (import.meta.env.VITE_UMI_URL ?? "http://localhost:3032").replace(
  /\/$/,
  "",
);
