//#region src/theme-contract.d.ts
/** A build-time sRGB color accepted by the Wabou theme compiler. */
type WabouThemeColor = `#${string}`;
/** Theme shape shared by the Vite plugin, style compiler, and editor tooling. */
interface WabouColorThemeOptions {
  default: string;
  themes: Record<string, {
    appearance: "light" | "dark";
    colors: Record<string, WabouThemeColor>;
  }>;
}
/**
 * Semantic colors used by `@wabou/ui` when an application does not provide a
 * theme. This module deliberately has no Vite or compiler dependencies so it
 * is also safe to load from editor tooling.
 */
declare const defaultWabouColorThemes: {
  readonly default: "light";
  readonly themes: {
    readonly dark: {
      readonly appearance: "dark";
      readonly colors: {
        readonly canvas: "#121418";
        readonly surface: "#1a1d22";
        readonly "surface-muted": "#16191e";
        readonly input: "#20242a";
        readonly control: "#24282f";
        readonly "control-hover": "#2d323a";
        readonly "control-pressed": "#363c45";
        readonly selected: "#233754";
        readonly primary: "#f2f4f7";
        readonly secondary: "#bac0c9";
        readonly muted: "#8e97a4";
        readonly subtle: "#30353d";
        readonly strong: "#464d58";
        readonly accent: "#4c8dff";
        readonly "accent-hover": "#6aa1ff";
        readonly "accent-pressed": "#397ce8";
        readonly "on-accent": "#121418";
        readonly danger: "#ef4444";
        readonly "danger-hover": "#dc2626";
        readonly "danger-pressed": "#b91c1c";
        readonly "danger-surface": "#450a0a";
        readonly "danger-primary": "#fecaca";
        readonly "success-surface": "#064e3b";
        readonly "success-primary": "#a7f3d0";
        readonly focus: "#74a8ff";
      };
    };
    readonly light: {
      readonly appearance: "light";
      readonly colors: {
        readonly canvas: "#ffffff";
        readonly surface: "#ffffff";
        readonly "surface-muted": "#f7f8fa";
        readonly input: "#ffffff";
        readonly control: "#f1f3f6";
        readonly "control-hover": "#e8ebef";
        readonly "control-pressed": "#dde2e8";
        readonly selected: "#e6efff";
        readonly primary: "#171a1f";
        readonly secondary: "#535b66";
        readonly muted: "#606a77";
        readonly subtle: "#dfe3e8";
        readonly strong: "#c5cbd3";
        readonly accent: "#2563eb";
        readonly "accent-hover": "#1d4ed8";
        readonly "accent-pressed": "#1e40af";
        readonly "on-accent": "#ffffff";
        readonly danger: "#dc2626";
        readonly "danger-hover": "#b91c1c";
        readonly "danger-pressed": "#991b1b";
        readonly "danger-surface": "#fef2f2";
        readonly "danger-primary": "#991b1b";
        readonly "success-surface": "#ecfdf5";
        readonly "success-primary": "#047857";
        readonly focus: "#3b82f6";
      };
    };
  };
};
type DefaultWabouSemanticColorToken = keyof (typeof defaultWabouColorThemes)["themes"]["light"]["colors"];
/** Semantic colors guaranteed by Wabou's built-in component theme. */
declare const defaultWabouSemanticColorTokens: readonly ("canvas" | "surface" | "surface-muted" | "input" | "control" | "control-hover" | "control-pressed" | "selected" | "primary" | "secondary" | "muted" | "subtle" | "strong" | "accent" | "accent-hover" | "accent-pressed" | "on-accent" | "danger" | "danger-hover" | "danger-pressed" | "danger-surface" | "danger-primary" | "success-surface" | "success-primary" | "focus")[];
//#endregion
export { defaultWabouSemanticColorTokens as a, defaultWabouColorThemes as i, WabouColorThemeOptions as n, WabouThemeColor as r, DefaultWabouSemanticColorToken as t };
//# sourceMappingURL=theme-contract-B79FL9uR.d.mts.map