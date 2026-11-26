import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// Adresse fournie par la CLI Tauri quand on développe sur un autre appareil.
const host = process.env.TAURI_DEV_HOST;

// Configuration adaptée à Tauri : https://v2.tauri.app/start/frontend/vite/
export default defineConfig({
  plugins: [svelte()],
  // Laisse les erreurs Rust visibles dans le terminal.
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: "ws", host, port: 1421 } : undefined,
    watch: {
      // Le code Rust est surveillé par Tauri, pas par Vite.
      ignored: ["**/src-tauri/**"],
    },
  },
  envPrefix: ["VITE_", "TAURI_ENV_*"],
  build: {
    // WebView2 (Windows) est basé sur Chromium.
    target: "chrome105",
    minify: !process.env.TAURI_ENV_DEBUG,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
});
