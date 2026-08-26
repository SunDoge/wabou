import type { WabouStyle } from "@wabou/core/style";
import { type JSX, omit } from "solid-js";
import { View, type ViewProps } from "../primitives";
import { mergeClasses } from "@wabou/core/style";

export interface AspectRatioProps extends Omit<ViewProps, "style"> {
  /** Width divided by height. Defaults to a square. */
  ratio?: number;
  style?: WabouStyle;
}

export function aspectRatioStyle(
  ratio: number | undefined,
  style?: WabouStyle,
): WabouStyle {
  const resolved = ratio ?? 1;
  if (!Number.isFinite(resolved) || resolved <= 0) {
    throw new RangeError("AspectRatio ratio must be a finite positive number");
  }
  return { ...style, "aspect-ratio": resolved };
}

/** A native Taffy aspect-ratio constraint with explicit overflow ownership. */
export function AspectRatio(props: AspectRatioProps): JSX.Element {
  const rest = omit(props, "ratio", "style", "class", "children");
  return (
    <View
      {...rest}
      data-wabou-owns="clip"
      class={mergeClasses("w-full min-w-0 overflow-hidden", props.class)}
      style={aspectRatioStyle(props.ratio, props.style)}
    >
      {props.children}
    </View>
  );
}
