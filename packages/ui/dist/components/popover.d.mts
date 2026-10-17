import { PopoverMotionOptions, PopoverProps as PopoverProps$1 } from "../primitives/popover.mjs";
import "../primitives.mjs";
import { JSX } from "solid-js";
//#region src/components/popover.d.ts
export type PopoverProps = PopoverProps$1;
/** Shared visual-motion contract for components backed by a popup surface. */
export interface PopupMotionProps {
  /** Override the default popup transition, or disable it without changing presence semantics. */
  motion?: false | PopoverMotionOptions;
}
/** A ready-to-use floating surface backed by Wabou's collision-aware overlay. */
export declare function Popover(props: PopoverProps): JSX.Element;
export declare function PopoverHeader(props: {
  children?: JSX.Element;
  class?: string;
}): JSX.Element;
export declare function PopoverTitle(props: {
  children?: JSX.Element;
  class?: string;
}): JSX.Element;
export declare function PopoverDescription(props: {
  children?: JSX.Element;
  class?: string;
}): JSX.Element;
export declare function PopoverFooter(props: {
  children?: JSX.Element;
  class?: string;
}): JSX.Element;
//#endregion
//# sourceMappingURL=popover.d.mts.map