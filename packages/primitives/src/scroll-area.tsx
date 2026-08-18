import type { Handle, NativeScrollbarStyle } from "@wabou/core/renderer";
import type { JSX } from "solid-js";
import { join } from "./class-names";
import { View, type WabouStyle } from "./view";

export const scrollAreaViewportClass = (className?: string) =>
  join("min-h-0 overflow-y-auto", className);

export interface ScrollAreaProps {
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
  onScroll?: (event: { scrollX?: number; scrollY?: number }) => void;
}

/**
 * Vertical native scroll viewport with explicit sizing.
 *
 * The inner wrapper deliberately cannot shrink. This makes its intrinsic
 * height become the viewport's scroll extent instead of allowing a flex
 * parent to compress overflowing sections until no scroll range remains.
 * The viewport itself deliberately does not grow: implicit `flex-1` makes a
 * nested scroll area expand with an ancestor's intrinsic content instead of
 * establishing its own scroll range.
 */
export function ScrollArea(props: ScrollAreaProps): JSX.Element {
  return (
    <View
      ref={props.ref}
      class={scrollAreaViewportClass(props.class)}
      style={props.style}
      scrollbar={props.scrollbar}
      onScroll={props.onScroll}
    >
      <View
        class={join("flex-none flex flex-col min-h-full", props.contentClass)}
      >
        {props.children}
      </View>
    </View>
  );
}
