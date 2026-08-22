import { createSignal, type JSX } from "solid-js";
import { createSweep, useReducedMotion } from "../animation";
import { createMeasuredSize, Spin, Svg, Text, View } from "../primitives";
import { join } from "./class-names";

export interface SkeletonProps {
  class?: string;
  /** Disable the shimmer while preserving the stable loading placeholder. */
  animated?: boolean;
}

const SPINNER_SOURCE = `<svg viewBox="0 0 24 24" fill="none"><circle cx="12" cy="12" r="9" stroke="currentColor" stroke-width="3" opacity="0.25"/><path d="M 12 3 A 9 9 0 0 1 21 12" stroke="currentColor" stroke-width="3" stroke-linecap="round"/></svg>`;

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
      class={join("overflow-hidden rounded-md bg-control", props.class)}
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
}): JSX.Element {
  return (
    <Spin
      role="status"
      aria-label={props.label ?? "Loading"}
      class={join("w-4 h-4 flex-none text-accent", props.class)}
      duration={0.9}
    >
      <Svg aria-hidden="true" class="w-full h-full" source={SPINNER_SOURCE} />
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
