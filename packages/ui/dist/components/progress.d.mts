import { TextProps, ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { JSX } from "solid-js";
//#region src/components/progress.d.ts
export interface ProgressValueDetails {
  value: number;
  min: number;
  max: number;
  percent: number;
}
export interface ProgressRootProps extends Omit<ViewProps, "children" | "class" | "role"> {
  value?: number;
  minValue?: number;
  maxValue?: number;
  indeterminate?: boolean;
  /** Direct accessible label; visual labels do not rely on DOM id wiring. */
  label?: string;
  getValueLabel?: (details: ProgressValueDetails) => string;
  children?: JSX.Element;
  class?: string;
}
export type ProgressSize = "xs" | "sm" | "default" | "lg";
export declare function normalizeProgressValue(value: number | undefined, minValue: number | undefined, maxValue: number | undefined): ProgressValueDetails;
/** Semantic progress state with explicit, composable visual parts. */
export declare function ProgressRoot(props: ProgressRootProps): JSX.Element;
export interface ProgressTrackProps extends ViewProps {
  size?: ProgressSize;
}
export declare function ProgressTrack(props: ProgressTrackProps): JSX.Element;
export declare function ProgressFill(props: ViewProps): JSX.Element;
export declare function ProgressLabel(props: TextProps): JSX.Element;
export declare function ProgressValueLabel(props: TextProps): JSX.Element;
export interface ProgressProps extends Omit<ProgressRootProps, "children" | "class"> {
  /** Classes applied to the visual track, preserving the original shorthand. */
  class?: string;
  size?: ProgressSize;
}
/** Compact progress bar; use ProgressRoot and parts for custom composition. */
export declare function Progress(props: ProgressProps): JSX.Element;
export interface ProgressCircleProps extends Omit<ProgressRootProps, "children" | "class"> {
  class?: string;
  size?: ProgressSize;
}
/** Compact circular progress indicator using the same semantic range contract. */
export declare function ProgressCircle(props: ProgressCircleProps): JSX.Element;
//#endregion
//# sourceMappingURL=progress.d.mts.map