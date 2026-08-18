import { Pulse, Spin, Text, View } from "@wabou/primitives";
import type { JSX } from "solid-js";
import { join } from "./class-names";

export function Skeleton(props: { class?: string }): JSX.Element {
  return (
    <Pulse
      aria-hidden="true"
      class={join("rounded-md bg-control", props.class)}
      from={0.45}
      to={0.85}
      duration={1.8}
    />
  );
}

export function Spinner(props: {
  label?: string;
  class?: string;
}): JSX.Element {
  return (
    <Spin
      role="status"
      aria-label={props.label ?? "Loading"}
      class={join("w-4 h-4 flex-none text-accent", props.class)}
      duration={0.9}
    >
      <svg
        aria-hidden="true"
        class="w-full h-full"
        viewBox="0 0 24 24"
        fill="none"
      >
        <circle
          cx="12"
          cy="12"
          r="9"
          stroke="currentColor"
          stroke-width="3"
          opacity="0.25"
        />
        <path
          d="M 12 3 A 9 9 0 0 1 21 12"
          stroke="currentColor"
          stroke-width="3"
          stroke-linecap="round"
        />
      </svg>
    </Spin>
  );
}

export function Kbd(props: {
  class?: string;
  children?: JSX.Element;
}): JSX.Element {
  return (
    <Text
      class={join(
        "h-5 min-w-5 px-1 py-0.5 flex-none text-center rounded bg-control text-xs font-medium text-muted",
        props.class,
      )}
    >
      {props.children}
    </Text>
  );
}

export function KbdGroup(props: {
  class?: string;
  children?: JSX.Element;
}): JSX.Element {
  return (
    <View class={join("inline-flex items-center gap-1", props.class)}>
      {props.children}
    </View>
  );
}
