/** A build-time sRGB color accepted by the Wabou theme compiler. */
export type WabouThemeColor = `#${string}`;

/** Theme shape shared by the Vite plugin, style compiler, and editor tooling. */
export interface WabouColorThemeOptions {
  default: string;
  themes: Record<
    string,
    {
      appearance: "light" | "dark";
      colors: Record<string, WabouThemeColor>;
    }
  >;
}

/**
 * Semantic colors used by `@wabou/ui` when an application does not provide a
 * theme. This module deliberately has no Vite or compiler dependencies so it
 * is also safe to load from editor tooling.
 */
export const defaultWabouColorThemes = {
  default: "light",
  themes: {
    dark: {
      appearance: "dark",
      colors: {
        canvas: "#121418",
        surface: "#1a1d22",
        "surface-muted": "#16191e",
        input: "#20242a",
        control: "#24282f",
        "control-hover": "#2d323a",
        "control-pressed": "#363c45",
        selected: "#233754",
        primary: "#f2f4f7",
        secondary: "#bac0c9",
        muted: "#8e97a4",
        subtle: "#30353d",
        strong: "#464d58",
        accent: "#4c8dff",
        "accent-hover": "#6aa1ff",
        "accent-pressed": "#397ce8",
        "on-accent": "#121418",
        danger: "#ef4444",
        "danger-hover": "#dc2626",
        "danger-pressed": "#b91c1c",
        "danger-surface": "#450a0a",
        "danger-primary": "#fecaca",
        "success-surface": "#064e3b",
        "success-primary": "#a7f3d0",
        focus: "#74a8ff",
      },
    },
    light: {
      appearance: "light",
      colors: {
        canvas: "#ffffff",
        surface: "#ffffff",
        "surface-muted": "#f7f8fa",
        input: "#ffffff",
        control: "#f1f3f6",
        "control-hover": "#e8ebef",
        "control-pressed": "#dde2e8",
        selected: "#e6efff",
        primary: "#171a1f",
        secondary: "#535b66",
        muted: "#606a77",
        subtle: "#dfe3e8",
        strong: "#c5cbd3",
        accent: "#2563eb",
        "accent-hover": "#1d4ed8",
        "accent-pressed": "#1e40af",
        "on-accent": "#ffffff",
        danger: "#dc2626",
        "danger-hover": "#b91c1c",
        "danger-pressed": "#991b1b",
        "danger-surface": "#fef2f2",
        "danger-primary": "#991b1b",
        "success-surface": "#ecfdf5",
        "success-primary": "#047857",
        focus: "#3b82f6",
      },
    },
  },
} as const satisfies WabouColorThemeOptions;

export type DefaultWabouSemanticColorToken =
  keyof (typeof defaultWabouColorThemes)["themes"]["light"]["colors"];

/** Semantic colors guaranteed by Wabou's built-in component theme. */
export const defaultWabouSemanticColorTokens = Object.freeze(
  Object.keys(
    defaultWabouColorThemes.themes[defaultWabouColorThemes.default].colors,
  ) as DefaultWabouSemanticColorToken[],
);
