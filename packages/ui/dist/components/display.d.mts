import { TextProps, ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { JSX } from "solid-js";
//#region src/components/display.d.ts
export interface SkeletonProps {
  class?: string;
  /** Disable the shimmer while preserving the stable loading placeholder. */
  animated?: boolean;
}
export declare function Skeleton(props: SkeletonProps): JSX.Element;
export declare function Spinner(props: {
  label?: string;
  class?: string;
  /** Hide spinner semantics when a parent status already announces progress. */
  decorative?: boolean;
  /** Duration of one revolution in seconds. */
  duration?: number;
  /** Playback-rate multiplier. */
  speed?: number;
  paused?: boolean;
}): JSX.Element;
export type KbdProps = TextProps;
export declare function Kbd(props: KbdProps): JSX.Element;
export type KbdGroupProps = ViewProps;
export declare function KbdGroup(props: KbdGroupProps): JSX.Element;
//#endregion
//# sourceMappingURL=display.d.mts.map