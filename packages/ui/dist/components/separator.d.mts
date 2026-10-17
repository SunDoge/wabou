import { ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { JSX } from "solid-js";
//#region src/components/separator.d.ts
export interface SeparatorProps extends Omit<ViewProps, "aria-hidden" | "aria-orientation" | "class" | "role"> {
  /** Direction of the dividing line. */
  orientation?: "horizontal" | "vertical";
  /**
   * Decorative separators stay out of the semantic tree. Set to false when
   * the divider represents a meaningful boundary between regions or controls.
   */
  decorative?: boolean;
  class?: string;
}
/** A visual divider with an opt-in semantic separator contract. */
export declare function Separator(props: SeparatorProps): JSX.Element;
export interface LabeledSeparatorProps extends Omit<ViewProps, "class"> {
  class?: string;
}
/**
 * A horizontal divider whose compact center content names the boundary.
 *
 * Keep the content short. Interactive content is supported so disclosures can
 * explain what happened between two regions without turning into a full row.
 */
export declare function LabeledSeparator(props: LabeledSeparatorProps): JSX.Element;
//#endregion
//# sourceMappingURL=separator.d.mts.map