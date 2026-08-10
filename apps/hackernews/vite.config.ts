import { defineWabouConfig } from "@wabou/vite";

export default defineWabouConfig({
  ignoreClasses: ["lucide", "lucide-*"],
  outDir: "../../dist/hackernews/resources",
  globalName: "HackerNewsApp",
});
