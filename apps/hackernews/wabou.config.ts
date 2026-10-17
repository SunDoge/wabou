import { defineConfig } from "@wabou/vite";

export default defineConfig({
  ignoreClasses: ["lucide", "lucide-*"],
  outDir: "../../dist/hackernews/resources",
  globalName: "HackerNewsApp",
});
