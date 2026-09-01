import { mergeClasses } from "@wabou/core/style";
import { type JSX, omit } from "solid-js";
import { Text, type TextProps, View, type ViewProps } from "../primitives";
import {
  componentsElevation,
  componentsSurfaceClass,
  useComponentsTheme,
} from "./theme";

export interface CardProps extends Omit<ViewProps, "class"> {
  class?: string;
}

export function Card(props: CardProps): JSX.Element {
  const theme = useComponentsTheme();
  const rest = omit(props, "class", "children", "shadows");
  return (
    <View
      {...rest}
      class={mergeClasses(
        "min-w-0 min-h-0 flex-none flex flex-col overflow-hidden",
        componentsSurfaceClass("raised"),
        props.class,
      )}
      shadows={
        props.shadows === undefined
          ? componentsElevation(theme(), "raised")
          : props.shadows
      }
    >
      {props.children}
    </View>
  );
}

export function CardHeader(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      class={mergeClasses(
        "relative min-w-0 flex flex-col gap-1.5 px-5 pt-5 pr-14",
        props.class,
      )}
    >
      {props.children}
    </View>
  );
}

export function CardTitle(props: TextProps): JSX.Element {
  return (
    <Text
      {...props}
      class={mergeClasses(
        "min-w-0 text-base font-semibold text-primary",
        props.class,
      )}
    >
      {props.children}
    </Text>
  );
}

export function CardDescription(props: TextProps): JSX.Element {
  return (
    <Text
      {...props}
      class={mergeClasses(
        "w-full min-w-0 whitespace-normal text-sm text-muted",
        props.class,
      )}
    >
      {props.children}
    </Text>
  );
}

/** Top-end action slot owned by the relative CardHeader surface. */
export function CardAction(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      class={mergeClasses(
        "absolute top-5 right-5 flex-none flex items-center justify-end",
        props.class,
      )}
    >
      {props.children}
    </View>
  );
}

export function CardContent(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      class={mergeClasses(
        "min-w-0 min-h-0 flex flex-col gap-4 px-5 pt-4 pb-5",
        props.class,
      )}
    >
      {props.children}
    </View>
  );
}

export function CardFooter(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      class={mergeClasses(
        "min-w-0 flex items-center gap-2 px-5 pb-5",
        props.class,
      )}
    >
      {props.children}
    </View>
  );
}
