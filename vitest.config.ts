import { svelte } from "@sveltejs/vite-plugin-svelte";
import { svelteTesting } from "@testing-library/svelte/vite";
import { defineConfig } from "vitest/config";

// Tests de l'interface : fichiers `*.test.ts` à côté du code testé.
export default defineConfig({
  plugins: [svelte(), svelteTesting()],
  test: {
    environment: "jsdom",
    include: ["src/**/*.test.ts"],
  },
});
