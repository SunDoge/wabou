import { defineWabouConfig } from "@wabou/vite";

export default defineWabouConfig(({ mode }) => ({
  ignoreClasses: ["lucide", "lucide-*"],
  entry: mode === "layout-test" ? "ui/fixture.tsx" : undefined,
  themeContrast: "error",
  theme: {
    default: "dark",
    themes: {
      dark: {
        appearance: "dark",
        colors: {
          canvas: "#080b10",
          surface: "#11161e",
          "surface-muted": "#0d1219",
          input: "#11161e",
          control: "#171e28",
          "control-hover": "#202a37",
          "control-pressed": "#283546",
          selected: "#1d3958",
          primary: "#f4f7fb",
          secondary: "#bdc7d5",
          muted: "#91a0b4",
          subtle: "#263140",
          strong: "#3a485b",
          accent: "#68b5ff",
          "accent-hover": "#83c2ff",
          "accent-pressed": "#4b9fe9",
          "on-accent": "#07111c",
          "on-danger": "#090d13",
          "on-success": "#090d13",
          danger: "#ff7b86",
          "danger-hover": "#ff929b",
          "danger-pressed": "#df616d",
          "danger-surface": "#3a1c23",
          "danger-primary": "#ffb6bd",
          "success-surface": "#12362c",
          "success-primary": "#7ee0b6",
          focus: "#77bbff",
        },
      },
    },
  },
}));
