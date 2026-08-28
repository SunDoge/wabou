import { paraglideVitePlugin } from "@inlang/paraglide-js";
import { defaultWabouColorThemes, defineWabouConfig } from "@wabou/vite";

export default defineWabouConfig(({ mode }) => ({
  entry: mode === "layout-test" ? "ui/layout-fixtures.tsx" : "ui/index.tsx",
  outDir: "../../dist/gallery/resources",
  vite: {
    plugins: [
      paraglideVitePlugin({
        project: "./project.inlang",
        outdir: "./ui/paraglide",
        strategy: ["baseLocale"],
        isServer: "false",
        emitTsDeclarations: true,
      }),
    ],
  },
  intl: {
    locales: ["en", "zh", "de"],
    timeZones: "golden",
  },
  theme: {
    default: "light",
    themes: {
      ...defaultWabouColorThemes.themes,
      violet: {
        appearance: "dark",
        colors: {
          canvas: "#0f0b1a",
          surface: "#1a1328",
          "surface-muted": "#151020",
          input: "#110d1c",
          control: "#2d2340",
          "control-hover": "#403158",
          "control-pressed": "#594276",
          selected: "#4c356b",
          primary: "#faf7ff",
          secondary: "#ddd6fe",
          muted: "#a78bba",
          subtle: "#352744",
          strong: "#5b4670",
          accent: "#8b5cf6",
          "accent-hover": "#7c3aed",
          "accent-pressed": "#6d28d9",
          "on-accent": "#ffffff",
          danger: "#fb7185",
          "danger-hover": "#f43f5e",
          "danger-pressed": "#e11d48",
          "danger-surface": "#4c1427",
          "danger-primary": "#fecdd3",
          "success-surface": "#123c35",
          "success-primary": "#99f6e4",
          focus: "#c4b5fd",
        },
      },
    },
  },
}));
