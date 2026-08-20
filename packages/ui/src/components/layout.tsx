import { createContext, type JSX, Show, useContext } from "solid-js";
import { match } from "ts-pattern";
import { Center, Column, Text, View } from "../primitives";
import { join } from "./class-names";
import { Dialog } from "./dialog";
export function Empty(props: { children?: JSX.Element; class?: string }) {
  return (
    <Column
      class={join(
        "w-full min-h-64 p-8 items-center justify-center gap-4 rounded-lg border border-subtle bg-surface shadow-xs",
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

/**
 * A horizontal primary/aside boundary with explicit flex shrink semantics.
 * Use `SplitPaneMain` for the elastic region and `SplitPaneAside` for a
 * class-sized fixed rail. Both regions clip at their own boundary, so content
 * cannot paint across the divider or a rounded parent clip.
 */
export function SplitPane(props: { children?: JSX.Element; class?: string }) {
  return (
    <View
      class={join("w-full min-w-0 flex flex-row overflow-hidden", props.class)}
    >
      {props.children}
    </View>
  );
}

export function SplitPaneMain(props: {
  children?: JSX.Element;
  class?: string;
}) {
  return (
    <View class={join("flex-1 min-w-0 overflow-hidden", props.class)}>
      {props.children}
    </View>
  );
}

export function SplitPaneAside(props: {
  children?: JSX.Element;
  class?: string;
}) {
  return (
    <View class={join("flex-none min-w-0 overflow-hidden", props.class)}>
      {props.children}
    </View>
  );
}

interface AdaptiveSplitPaneContextValue {
  compact(): boolean;
}

const AdaptiveSplitPaneContext = createContext<AdaptiveSplitPaneContextValue>();

/**
 * Master/detail layout whose detail region can move from an inline rail to a
 * modal surface without changing the application's selection model.
 */
export function AdaptiveSplitPane(props: {
  children?: JSX.Element;
  compact: boolean;
  class?: string;
}) {
  const context: AdaptiveSplitPaneContextValue = {
    compact: () => props.compact,
  };
  return (
    <AdaptiveSplitPaneContext value={context}>
      <SplitPane class={props.class}>{props.children}</SplitPane>
    </AdaptiveSplitPaneContext>
  );
}

export function AdaptiveSplitPaneMain(props: {
  children?: JSX.Element;
  class?: string;
}) {
  return <SplitPaneMain class={props.class}>{props.children}</SplitPaneMain>;
}

export function AdaptiveSplitPaneDetail(props: {
  children?: JSX.Element;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  "aria-label": string;
  class?: string;
  modalClass?: string;
}) {
  const context = useContext(AdaptiveSplitPaneContext);
  return (
    <Show
      when={context.compact()}
      fallback={
        <SplitPaneAside class={props.class}>{props.children}</SplitPaneAside>
      }
    >
      <Dialog
        open={props.open}
        onOpenChange={(open) => props.onOpenChange(open)}
        aria-label={props["aria-label"]}
        contentClass={join("h-11/12 p-0 overflow-hidden", props.modalClass)}
      >
        {props.children}
      </Dialog>
    </Show>
  );
}
