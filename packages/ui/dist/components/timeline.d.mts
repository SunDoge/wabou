import { ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { JSX } from "solid-js";
//#region src/components/timeline.d.ts
export interface TimelineItem {
  id: string;
  title: string;
  description?: string;
  time?: string;
  status?: "complete" | "current" | "pending";
}
export interface TimelineProps extends Omit<ViewProps, "children"> {
  items: readonly TimelineItem[];
}
export declare function Timeline(props: TimelineProps): JSX.Element;
//#endregion
//# sourceMappingURL=timeline.d.mts.map