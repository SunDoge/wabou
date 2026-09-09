import { defineConfig } from "unocss";
import { presetWabou } from "../../packages/vite/src/preset/index";

export default defineConfig({ presets: [presetWabou()] });
