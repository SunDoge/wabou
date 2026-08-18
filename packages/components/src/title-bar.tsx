import { useWindow } from "@wabou/core";
import { View, type ViewProps, type WabouStyle } from "@wabou/primitives";
import type { JSX } from "solid-js";
import { join } from "./class-names";

export const titleBarClass = "border-b border-subtle";

export interface TitleBarProps extends ViewProps {
  class?: string;
  children?: JSX.Element;
}

export const titleBarLayoutStyle = {
  display: "flex",
  "flex-direction": "row",
  "align-items": "center",
  "flex-shrink": 0,
  height: "40px",
} as const;

export const titleBarDragRegionLayoutStyle = {
  display: "flex",
  "flex-direction": "row",
  "align-items": "center",
  "flex-grow": 1,
  "flex-shrink": 1,
  "flex-basis": "0%",
  height: "100%",
} as const;

/** Layout shell for an application-owned title bar. */
export function TitleBar(props: TitleBarProps): JSX.Element {
  return (
    <View
      {...props}
      class={join(titleBarClass, props.class)}
      style={{ ...titleBarLayoutStyle, ...props.style }}
    />
  );
}

export interface TitleBarDragRegionProps
  extends Omit<ViewProps, "onPointerDown" | "onDblClick"> {
  class?: string;
  style?: WabouStyle;
  children?: JSX.Element;
}

/** Explicit non-interactive region that moves the native window. */
export function TitleBarDragRegion(
  props: TitleBarDragRegionProps,
): JSX.Element {
  const window = useWindow();
  return (
    <View
      {...props}
      class={props.class}
      style={{ ...titleBarDragRegionLayoutStyle, ...props.style }}
      onPointerDown={(event: { button: number }) => {
        if (event.button === 0) window.startDragging();
      }}
      onDblClick={() => window.setMaximized(!window.maximized())}
    />
  );
}
