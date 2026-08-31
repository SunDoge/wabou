import { createElement, spread } from "@wabou/core/renderer";
import { createSignal, type JSX } from "solid-js";
import {
  createNativeLoopAnimation,
  createSweep,
  useReducedMotion,
} from "../animation";
import { createMeasuredSize, Text, View } from "../primitives";
import { mergeClasses } from "@wabou/core/style";

export interface SkeletonProps {
  class?: string;
  /** Disable the shimmer while preserving the stable loading placeholder. */
  animated?: boolean;
}

export function Skeleton(props: SkeletonProps): JSX.Element {
  const reducedMotion = useReducedMotion();
  const motionDisabled = () => props.animated === false || reducedMotion();
  const [width, setWidth] = createSignal(0, { ownedWrite: true });
  const measured = createMeasuredSize({
    onChange: (size) => setWidth(size.width),
  });
  const sweep = createSweep({
    extent: width,
    itemRatio: 0.4,
    duration: 1.6,
    ease: "easeInOut",
    reducedMotion: motionDisabled,
    reducedValue: 0.5,
  });
  return (
    <View
      ref={measured.ref}
      aria-hidden="true"
      class={mergeClasses("overflow-hidden rounded-md bg-control", props.class)}
    >
      <View
        class="w-2/5 h-full flex-none bg-control-hover"
        transform={sweep.transform()}
        style={{ opacity: motionDisabled() ? 0 : 1 }}
      />
    </View>
  );
}

export function Spinner(props: {
  label?: string;
  class?: string;
  /** Hide spinner semantics when a parent status already announces progress. */
  decorative?: boolean;
  /** Duration of one revolution in seconds. */
  duration?: number;
  /** Playback-rate multiplier. */
  speed?: number;
  paused?: boolean;
}): JSX.Element {
  const reducedMotion = useReducedMotion();
  const animation = createNativeLoopAnimation({
    duration: () => props.duration ?? 0.9,
    speed: () => props.speed ?? 1,
    paused: () => props.paused ?? false,
    reducedMotion,
  });
  const node = createElement("spinner");
  spread(
    node,
    {
      get role() {
        return props.decorative ? undefined : "status";
      },
      get "aria-hidden"() {
        return props.decorative ? true : undefined;
      },
      get "aria-label"() {
        return props.decorative ? undefined : (props.label ?? "Loading");
      },
      get class() {
        return mergeClasses("w-4 h-4 flex-none text-accent", props.class);
      },
      get widgetConfig() {
        return { animation: animation() };
      },
    },
    false,
  );
  return node as unknown as JSX.Element;
}

export function Kbd(props: {
  class?: string;
  children?: JSX.Element;
}): JSX.Element {
  return (
    <Text
      class={mergeClasses(
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
    <View class={mergeClasses("inline-flex items-center gap-1", props.class)}>
      {props.children}
    </View>
  );
}
