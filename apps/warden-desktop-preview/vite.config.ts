import { fileURLToPath } from "node:url";
import { defineWabouConfig } from "@wabou/vite";

export default defineWabouConfig({
  root: fileURLToPath(new URL("../warden-desktop", import.meta.url)),
  entry: fileURLToPath(new URL("../warden-desktop/ui/preview.tsx", import.meta.url)),
  outDir: fileURLToPath(
    new URL("../../dist/warden-desktop-preview/resources", import.meta.url),
  ),
});
