import { mergeClasses } from "@wabou/core/style";
import { type JSX, omit, Show } from "solid-js";
import { Pulse, Text, View, type ViewProps } from "../primitives";

export type ActivityStatusTone = "accent" | "danger" | "muted" | "success";

const toneClass: Record<ActivityStatusTone, string> = {
  accent: "bg-accent",
  danger: "bg-danger-primary",
  muted: "bg-muted",
  success: "bg-success-primary",
};

export interface ActivityStatusIndicatorProps {
  animated?: boolean;
  tone?: ActivityStatusTone;
  class?: string;
}

/** A consistently sized status dot. Animation is explicit so idle state never
 * keeps the native frame clock alive accidentally. */
export function ActivityStatusIndicator(
  props: ActivityStatusIndicatorProps,
): JSX.Element {
  const className = () =>
    mergeClasses(
      "w-1.5 h-1.5 flex-none rounded-full",
      toneClass[props.tone ?? "accent"],
      props.class,
    );
  return (
    <Show
      when={props.animated}
      fallback={<View aria-hidden="true" class={className()} />}
    >
      <Pulse
        aria-hidden="true"
        class={className()}
        from={0.3}
        to={1}
        duration={0.8}
      />
    </Show>
  );
}

export interface ActivityStatusProps
  extends Omit<ViewProps, "children" | "class"> {
  label: string;
  animated?: boolean;
  tone?: ActivityStatusTone;
  class?: string;
  textClass?: string;
}

/** Shrink-safe inline progress or presence status for agent and desktop UI. */
export function ActivityStatus(props: ActivityStatusProps): JSX.Element {
  const forwarded = omit(
    props,
    "label",
    "animated",
    "tone",
    "class",
    "textClass",
  );
  return (
    <View
      {...forwarded}
      role={props.role ?? "status"}
      aria-label={props["aria-label"] ?? props.label}
      class={mergeClasses(
        "min-w-0 max-w-full flex flex-row items-center gap-1.5",
        props.class,
      )}
    >
      <ActivityStatusIndicator animated={props.animated} tone={props.tone} />
      <Text
        class={mergeClasses(
          "min-w-0 truncate text-xs text-muted",
          props.textClass,
        )}
      >
        {props.label}
      </Text>
    </View>
  );
}
