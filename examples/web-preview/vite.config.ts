import { defineWabouConfig } from "../../packages/vite/src/index.ts";

export default defineWabouConfig({
  root: import.meta.dirname,
  target: "browser",
  entry: "index.tsx",
  outDir: "../../target/web-preview/site",
  browser: {
    canvas: "#preview",
    fonts: ["/DejaVuSans.ttf"],
    runtimeModule: "../../target/web-preview/pkg/wabou_web.js",
  },
  vite: { server: { host: "0.0.0.0" } },
});
