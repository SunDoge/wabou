import { Easing } from "../animation.mjs";
import { ViewProps, WabouStyle } from "./view.mjs";
import { JSX } from "solid-js";
//#region src/primitives/collapsible-presence.d.ts
export interface CollapsiblePresenceProps {
  open: boolean;
  children?: JSX.Element;
  class?: string;
  contentClass?: string;
  /** Props applied to the retained content node inside the animated viewport. */
  contentProps?: Omit<ViewProps, "children" | "class" | "style">;
  style?: WabouStyle;
  contentStyle?: WabouStyle;
  duration?: number;
  ease?: Easing;
  reducedMotion?: boolean;
  /** Animate an initially-open disclosure from zero height. Defaults to false. */
  animateInitial?: boolean;
}
/**
 * Measured disclosure content with explicit presence and subtree isolation.
 * Height participates in layout while a subtree opacity layer masks glyphs
 * crossing the moving clip edge.
 */
export declare function CollapsiblePresence(props: CollapsiblePresenceProps): JSX.Element;
//#endregion
//# sourceMappingURL=collapsible-presence.d.mts.map