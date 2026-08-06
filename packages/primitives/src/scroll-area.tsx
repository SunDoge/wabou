import type { Handle, NativeScrollbarStyle } from "@wabou/solid-renderer";
import type { JSX } from "solid-js";
import { View, type WabouStyle } from "./view";

const join = (...values: Array<string | undefined>) =>
  values.filter(Boolean).join(" ");

export interface ScrollAreaProps {
  children?: JSX.Element;
  /** Classes applied to the clipped scrolling viewport. */
  class?: string;
  /** Classes applied to the intrinsic-height content wrapper. */
  contentClass?: string;
  style?: WabouStyle;
  ref?: (node: Handle) => void;
  scrollbar?: NativeScrollbarStyle;
}

/**
 * Vertical native scroll viewport.
 *
 * The inner wrapper deliberately cannot shrink. This makes its intrinsic
 * height become the viewport's scroll extent instead of allowing a flex
 * parent to compress overflowing sections until no scroll range remains.
 */
export function ScrollArea(props: ScrollAreaProps): JSX.Element {
  return (
    <View
      ref={props.ref}
      class={join("flex-1 min-h-0 overflow-y-auto", props.class)}
      style={props.style}
      scrollbar={props.scrollbar}
    >
      <View
        class={join("flex-none flex flex-col min-h-full", props.contentClass)}
      >
        {props.children}
      </View>
    </View>
  );
}
