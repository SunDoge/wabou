import { fileURLToPath } from "node:url";
import { defineWabouConfig } from "@wabou/vite";

export default defineWabouConfig({
  root: fileURLToPath(new URL("../bitwarden", import.meta.url)),
  entry: fileURLToPath(new URL("../bitwarden/ui/preview.tsx", import.meta.url)),
  outDir: fileURLToPath(
    new URL("../../dist/bitwarden-preview/resources", import.meta.url),
  ),
});
