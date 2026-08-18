import { defineWabouConfig } from "@wabou/vite";

export default defineWabouConfig({
  root: import.meta.dirname,
  entry: "entry.ts",
  outDir: "dist-wabou",
  intl: {
    locales: ["en", "zh"],
  },
});
