import { ViewProps, WabouStyle } from "./view.mjs";
import { JSX } from "solid-js";
import { Handle, NativeScrollbarStyle } from "@wabou/core/renderer";
//#region src/primitives/scroll-area.d.ts
export interface ScrollAreaProps extends Omit<ViewProps, "children" | "class" | "style" | "ref" | "scrollbar" | "onScroll"> {
  children?: JSX.Element;
  /**
   * Classes applied to the clipped scrolling viewport.
   *
   * The viewport has no implicit flex growth. Give it an explicit height or
   * use `flex-1 min-h-0` inside a bounded flex container.
   */
  class?: string;
  /** Classes applied to the intrinsic-height content wrapper. */
  contentClass?: string;
  style?: WabouStyle;
  ref?: (node: Handle) => void;
  scrollbar?: NativeScrollbarStyle;
  onScroll?: (event: {
    scrollX?: number;
    scrollY?: number;
  }) => void;
}
/**
 * Vertical native scroll viewport with explicit sizing.
 *
 * The inner wrapper deliberately cannot shrink. This makes its intrinsic
 * height become the viewport's scroll extent instead of allowing a flex
 * parent to compress overflowing sections until no scroll range remains.
 * The viewport also locks its cross axis. Otherwise focus reveal can move a
 * nominally vertical viewport sideways when a descendant is slightly wider,
 * making split-pane edges appear clipped. It deliberately does not grow:
 * implicit `flex-1` makes a
 * nested scroll area expand with an ancestor's intrinsic content instead of
 * establishing its own scroll range.
 */
export declare function ScrollArea(props: ScrollAreaProps): JSX.Element;
//#endregion
//# sourceMappingURL=scroll-area.d.mts.map