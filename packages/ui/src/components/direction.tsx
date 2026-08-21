import { createComponent, createContext, type JSX, useContext } from "solid-js";
import { Text, type TextProps, View, type ViewProps } from "../primitives";
import { join } from "./class-names";

export type Direction = "ltr" | "rtl";

const DirectionContext = createContext<Direction>("ltr");

export interface DirectionProviderProps {
  dir: Direction;
  children?: JSX.Element;
}

/** Own logical direction in JavaScript instead of relying on web inheritance. */
export function DirectionProvider(props: DirectionProviderProps): JSX.Element {
  return createComponent(DirectionContext, {
    get value() {
      return props.dir;
    },
    get children() {
      return props.children;
    },
  });
}

export function useDirection(): Direction {
  return useContext(DirectionContext);
}

export interface DirectionalRowProps extends Omit<ViewProps, "class"> {
  class?: string;
}

export function DirectionalRow(props: DirectionalRowProps): JSX.Element {
  const direction = () => useDirection();
  return (
    <View
      {...props}
      class={join(
        "flex",
        direction() === "rtl" ? "flex-row-reverse" : "flex-row",
        props.class,
      )}
    />
  );
}

export interface DirectionalTextProps extends Omit<TextProps, "class"> {
  class?: string;
}

export function DirectionalText(props: DirectionalTextProps): JSX.Element {
  const direction = () => useDirection();
  return (
    <Text
      {...props}
      class={join(
        direction() === "rtl" ? "text-right" : "text-left",
        props.class,
      )}
    />
  );
}
