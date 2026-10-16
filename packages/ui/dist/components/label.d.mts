import { TextProps } from "../primitives/view.mjs";
import "../primitives.mjs";
import { JSX } from "solid-js";
import { Handle } from "@wabou/core/renderer";
//#region src/components/label.d.ts
export interface LabelProps extends Omit<TextProps, "class" | "role"> {
  class?: string;
  disabled?: boolean;
  /**
   * Explicit native control target. Unlike HTML `for`, this cannot silently
   * point at a missing string id and remains safe across retained-tree reuse.
   */
  control?: Handle | (() => Handle | undefined);
}
/** Text label that forwards pointer activation to an explicit native control. */
export declare function Label(props: LabelProps): JSX.Element;
//#endregion
//# sourceMappingURL=label.d.mts.map