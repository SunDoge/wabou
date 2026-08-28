import { useWindow } from "@wabou/core";
import { type Shadow, shadow } from "@wabou/core/style";
import type { JSX } from "solid-js";
import {
  View,
  type ViewProps,
  type WabouClassList,
  type WabouStyle,
} from "../primitives";
import { mergeClasses } from "@wabou/core/style";
import { useComponentsTheme } from "./theme";

export interface WindowFrameProps extends ViewProps {
  class?: string;
  children?: JSX.Element;
  /** Clip the application surface to desktop-style corners when restored. */
  rounded?: boolean;
}

export function windowFrameBackdropClassList(
  maximized: boolean,
  rounded = true,
): WabouClassList {
  const decorated = rounded && !maximized;
  return {
    "p-3": decorated,
  };
}

export function windowFrameClientClassList(
  maximized: boolean,
  rounded = true,
  classList: WabouClassList = {},
): WabouClassList {
  const decorated = rounded && !maximized;
  return {
    ...classList,
    "rounded-xl": decorated,
    border: decorated,
    "border-subtle": decorated,
    "overflow-hidden": decorated,
  };
}

/** Two restrained client-decoration layers sized to fit the 12px backdrop. */
export function windowFrameShadows(theme: "light" | "dark"): Shadow[] {
  const ambient = theme === "dark" ? 0x0000004d : 0x00000024;
  const contact = theme === "dark" ? 0x00000052 : 0x0000002e;
  return [
    shadow({ offsetY: 2, spread: -1, stdDev: 3, color: ambient, radius: 12 }),
    shadow({ offsetY: 1, spread: 0, stdDev: 1.5, color: contact, radius: 12 }),
  ];
}

/**
 * Root frame for an application-owned title bar and window chrome.
 *
 * Rounded outer corners require the native window to preserve alpha and the
 * Rust host to clear with a transparent base color. Maximized windows are
 * intentionally square so their content reaches every display edge.
 */
export function WindowFrame(props: WindowFrameProps): JSX.Element {
  const window = useWindow();
  const theme = useComponentsTheme();
  const decorated = () => props.rounded !== false && !window.maximized();
  return (
    <View
      class="w-full h-full bg-transparent"
      classList={windowFrameBackdropClassList(
        window.maximized(),
        props.rounded !== false,
      )}
    >
      <View
        {...props}
        class={mergeClasses("w-full h-full", props.class)}
        classList={windowFrameClientClassList(
          window.maximized(),
          props.rounded !== false,
          props.classList,
        )}
        shadows={
          props.shadows !== undefined
            ? props.shadows
            : decorated()
              ? windowFrameShadows(theme())
              : null
        }
      />
    </View>
  );
}

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
      class={mergeClasses(titleBarClass, props.class)}
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
