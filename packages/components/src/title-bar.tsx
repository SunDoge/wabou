import { useWindow } from "@wabou/core";
import { View, type ViewProps } from "@wabou/primitives";
import type { JSX } from "solid-js";

const join = (...values: Array<string | undefined | false>) =>
  values.filter(Boolean).join(" ");

export interface TitleBarProps extends ViewProps {
  class?: string;
  children?: JSX.Element;
}

/** Layout shell for an application-owned title bar. */
export function TitleBar(props: TitleBarProps): JSX.Element {
  return (
    <View
      {...props}
      class={join(
        "h-10 flex-none flex-row items-center border-b border-default",
        props.class,
      )}
    />
  );
}

export interface TitleBarDragRegionProps
  extends Omit<ViewProps, "onPointerDown" | "onDblClick"> {
  class?: string;
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
      class={join("h-full flex-1 flex-row items-center", props.class)}
      onPointerDown={(event: { button: number }) => {
        if (event.button === 0) window.startDragging();
      }}
      onDblClick={() => window.setMaximized(!window.maximized())}
    />
  );
}
