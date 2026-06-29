import { fileURLToPath, URL } from "node:url";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";

// muse-web talks to Maki directly (Maki's CorsLayer is permissive, see
// maki/src/main.rs:133), so there is no dev proxy here. Point the client at
// Maki via VITE_MAKI_URL (see .env.example). The signed stream URLs Maki
// returns are absolute and also CORS-enabled, so fetch()+decodeAudioData()
// for the gapless path works without a BFF.
export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      "@": fileURLToPath(new URL("./src", import.meta.url)),
    },
  },
});
