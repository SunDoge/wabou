import { number, px } from "@wabou/core/style";
import { createEffect, type JSX, Show, untrack } from "solid-js";
import { createTransition, type Easing, useReducedMotion } from "../animation";
import { createMeasuredSize } from "./measure";
import { createPresence } from "./presence";
import { View, type ViewProps, type WabouStyle } from "./view";

export interface CollapsiblePresenceProps {
  open: boolean;
  children?: JSX.Element;
  class?: string;
  contentClass?: string;
  /** Props applied to the retained content node inside the animated viewport. */
  contentProps?: Omit<ViewProps, "children" | "class" | "style">;
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
  const inheritedReducedMotion = useReducedMotion();
  const reducedMotion = () => props.reducedMotion ?? inheritedReducedMotion();
  const open = () => props.open;
  const initiallyOpen = untrack(open);
  const presence = createPresence(open);
  let initialMeasurement = true;
  let heightTransition: ReturnType<typeof createTransition> | undefined;
  const opacityTransition = createTransition(() => (open() ? 1 : 0), {
    duration: props.duration ?? 0.2,
    ease: props.ease ?? "easeOut",
    reducedMotion,
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
    reducedMotion,
  });
  heightTransition = createTransition(
    () => (open() && measured.measured() ? measured.height() : 0),
    {
      ...transitionOptions(),
      onComplete(value) {
        const isOpen = untrack(open);
        if (value === 0 && !isOpen) presence.finishExit();
        else if (isOpen) presence.finishEnter();
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
      interactionBlocked={!open()}
      aria-hidden={open() ? undefined : "true"}
    >
      <Show when={presence.mounted()}>
        <View ref={measured.ref}>
          <View
            {...props.contentProps}
            class={props.contentClass}
            style={props.contentStyle}
          >
            {props.children}
          </View>
        </View>
      </Show>
    </View>
  );
}
