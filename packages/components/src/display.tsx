import { animate, animateKeyframes } from "@wabou/animation";
import { rotate2d, Text, View } from "@wabou/primitives";
import { createSignal, type JSX, onCleanup } from "solid-js";

const join = (...values: Array<string | undefined | false>) =>
  values.filter(Boolean).join(" ");

export function Skeleton(props: { class?: string }): JSX.Element {
  const [opacity, setOpacity] = createSignal(0.55);
  const animation = animateKeyframes([0.45, 0.85, 0.45], {
    duration: 1.8,
    ease: "easeInOut",
    repeat: Infinity,
    onUpdate: setOpacity,
  });
  onCleanup(() => animation.stop());
  return (
    <View
      aria-hidden="true"
      class={join("rounded-md bg-control", props.class)}
      style={{ opacity: opacity() }}
    />
  );
}

export function Spinner(props: {
  label?: string;
  class?: string;
}): JSX.Element {
  const [angle, setAngle] = createSignal(0);
  const animation = animate(0, Math.PI * 2, {
    duration: 0.9,
    ease: "linear",
    repeat: Infinity,
    onUpdate: setAngle,
  });
  onCleanup(() => animation.stop());
  return (
    <View
      role="status"
      aria-label={props.label ?? "Loading"}
      class={join("w-4 h-4 flex-none text-accent", props.class)}
      transform={rotate2d(angle())}
    >
      <svg class="w-full h-full" viewBox="0 0 24 24" fill="none">
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
    </View>
  );
}

export function Kbd(props: {
  class?: string;
  children?: JSX.Element;
}): JSX.Element {
  return (
    <Text
      class={join(
        "h-5 min-w-5 px-1 flex-none text-center rounded bg-control text-xs font-medium text-muted",
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
