import type { Handle } from "@wabou/core/renderer";
import { mergeClasses } from "@wabou/core/style";
import { type JSX, omit, Show } from "solid-js";
import {
  createScrollReset,
  ScrollArea,
  type ScrollAreaProps,
  Text,
  View,
} from "../primitives";

export const pageViewportClass = (className?: string) =>
  mergeClasses("min-w-0 min-h-0 flex-1", className);

export const pageViewportContentClass = (className?: string) =>
  mergeClasses("w-full", className);

export interface PageViewportProps
  extends Omit<ScrollAreaProps, "class" | "contentClass" | "ref"> {
  children?: JSX.Element;
  /** Classes applied to the bounded scrolling viewport. */
  class?: string;
  /** Classes applied to the intrinsic-height page content wrapper. */
  contentClass?: string;
  /** Reset the page to its origin whenever this identity changes. */
  resetKey?: unknown;
  ref?: (node: Handle) => void;
}

/**
 * A full-height application page boundary.
 *
 * This composes native scrolling with an explicitly sized content wrapper and
 * optional identity-based scroll reset. Page implementations can therefore
 * focus on their own layout instead of reconstructing flex/overflow rules.
 */
export function PageViewport(props: PageViewportProps): JSX.Element {
  let viewport: Handle | undefined;
  const forwarded = omit(
    props,
    "children",
    "class",
    "contentClass",
    "resetKey",
    "ref",
    "style",
    "scrollbar",
    "onScroll",
  );
  createScrollReset({
    target: () => viewport,
    key: () => props.resetKey,
  });
  return (
    <ScrollArea
      {...forwarded}
      class={pageViewportClass(props.class)}
      contentClass={pageViewportContentClass(props.contentClass)}
      style={props.style}
      scrollbar={props.scrollbar}
      onScroll={props.onScroll}
      ref={(node) => {
        viewport = node;
        props.ref?.(node);
      }}
    >
      {props.children}
    </ScrollArea>
  );
}

export const pageHeaderClass = (className?: string, stacked = false) =>
  mergeClasses(
    "min-w-0 min-h-12 flex-none flex justify-between gap-4",
    stacked ? "flex-col items-stretch" : "flex-row items-center",
    className,
  );

export const pageHeaderTitleClass = () =>
  "whitespace-nowrap text-2xl font-semibold text-primary";

export const pageHeaderDescriptionClass = (stacked = false) =>
  mergeClasses(
    "text-sm text-secondary",
    stacked ? "whitespace-normal" : "truncate",
  );

export interface PageHeaderProps {
  title: string;
  description?: string;
  /** Optional content rendered immediately after the title. */
  titleAdornment?: JSX.Element;
  /** Page-level controls rendered at the trailing edge. */
  actions?: JSX.Element;
  /** Stack actions below the title for narrow application windows. */
  stacked?: boolean;
  class?: string;
}

/** Consistent page title, supporting text and trailing application actions. */
export function PageHeader(props: PageHeaderProps): JSX.Element {
  return (
    <View class={pageHeaderClass(props.class, props.stacked)}>
      <View class="min-w-0 flex-1 flex flex-row items-center gap-3">
        <View class="min-w-0 flex flex-col gap-1">
          <Text role="heading" class={pageHeaderTitleClass()}>
            {props.title}
          </Text>
          <Show when={props.description}>
            {(description) => (
              <Text class={pageHeaderDescriptionClass(props.stacked)}>
                {description()}
              </Text>
            )}
          </Show>
        </View>
        {props.titleAdornment}
      </View>
      <Show when={props.actions}>
        {(actions) => (
          <View
            class="flex-none flex flex-row items-center gap-2"
            classList={{ "w-full": props.stacked }}
          >
            {actions()}
          </View>
        )}
      </Show>
    </View>
  );
}
