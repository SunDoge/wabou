import { animate, animateKeyframes } from "@wabou/animation";
import { Center, Text, View } from "@wabou/primitives";
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
    <Center
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
  const transform = () => {
    const cosine = Math.cos(angle());
    const sine = Math.sin(angle());
    const center = 8;
    return [
      cosine,
      sine,
      -sine,
      cosine,
      center - cosine * center + sine * center,
      center - sine * center - cosine * center,
    ] as const;
  };
  return (
    <Center
      role="status"
      aria-label={props.label ?? "Loading"}
      class={join("w-4 h-4 flex-none text-accent", props.class)}
    >
      <Text class="w-4 h-4 text-sm font-bold" transform={transform()}>
        ◒
      </Text>
    </Center>
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
