import { TextProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { JSX } from "solid-js";
//#region src/components/badge.d.ts
export type BadgeVariant = "default" | "secondary" | "outline" | "ghost" | "link" | "info" | "success" | "destructive";
export type BadgeSize = "sm" | "default" | "lg";
export interface BadgeProps extends Omit<TextProps, "class"> {
  variant?: BadgeVariant;
  size?: BadgeSize;
  /** Typography weight selected without competing utility declarations. */
  weight?: "normal" | "medium";
  class?: string;
}
export declare function badgeClass(variant?: BadgeVariant, weight?: NonNullable<BadgeProps["weight"]>, className?: string, size?: BadgeSize): string;
/** Compact status text with shadcn-compatible visual variants. */
export declare function Badge(props: BadgeProps): JSX.Element;
//#endregion
//# sourceMappingURL=badge.d.mts.map