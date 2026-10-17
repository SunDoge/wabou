import { ViewProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { JSX } from "solid-js";
//#region src/components/group-box.d.ts
export type GroupBoxVariant = "normal" | "fill" | "outline";
export interface GroupBoxProps extends Omit<ViewProps, "children" | "class"> {
  title?: JSX.Element;
  description?: JSX.Element;
  children?: JSX.Element;
  variant?: GroupBoxVariant;
  class?: string;
  headerClass?: string;
  contentClass?: string;
}
export declare function groupBoxContentClass(variant?: GroupBoxVariant, className?: string): string;
/** A lightweight titled surface for related controls and settings rows. */
export declare function GroupBox(props: GroupBoxProps): JSX.Element;
//#endregion
//# sourceMappingURL=group-box.d.mts.map