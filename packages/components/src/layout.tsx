import { Center, Column, Text, View } from "@wabou/primitives";
import type { JSX } from "solid-js";
import { match } from "ts-pattern";
const join = (...values: Array<string | undefined | false>) =>
  values.filter(Boolean).join(" ");
export function Empty(props: { children?: JSX.Element; class?: string }) {
  return (
    <Column
      class={join(
        "w-full min-h-64 p-8 items-center justify-center gap-5 rounded-xl border border-subtle bg-surface",
        props.class,
      )}
    >
      {props.children}
    </Column>
  );
}
export function EmptyHeader(props: { children?: JSX.Element; class?: string }) {
  return (
    <Column class={join("max-w-md items-center gap-2", props.class)}>
      {props.children}
    </Column>
  );
}
export function EmptyMedia(props: { children?: JSX.Element; class?: string }) {
  return (
    <Center
      class={join(
        "w-12 h-12 flex-none rounded-lg bg-control text-secondary",
        props.class,
      )}
    >
      {props.children}
    </Center>
  );
}
export function EmptyTitle(props: { children?: JSX.Element; class?: string }) {
  return (
    <Text class={join("text-base font-semibold text-primary", props.class)}>
      {props.children}
    </Text>
  );
}
export function EmptyDescription(props: {
  children?: JSX.Element;
  class?: string;
}) {
  return (
    <Text
      class={join(
        "w-full min-w-0 whitespace-normal text-center text-sm text-muted",
        props.class,
      )}
    >
      {props.children}
    </Text>
  );
}
export function EmptyContent(props: {
  children?: JSX.Element;
  class?: string;
}) {
  return (
    <View class={join("flex items-center gap-2", props.class)}>
      {props.children}
    </View>
  );
}
export function ButtonGroup(props: {
  children?: JSX.Element;
  orientation?: "horizontal" | "vertical";
  class?: string;
}) {
  const layout = () =>
    match(props.orientation ?? "horizontal")
      .with("horizontal", () => "flex-row items-center")
      .with("vertical", () => "flex-col items-stretch")
      .exhaustive();
  return (
    <View role="group" class={join("flex gap-1", layout(), props.class)}>
      {props.children}
    </View>
  );
}
export function ButtonGroupText(props: {
  children?: JSX.Element;
  class?: string;
}) {
  return (
    <Text class={join("px-2 text-sm text-muted", props.class)}>
      {props.children}
    </Text>
  );
}
