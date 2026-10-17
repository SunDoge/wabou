import { ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { ComponentTone } from "./tone.mjs";
import { JSX } from "solid-js";
//#region src/components/activity-status.d.ts
export type ActivityStatusTone = Exclude<ComponentTone, "neutral"> | "muted";
export interface ActivityStatusIndicatorProps {
  animated?: boolean;
  tone?: ActivityStatusTone;
  class?: string;
}
/** A consistently sized status dot. Animation is explicit so idle state never
 * keeps the native frame clock alive accidentally. */
export declare function ActivityStatusIndicator(props: ActivityStatusIndicatorProps): JSX.Element;
export interface ActivityStatusProps extends Omit<ViewProps, "children" | "class"> {
  label: string;
  animated?: boolean;
  tone?: ActivityStatusTone;
  class?: string;
  textClass?: string;
}
/** Shrink-safe inline progress or presence status for agent and desktop UI. */
export declare function ActivityStatus(props: ActivityStatusProps): JSX.Element;
//#endregion
//# sourceMappingURL=activity-status.d.mts.map