import { createTransition, type Easing } from "@wabou/animation";
import { number, px } from "@wabou/style";
import { createEffect, type JSX, Show } from "solid-js";
import { createMeasuredSize } from "./measure";
import { createPresence } from "./presence";
import { View, type WabouStyle } from "./view";

export interface CollapsiblePresenceProps {
  open: boolean;
  children?: JSX.Element;
  class?: string;
  contentClass?: string;
  style?: WabouStyle;
  contentStyle?: WabouStyle;
  duration?: number;
  ease?: Easing;
  reducedMotion?: boolean;
  /** Animate an initially-open disclosure from zero height. Defaults to false. */
  animateInitial?: boolean;
}

/**
 * Measured disclosure content with explicit presence and subtree isolation.
 * Height participates in layout while a subtree opacity layer masks glyphs
 * crossing the moving clip edge.
 */
export function CollapsiblePresence(
  props: CollapsiblePresenceProps,
): JSX.Element {
  const open = () => props.open;
  const initiallyOpen = open();
  const presence = createPresence(open);
  let initialMeasurement = true;
  let heightTransition: ReturnType<typeof createTransition> | undefined;
  const opacityTransition = createTransition(() => (open() ? 1 : 0), {
    duration: props.duration ?? 0.2,
    ease: props.ease ?? "easeOut",
    reducedMotion: () => props.reducedMotion ?? false,
  });
  const measured = createMeasuredSize({
    onChange(size) {
      if (initialMeasurement && initiallyOpen && !props.animateInitial) {
        heightTransition?.jump(size.height);
      }
      initialMeasurement = false;
    },
  });
  const transitionOptions = () => ({
    duration: props.duration ?? 0.2,
    ease: props.ease ?? "easeOut",
    reducedMotion: () => props.reducedMotion ?? false,
  });
  heightTransition = createTransition(
    () => (open() && measured.measured() ? measured.height() : 0),
    {
      ...transitionOptions(),
      onComplete(value) {
        if (value === 0 && !open()) presence.finishExit();
        else if (open()) presence.finishEnter();
      },
    },
  );
  createEffect(
    () => [open(), measured.measured(), measured.height()] as const,
    ([isOpen, isMeasured, height]) => {
    if (isOpen && isMeasured && height === 0) {
      presence.finishEnter();
    }
    },
  );

  const style = (): WabouStyle => ({
    ...props.style,
    height: px(heightTransition?.value() ?? 0),
    opacity: number(opacityTransition.value()),
  });

  return (
    <View
      class={props.class}
      classList={{ "overflow-hidden": true }}
      style={style()}
      inert={open() ? undefined : ""}
      aria-hidden={open() ? undefined : "true"}
    >
      <Show when={presence.mounted()}>
        <View ref={measured.ref}>
          <View class={props.contentClass} style={props.contentStyle}>
            {props.children}
          </View>
        </View>
      </Show>
    </View>
  );
}
