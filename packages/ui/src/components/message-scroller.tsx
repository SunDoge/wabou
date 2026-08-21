import type { Handle, WabouScrollEvent } from "@wabou/core/renderer";
import arrowDown from "lucide-static/icons/arrow-down.svg?raw";
import {
  createContext,
  createSignal,
  type JSX,
  omit,
  onCleanup,
  Show,
  useContext,
} from "solid-js";
import {
  createMeasuredSize,
  Icon,
  rotate2d,
  translate2d,
  View,
  type ViewProps,
} from "../primitives";
import { Button, type ButtonProps } from "./button";
import { join } from "./class-names";

export type MessageScrollDirection = "start" | "end";

export function messageScrollRange(
  contentHeight: number,
  viewportHeight: number,
): number {
  return Math.max(0, contentHeight - viewportHeight);
}

export function isMessageScrollNearEnd(
  scrollY: number,
  contentHeight: number,
  viewportHeight: number,
  threshold = 24,
): boolean {
  return (
    scrollY >=
    messageScrollRange(contentHeight, viewportHeight) - Math.max(0, threshold)
  );
}

export interface MessageScrollerControls {
  followingEnd(): boolean;
  canScrollStart(): boolean;
  canScrollEnd(): boolean;
  scrollTo(direction: MessageScrollDirection): void;
}

interface MessageScrollerContextValue extends MessageScrollerControls {
  setViewport(node: Handle): void;
  setContent(node: Handle): void;
  handleScroll(event: WabouScrollEvent): void;
}

const MessageScrollerContext = createContext<MessageScrollerContextValue>();

function requireMessageScroller(): MessageScrollerContextValue {
  const context = useContext(MessageScrollerContext);
  if (!context) {
    throw new Error(
      "MessageScroller components require a MessageScroller root",
    );
  }
  return context;
}

export function useMessageScroller(): MessageScrollerControls {
  return requireMessageScroller();
}

export interface MessageScrollerProps extends ViewProps {
  /** Start attached to the end of the conversation. Defaults to true. */
  followEnd?: boolean;
  /** Distance in logical pixels that still counts as being at the end. */
  endThreshold?: number;
}

export function MessageScroller(props: MessageScrollerProps): JSX.Element {
  const forwarded = omit(
    props,
    "followEnd",
    "endThreshold",
    "class",
    "children",
  );
  const [scrollY, setScrollY] = createSignal(0);
  const [followingEnd, setFollowingEnd] = createSignal(props.followEnd ?? true);
  const threshold = () => Math.max(0, props.endThreshold ?? 24);
  let viewport: Handle | undefined;
  let frame: number | undefined;

  const scheduleEnd = () => {
    if (!followingEnd() || !viewport) return;
    if (frame !== undefined) cancelAnimationFrame(frame);
    frame = requestAnimationFrame(() => {
      frame = undefined;
      viewport?.scrollTo({ top: Number.MAX_SAFE_INTEGER });
    });
  };

  const viewportSize = createMeasuredSize({ onChange: scheduleEnd });
  const contentSize = createMeasuredSize({ onChange: scheduleEnd });
  const range = () =>
    messageScrollRange(contentSize.height(), viewportSize.height());
  const nearEnd = () =>
    isMessageScrollNearEnd(
      scrollY(),
      contentSize.height(),
      viewportSize.height(),
      threshold(),
    );

  const context: MessageScrollerContextValue = {
    followingEnd,
    canScrollStart: () => scrollY() > threshold(),
    canScrollEnd: () => range() > 0 && !nearEnd(),
    scrollTo: (direction) => {
      setFollowingEnd(direction === "end");
      viewport?.scrollTo({
        top: direction === "end" ? Number.MAX_SAFE_INTEGER : 0,
      });
    },
    setViewport: (node) => {
      viewport = node;
      viewportSize.ref(node);
      scheduleEnd();
    },
    setContent: (node) => contentSize.ref(node),
    handleScroll: (event) => {
      const next = Math.max(0, event.scrollY ?? 0);
      setScrollY(next);
      setFollowingEnd(
        isMessageScrollNearEnd(
          next,
          contentSize.height(),
          viewportSize.height(),
          threshold(),
        ),
      );
    },
  };

  onCleanup(() => {
    if (frame !== undefined) cancelAnimationFrame(frame);
  });

  return (
    <MessageScrollerContext value={context}>
      <View
        {...forwarded}
        class={join(
          "relative w-full h-full min-w-0 min-h-0 flex flex-col overflow-hidden",
          props.class,
        )}
        transform={props.transform ?? translate2d(-16, 0)}
      >
        {props.children}
      </View>
    </MessageScrollerContext>
  );
}

export interface MessageScrollerViewportProps extends ViewProps {}

export function MessageScrollerViewport(
  props: MessageScrollerViewportProps,
): JSX.Element {
  const context = requireMessageScroller();
  const forwarded = omit(props, "class", "children", "ref", "onScroll");
  return (
    <View
      {...forwarded}
      ref={(node) => {
        context.setViewport(node);
        props.ref?.(node);
      }}
      class={join(
        "w-full min-w-0 min-h-0 flex-1 overflow-x-hidden overflow-y-auto",
        props.class,
      )}
      scrollbar={props.scrollbar ?? { visibility: "auto" }}
      onScroll={(event) => {
        context.handleScroll(event);
        props.onScroll?.(event);
      }}
    >
      {props.children}
    </View>
  );
}

export function MessageScrollerContent(props: ViewProps): JSX.Element {
  const context = requireMessageScroller();
  const forwarded = omit(props, "class", "children", "ref");
  return (
    <View
      {...forwarded}
      ref={(node) => {
        context.setContent(node);
        props.ref?.(node);
      }}
      class={join(
        "w-full min-w-0 min-h-full flex-none flex flex-col gap-4",
        props.class,
      )}
    >
      {props.children}
    </View>
  );
}

export function MessageScrollerItem(props: ViewProps): JSX.Element {
  return (
    <View {...props} class={join("w-full min-w-0 flex-none", props.class)}>
      {props.children}
    </View>
  );
}

export interface MessageScrollerButtonProps
  extends Omit<ButtonProps, "onClick"> {
  direction?: MessageScrollDirection;
  onClick?: ButtonProps["onClick"];
}

export function MessageScrollerButton(
  props: MessageScrollerButtonProps,
): JSX.Element {
  const context = requireMessageScroller();
  const direction = () => props.direction ?? "end";
  const active = () =>
    direction() === "end" ? context.canScrollEnd() : context.canScrollStart();
  const forwarded = omit(props, "direction", "class", "children", "onClick");
  const label = () =>
    direction() === "end" ? "Scroll to end" : "Scroll to start";
  return (
    <Show when={active()}>
      <Button
        {...forwarded}
        aria-label={props["aria-label"] ?? label()}
        variant={props.variant ?? "secondary"}
        size={props.size ?? "icon"}
        class={join(
          "absolute z-10 left-1/2 flex-none rounded-full shadow-sm",
          direction() === "end" ? "bottom-3" : "top-3",
          props.class,
        )}
        onClick={(event) => {
          context.scrollTo(direction());
          props.onClick?.(event);
        }}
      >
        {props.children ?? (
          <Icon
            aria-hidden="true"
            source={arrowDown}
            size={16}
            transform={direction() === "start" ? rotate2d(Math.PI) : undefined}
          />
        )}
      </Button>
    </Show>
  );
}
