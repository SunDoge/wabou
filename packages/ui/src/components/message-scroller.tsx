import {
  type Handle,
  type LayoutRect,
  useHost,
  type WabouScrollEvent,
} from "@wabou/core/renderer";
import { mergeClasses } from "@wabou/core/style";
import arrowDown from "lucide-static/icons/arrow-down.svg?raw";
import {
  createContext,
  createEffect,
  createSignal,
  For as ForValue,
  type JSX,
  omit,
  onCleanup,
  Show,
  untrack,
  useContext,
} from "solid-js";
import {
  createMeasuredSize,
  Icon,
  rotate2d,
  Text,
  translate2d,
  View,
  type ViewProps,
} from "../primitives";
import { Button, type ButtonProps } from "./button";
import { Tooltip } from "./tooltip";

export type MessageScrollDirection = "start" | "end";

export interface MessageScrollIntoViewOptions {
  margin?: number;
  align?: "nearest" | "start";
}

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

/** Smallest vertical delta that reveals a target without disturbing visible content. */
export function messageScrollRevealDelta(
  viewport: LayoutRect,
  target: LayoutRect,
  margin = 12,
): number {
  const inset = Math.max(0, margin);
  const visibleTop = viewport.y + inset;
  const visibleBottom = viewport.y + viewport.height - inset;
  if (target.y < visibleTop) return target.y - visibleTop;
  const targetBottom = target.y + target.height;
  if (targetBottom > visibleBottom) return targetBottom - visibleBottom;
  return 0;
}

/** Vertical delta that places a target at the viewport's reading start. */
export function messageScrollStartDelta(
  viewport: LayoutRect,
  target: LayoutRect,
  margin = 12,
): number {
  return target.y - (viewport.y + Math.max(0, margin));
}

export interface MessageScrollerControls {
  followingEnd(): boolean;
  canScrollStart(): boolean;
  canScrollEnd(): boolean;
  activeAnchor(): string | undefined;
  scrollTo(direction: MessageScrollDirection): void;
  scrollIntoView(target: Handle, options?: MessageScrollIntoViewOptions): void;
  scrollToAnchor(anchor: string, options?: MessageScrollIntoViewOptions): void;
}

interface MessageScrollerContextValue extends MessageScrollerControls {
  setViewport(node: Handle): void;
  setContent(node: Handle): void;
  handleScroll(event: WabouScrollEvent): void;
  registerAnchor(anchor: string, node: Handle): void;
  unregisterAnchor(anchor: string, node: Handle): void;
}

const MessageScrollerContext =
  createContext<MessageScrollerContextValue | null>(null);

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

export interface MessageAnchorRect {
  id: string;
  rect: LayoutRect;
}

/** Pick the last conversation anchor that has crossed the reading line. */
export function activeMessageAnchor(
  viewport: LayoutRect,
  anchors: readonly MessageAnchorRect[],
  offset = 64,
): string | undefined {
  if (anchors.length === 0) return undefined;
  const line = viewport.y + Math.min(Math.max(0, offset), viewport.height / 3);
  const ordered = [...anchors].sort((first, second) =>
    first.rect.y === second.rect.y
      ? first.rect.x - second.rect.x
      : first.rect.y - second.rect.y,
  );
  let active = ordered[0]?.id;
  for (const anchor of ordered) {
    if (anchor.rect.y > line) break;
    active = anchor.id;
  }
  return active;
}

