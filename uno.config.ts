import { presetWabou } from "@wabou/vite/preset";
import { defineConfig } from "unocss";

// Editor/LSP configuration only. Wabou's Vite plugin compiles the same
// generated manifest directly to Style IR; no production CSS is emitted.
export default defineConfig({
  presets: [presetWabou()],
  content: {
    filesystem: ["apps/**/*.{ts,tsx,js,jsx}", "packages/**/*.{ts,tsx,js,jsx}"],
  },
});
