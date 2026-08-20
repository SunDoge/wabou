import type { JSX } from "solid-js";
import {
  Popover as HeadlessPopover,
  type PopoverMotionOptions,
  type PopoverProps as HeadlessPopoverProps,
  Text,
  View,
} from "../primitives";
import { join } from "./class-names";
import { componentsElevation, useComponentsTheme } from "./theme";

export type PopoverProps = HeadlessPopoverProps;

/** Shared visual-motion contract for components backed by a popup surface. */
export interface PopupMotionProps {
  /** Override the default popup transition, or disable it without changing presence semantics. */
  motion?: false | PopoverMotionOptions;
}

/** A ready-to-use floating surface backed by Wabou's collision-aware overlay. */
export function Popover(props: PopoverProps): JSX.Element {
  const theme = useComponentsTheme();
  return (
    <HeadlessPopover
      {...props}
      contentClass={join(
        "min-w-48 max-w-sm min-h-0 p-4 flex flex-col gap-3 rounded-lg border border-subtle bg-surface",
        props.contentClass,
      )}
      contentShadows={
        props.contentShadows === undefined
          ? componentsElevation(theme(), "floating")
          : props.contentShadows
      }
    />
  );
}

export function PopoverHeader(props: {
  children?: JSX.Element;
  class?: string;
}): JSX.Element {
  return (
    <View class={join("min-w-0 flex flex-col gap-1", props.class)}>
      {props.children}
    </View>
  );
}

export function PopoverTitle(props: {
  children?: JSX.Element;
  class?: string;
}): JSX.Element {
  return (
    <Text
      role="heading"
      class={join(
        "whitespace-normal text-sm font-semibold text-primary",
        props.class,
      )}
    >
      {props.children}
    </Text>
  );
}

export function PopoverDescription(props: {
  children?: JSX.Element;
  class?: string;
}): JSX.Element {
  return (
    <Text
      class={join("min-w-0 whitespace-normal text-xs text-muted", props.class)}
    >
      {props.children}
    </Text>
  );
}

export function PopoverFooter(props: {
  children?: JSX.Element;
  class?: string;
}): JSX.Element {
  return (
    <View
      class={join("min-w-0 flex items-center justify-end gap-2", props.class)}
    >
      {props.children}
    </View>
  );
}