export function MessageScroller(props: MessageScrollerProps): JSX.Element {
  const host = useHost();
  const forwarded = omit(
    props,
    "followEnd",
    "endThreshold",
    "class",
    "children",
  );
  const [scrollY, setScrollY] = createSignal(0);
  const [followingEnd, setFollowingEnd] = createSignal(
    untrack(() => props.followEnd ?? true),
  );
  const [activeAnchor, setActiveAnchor] = createSignal<string>();
  const threshold = () => Math.max(0, props.endThreshold ?? 24);
  let viewport: Handle | undefined;
  let endFrame: number | undefined;
  let anchorFrame: number | undefined;
  const anchors = new Map<string, Handle>();
  let measuredAnchors: MessageAnchorRect[] = [];

  const updateActiveFromScroll = (position = scrollY()) => {
    if (
      followingEnd() &&
      isMessageScrollNearEnd(
        position,
        contentSize.height(),
        viewportSize.height(),
        threshold(),
      )
    ) {
      const last = measuredAnchors.reduce<MessageAnchorRect | undefined>(
        (current, anchor) =>
          !current || anchor.rect.y > current.rect.y ? anchor : current,
        undefined,
      );
      setActiveAnchor(last?.id);
      return;
    }
    setActiveAnchor(
      activeMessageAnchor(
        {
          x: 0,
          y: position,
          width: viewportSize.width(),
          height: viewportSize.height(),
        },
        measuredAnchors,
      ),
    );
  };

  const measureAnchors = () => {
    if (!viewport || anchors.size === 0) {
      measuredAnchors = [];
      setActiveAnchor(undefined);
      return;
    }
    const targets = [...anchors.entries()];
    const snapshot = host.layout.snapshot([
      viewport,
      ...targets.map(([, node]) => node),
    ]);
    const nodeByKey = new Map(
      snapshot.nodes.map((node) => [`${node.id.lo}:${node.id.hi}`, node]),
    );
    const viewportMetrics = nodeByKey.get(
      `${viewport.id.lo}:${viewport.id.hi}`,
    );
    if (!viewportMetrics) return;
    measuredAnchors = targets.flatMap(([id, node]) => {
      const metrics = nodeByKey.get(`${node.id.lo}:${node.id.hi}`);
      return metrics
        ? [
            {
              id,
              rect: {
                x: metrics.rect.x - viewportMetrics.rect.x,
                y: metrics.rect.y - viewportMetrics.rect.y + scrollY(),
                width: metrics.rect.width,
                height: metrics.rect.height,
              },
            },
          ]
        : [];
    });
    updateActiveFromScroll();
  };

  const scheduleAnchorMeasure = () => {
    if (anchorFrame !== undefined) cancelAnimationFrame(anchorFrame);
    anchorFrame = requestAnimationFrame(() => {
      anchorFrame = undefined;
      measureAnchors();
    });
  };

  const scheduleEnd = (force = false) => {
    if ((!force && !followingEnd()) || !viewport) return;
    if (endFrame !== undefined) cancelAnimationFrame(endFrame);
    endFrame = requestAnimationFrame(() => {
      endFrame = undefined;
      viewport?.scrollTo({ top: Number.MAX_SAFE_INTEGER });
    });
  };

  const geometryChanged = () => {
    scheduleEnd();
    scheduleAnchorMeasure();
  };
  createEffect(
    () => props.followEnd,
    (next, previous) => {
      if (next === undefined || next === previous) return;
      setFollowingEnd(next);
      if (next) {
        scheduleEnd(true);
      } else if (endFrame !== undefined) {
        cancelAnimationFrame(endFrame);
        endFrame = undefined;
      }
    },
  );
  const viewportSize = createMeasuredSize({ onChange: geometryChanged });
  const contentSize = createMeasuredSize({ onChange: geometryChanged });
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
    activeAnchor,
    scrollTo: (direction) => {
      setFollowingEnd(direction === "end");
      viewport?.scrollTo({
        top: direction === "end" ? Number.MAX_SAFE_INTEGER : 0,
      });
    },
    scrollIntoView: (target, options) => {
      if (!viewport) return;
      const snapshot = host.layout.snapshot([viewport, target]);
      const viewportMetrics = snapshot.nodes.find(
        (node) =>
          node.id.lo === viewport?.id.lo && node.id.hi === viewport?.id.hi,
      );
      const targetMetrics = snapshot.nodes.find(
        (node) => node.id.lo === target.id.lo && node.id.hi === target.id.hi,
      );
      if (!viewportMetrics || !targetMetrics) return;
      const delta =
        options?.align === "start"
          ? messageScrollStartDelta(
              viewportMetrics.rect,
              targetMetrics.rect,
              options.margin,
            )
          : messageScrollRevealDelta(
              viewportMetrics.rect,
              targetMetrics.rect,
              options?.margin,
            );
      if (delta === 0) return;
      setFollowingEnd(false);
      viewport.scrollBy({ top: delta });
    },
    scrollToAnchor: (anchor, options) => {
      const target = anchors.get(anchor);
      if (target) context.scrollIntoView(target, options);
    },
    setViewport: (node) => {
      viewport = node;
      viewportSize.ref(node);
      scheduleEnd();
      scheduleAnchorMeasure();
    },
    setContent: (node) => {
      contentSize.ref(node);
      scheduleAnchorMeasure();
    },
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
      updateActiveFromScroll(next);
    },
    registerAnchor: (anchor, node) => {
      anchors.set(anchor, node);
      scheduleAnchorMeasure();
    },
    unregisterAnchor: (anchor, node) => {
      const current = anchors.get(anchor);
      if (current?.id.lo === node.id.lo && current.id.hi === node.id.hi) {
        anchors.delete(anchor);
        scheduleAnchorMeasure();
      }
    },
  };

  onCleanup(() => {
    if (endFrame !== undefined) cancelAnimationFrame(endFrame);
    if (anchorFrame !== undefined) cancelAnimationFrame(anchorFrame);
  });

  return (
    <MessageScrollerContext value={context}>
      <View
        {...forwarded}
        class={mergeClasses(
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
      class={mergeClasses(
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
      class={mergeClasses(
        "w-full min-w-0 min-h-full flex-none flex flex-col gap-4",
        props.class,
      )}
    >
      {props.children}
    </View>
  );
}

export interface MessageScrollerItemProps extends ViewProps {
  /** Stable semantic id used by conversation navigation. */
  anchor?: string;
}

export function MessageScrollerItem(
  props: MessageScrollerItemProps,
): JSX.Element {
  const context = useContext(MessageScrollerContext);
  const forwarded = omit(props, "anchor", "class", "children", "ref");
  let node: Handle | undefined;
  let registered: string | undefined;

  createEffect(
    () => props.anchor,
    (next) => {
      if (node && registered && registered !== next) {
        context?.unregisterAnchor(registered, node);
        registered = undefined;
      }
      if (node && next && next !== registered) {
        context?.registerAnchor(next, node);
        registered = next;
      }
    },
  );

  onCleanup(() => {
    if (node && registered) context?.unregisterAnchor(registered, node);
  });

  return (
    <View
      {...forwarded}
      ref={(handle) => {
        node = handle;
        const anchor = props.anchor;
        if (anchor) {
          context?.registerAnchor(anchor, handle);
          registered = anchor;
        }
        props.ref?.(handle);
      }}
      data-message-anchor={props.anchor}
      class={mergeClasses("w-full min-w-0 flex-none", props.class)}
    >
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
        class={mergeClasses(
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

export interface MessageScrollerNavigatorItem {
  id: string;
  label: string;
}

export interface MessageScrollerNavigatorProps {
  items: readonly MessageScrollerNavigatorItem[];
  "aria-label": string;
  itemAriaLabel(item: MessageScrollerNavigatorItem, index: number): string;
  minItems?: number;
  class?: string;
  railClass?: string;
}

/** Compact anchor rail for navigating long retained conversations. */
export function MessageScrollerNavigator(
  props: MessageScrollerNavigatorProps,
): JSX.Element {
  const context = requireMessageScroller();
  const reveal = (id: string) => {
    context.scrollToAnchor(id, { margin: 24, align: "start" });
  };

  return (
    <Show when={props.items.length >= (props.minItems ?? 2)}>
      <View
        role="group"
        aria-label={props["aria-label"]}
        class={mergeClasses(
          "absolute z-20 right-2 top-4 bottom-14 w-8 flex flex-col items-center justify-center pointer-events-none",
          props.class,
        )}
      >
        <View
          class={mergeClasses(
            "max-h-full py-1 rounded-full border border-subtle bg-surface shadow-xs overflow-y-auto pointer-events-auto",
            props.railClass,
          )}
        >
          <ForValue each={props.items} keyed={false}>
            {(item, index) => (
              <Tooltip
                placement="left"
                openDelay={240}
                contentClass="max-w-sm"
                trigger={(tooltip) => (
                  <Button
                    ref={tooltip.ref}
                    variant="ghost"
                    size="icon"
                    class="w-7 h-7 p-0 rounded-full"
                    aria-label={props.itemAriaLabel(item(), index)}
                    aria-current={
                      context.activeAnchor() === item().id ? "step" : undefined
                    }
                    onPointerEnter={tooltip.onPointerEnter}
                    onPointerLeave={tooltip.onPointerLeave}
                    onFocus={tooltip.onFocus}
                    onBlur={tooltip.onBlur}
                    onKeyDown={tooltip.onKeyDown}
                    onClick={() => reveal(item().id)}
                  >
                    <View
                      aria-hidden="true"
                      class={
                        context.activeAnchor() === item().id
                          ? "w-4 h-1 rounded-full bg-accent"
                          : "w-3 h-1 rounded-full bg-subtle"
                      }
                    />
                  </Button>
                )}
              >
                <Text class="text-xs text-primary whitespace-normal">
                  {item().label}
                </Text>
              </Tooltip>
            )}
          </ForValue>
        </View>
      </View>
    </Show>
  );
}
