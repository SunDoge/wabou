/**
 * Semantic colors guaranteed by Wabou's built-in component theme.
 *
 * Keep this independent from the Vite plugin so editor tooling can load the
 * contract without initializing Vite or the style compiler.
 */
export const defaultWabouSemanticColorTokens = [
  "canvas",
  "surface",
  "surface-muted",
  "input",
  "control",
  "control-hover",
  "control-pressed",
  "selected",
  "primary",
  "secondary",
  "muted",
  "subtle",
  "strong",
  "accent",
  "accent-hover",
  "accent-pressed",
  "on-accent",
  "danger",
  "danger-hover",
  "danger-pressed",
  "danger-surface",
  "danger-primary",
  "success-surface",
  "success-primary",
  "focus",
] as const;

export type DefaultWabouSemanticColorToken =
  (typeof defaultWabouSemanticColorTokens)[number];
