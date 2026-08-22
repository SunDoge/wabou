import { defineWabouConfig } from "@wabou/vite";

export default defineWabouConfig({
  ignoreClasses: ["lucide", "lucide-*"],
  outDir: "../../dist/manga-ocr-wabou/resources",
  globalName: "MangaOcrWabouApp",
  theme: {
    default: "dark",
    themes: {
      dark: {
        appearance: "dark",
        colors: {
          canvas: "#0c0d10",
          surface: "#15171c",
          "surface-muted": "#1c1f26",
          input: "#1c1f26",
          control: "#232731",
          "control-hover": "#2c3240",
          "control-pressed": "#353d4d",
          selected: "#293140",
          primary: "#f3f4f6",
          secondary: "#c1c7d0",
          muted: "#8992a3",
          subtle: "#2a2f39",
          strong: "#414957",
          accent: "#8b5cf6",
          "accent-hover": "#7c3aed",
          "accent-pressed": "#6d28d9",
          "on-accent": "#ffffff",
          danger: "#ef4444",
          "danger-hover": "#dc2626",
          "danger-pressed": "#b91c1c",
          "danger-surface": "#35171b",
          "danger-primary": "#fca5a5",
          "success-surface": "#102d25",
          "success-primary": "#6ee7b7",
          focus: "#a78bfa"
        }
      }
    }
  }
});

