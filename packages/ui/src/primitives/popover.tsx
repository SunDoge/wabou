import {
  type Handle,
  observeGlobalPointerEvent,
  Portal,
  useHost,
} from "@wabou/core/renderer";
import { number, type Shadow, scale2d } from "@wabou/core/style";
import {
  createEffect,
  createSignal,
  type JSX,
  onCleanup,
  Show,
  untrack,
} from "solid-js";
import { type Easing, useReducedMotion } from "../animation";
import {
  createOverlayLayer,
  type OverlayDismissReason,
  type OverlayPlane,
  useOverlayPlane,
} from "./overlay-layer";
import {
  computeHostFloatingPosition,
  computeHostPointFloatingPosition,
  flip,
  LayoutTargetUnavailableError,
  offset,
  type Placement,
  type PointAnchor,
  shift,
} from "./positioner";
import { createTransitionPresence } from "./transition-presence";
import { View, type ViewProps, type WabouStyle } from "./view";

export interface PopoverTriggerProps {
  ref: (node: Handle) => void;
  onPointerDown: (event: { button?: number; stopPropagation(): void }) => void;
  onPointerCancel: () => void;
  onClick: (event: { stopPropagation(): void }) => void;
  onKeyDown: (event: {
    key: string;
    preventDefault(): void;
    stopPropagation(): void;
  }) => void;
  "aria-haspopup"?: "dialog" | "listbox" | "menu" | "tree" | "grid";
  "aria-expanded": boolean;
}

interface PopoverBaseProps {
  trigger: (props: PopoverTriggerProps) => JSX.Element;
  children?: JSX.Element;
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?: (
    open: boolean,
    reason?: OverlayDismissReason | "trigger",
  ) => void;
  placement?: Placement;
  offset?: number;
  /** Positions from a viewport point instead of the trigger's layout box. */
  anchorPoint?: () => PointAnchor | undefined;
  contentClass?: string;
  contentStyle?: WabouStyle;
  contentShadows?: readonly Shadow[] | null;
  /** Removes the positioned content subtree from native hit testing. */
  contentInteractionBlocked?: boolean;
  /** Keeps composed hover/focus surfaces open while the pointer is inside. */
  onContentPointerEnter?: ViewProps["onPointerEnter"];
  onContentPointerLeave?: ViewProps["onPointerLeave"];
  onContentFocusIn?: ViewProps["onFocusIn"];
  onContentFocusOut?: ViewProps["onFocusOut"];
  closeOnEscape?: boolean;
  restoreFocus?: boolean;
  /**
   * `passthrough` dismisses from global pointer capture while allowing the
   * underlying target to receive the same gesture. Modal-like surfaces keep
   * the default full-viewport backdrop.
   */
  outsidePointerStrategy?: "backdrop" | "passthrough";
  /** Defaults to the nearest overlay plane, or `floating` at app content. */
  plane?: OverlayPlane;
  /** Set to false to keep presence semantics while disabling visual motion. */
  motion?: false | PopoverMotionOptions;
  /** Open on primary pointer-down; useful for native-feeling selects and menus. */
  openOnPointerDown?: boolean;
}

export interface PopoverMotionOptions {
  duration?: number;
  ease?: Easing;
  /** Initial scale around the panel center. Defaults to 0.98. */
  fromScale?: number;
}

export type PopoverProps = PopoverBaseProps &
  (
    | {
        /** Required until the native semantic tree resolves aria-labelledby. */
        "aria-label": string;
        contentRole?: "dialog";
        popupRole?: never;
      }
    | {
        /** Flatten the positioned shell when its child owns popup semantics. */
        contentRole: "presentation";
        popupRole: "listbox" | "menu" | "tree" | "grid" | "tooltip";
        "aria-label"?: never;
      }
  );

/** A root-layer floating panel positioned from native layout snapshots. */
export function Popover(props: PopoverProps): JSX.Element {
  const host = useHost();
  const inheritedPlane = useOverlayPlane();
  const reducedMotion = useReducedMotion();
  const plane = () => props.plane ?? inheritedPlane;
  const [uncontrolledOpen, setUncontrolledOpen] = createSignal(
    untrack(() => props.defaultOpen ?? false),
  );
  const [position, setPosition] = createSignal({ x: 0, y: 0 });
  const [positioned, setPositioned] = createSignal(false);
  const open = () => props.open ?? uncontrolledOpen();
  const motion = untrack(() => props.motion);
  const presence = createTransitionPresence(open, {
    ready: positioned,
    duration: motion === false ? 0 : (motion?.duration ?? 0.14),
    ease: motion === false ? "linear" : (motion?.ease ?? "easeOut"),
    reducedMotion: () => motion === false || reducedMotion(),
  });
  let anchor: Handle | undefined;
  let content: Handle | undefined;
  let frame = 0;
  let positionRequest = 0;
  let suppressPointerClick = false;
  let observer: ResizeObserver | undefined;
  const motionFromScale = () =>
    motion === false ? 1 : (motion?.fromScale ?? 0.98);

  const contains = (root: Handle | undefined, target: Handle | undefined) => {
    if (!root || !target) return false;
    let current: Handle | null = target;
    while (current) {
      if (current === root) return true;
      current = current.parent;
    }
    return false;
  };

  const setOpen = (
    next: boolean,
    reason?: OverlayDismissReason | "trigger",
  ) => {
    if (props.open === undefined) setUncontrolledOpen(next);
    props.onOpenChange?.(next, reason);
  };
  const layer = createOverlayLayer({
    open,
    plane,
    onDismiss: (reason) => setOpen(false, reason),
    closeOnEscape: () => props.closeOnEscape ?? true,
    returnFocus: () => anchor,
    restoreFocus: () => props.restoreFocus ?? true,
  });
  const stopObservingPointer = observeGlobalPointerEvent("click", (target) => {
    if (
      props.outsidePointerStrategy !== "passthrough" ||
      !open() ||
      contains(anchor, target) ||
      contains(content, target)
    )
      return;
    layer.onOutside({ preventDefault() {}, stopPropagation() {} });
  });

  const updatePosition = async () => {
    const point = props.anchorPoint?.();
    const reference = anchor;
    if (!open() || (!point && !reference) || !content) return;
    const request = ++positionRequest;
    try {
      const options = {
        placement: props.placement ?? "bottom-start",
        middleware: [offset(props.offset ?? 6), flip(), shift({ padding: 8 })],
      };
      let result: Awaited<ReturnType<typeof computeHostFloatingPosition>>;
      if (point) {
        result = await computeHostPointFloatingPosition(
          point,
          content,
          host,
          options,
        );
      } else {
        if (!reference) return;
        result = await computeHostFloatingPosition(
          reference,
          content,
          host,
          options,
        );
      }
      if (!open() || request !== positionRequest) return;
      setPosition({ x: result.x, y: result.y });
      setPositioned(true);
    } catch (error) {
      // Portal mutations and native layout complete on separate frames. A
      // missing target is not a usable zero-sized rect; keep the panel outside
      // the viewport and retry after the host has published the next snapshot.
      if (
        error instanceof LayoutTargetUnavailableError &&
        open() &&
        request === positionRequest
      ) {
        schedulePosition();
        return;
      }
      throw error;
    }
  };

  const schedulePosition = () => {
    cancelAnimationFrame(frame);
    frame = requestAnimationFrame(() => void updatePosition());
  };

  const observe = (node: Handle) => {
    observer?.observe(node as never);
    schedulePosition();
  };

  createEffect(open, (isOpen) => {
    if (!isOpen) {
      positionRequest++;
      observer?.disconnect();
      observer = undefined;
      return;
    }
    observer = new ResizeObserver(schedulePosition);
    if (anchor) observer.observe(anchor as never);
    // Portal mutations flush with the opening event, so the next native frame
    // can normally position the panel immediately. If its completed layout is
    // not published yet, updatePosition schedules the existing bounded retry.
    frame = requestAnimationFrame(() => void updatePosition());
  });
  createEffect(presence.phase, (phase) => {
    if (phase === "unmounted") setPositioned(false);
  });

  onCleanup(() => {
    stopObservingPointer();
    cancelAnimationFrame(frame);
    observer?.disconnect();
  });

  const handleEscape = (event: {
    key: string;
    preventDefault(): void;
    stopPropagation(): void;
  }) => layer.onEscape(event);

  const popup = ():
    | "dialog"
    | "listbox"
    | "menu"
    | "tree"
    | "grid"
    | undefined => {
    if (props.contentRole !== "presentation") return "dialog";
    return props.popupRole === "tooltip" ? undefined : props.popupRole;
  };
  const triggerProps: PopoverTriggerProps = {
    ref: (node) => {
      anchor = node;
      if (open()) observe(node);
    },
    onPointerDown: (event) => {
      if (
        !props.openOnPointerDown ||
        open() ||
        (event.button !== undefined && event.button !== 0)
      )
        return;
      event.stopPropagation();
      suppressPointerClick = true;
      setOpen(true, "trigger");
    },
    onPointerCancel: () => {
      suppressPointerClick = false;
    },
    onClick: (event) => {
      event.stopPropagation();
      if (suppressPointerClick) {
        suppressPointerClick = false;
        return;
      }
      setOpen(!open(), "trigger");
    },
    onKeyDown: handleEscape,
    get "aria-haspopup"() {
      return popup();
    },
    get "aria-expanded"() {
      return open();
    },
  };
  // Render functions create component ownership. Invoke the trigger once and
  // expose reactive attributes through getters instead of recreating its
  // subtree whenever the overlay opens or closes.
  const trigger = untrack(() => props.trigger(triggerProps));

  return (
    <>
      {trigger}
      <Show when={presence.mounted()}>
        <Portal
          plane={plane()}
          ref={(node: Handle) => {
            observe(node);
          }}
          role="presentation"
          style={{
            position: "absolute",
            left: 0,
            top: 0,
            width: "100%",
            height: "100%",
            "z-index": layer.zIndex(),
            "pointer-events":
              !open() || props.outsidePointerStrategy === "passthrough"
                ? "none"
                : "auto",
          }}
          onClick={
            props.outsidePointerStrategy === "passthrough"
              ? undefined
              : layer.onOutside
          }
          onKeyDown={handleEscape}
          onWheel={schedulePosition}
        >
          <View
            ref={(node) => {
              content = node;
              observe(node);
            }}
            role={props.contentRole ?? "dialog"}
            aria-label={props["aria-label"]}
            class={props.contentClass}
            shadows={props.contentShadows}
            transform={scale2d(
              motionFromScale() + presence.progress() * (1 - motionFromScale()),
            )}
            interactionBlocked={!open() || props.contentInteractionBlocked}
            aria-hidden={open() ? undefined : "true"}
            style={{
              position: "absolute",
              // The panel must participate in layout before Floating UI can
              // measure it. Keep that measurement pass outside the viewport:
              // Keep the measurement pass outside the viewport so it is also
              // absent from hit testing while the position is unresolved.
              left: positioned() ? `${position().x}px` : "-100000px",
              top: positioned() ? `${position().y}px` : "-100000px",
              ...props.contentStyle,
              opacity: number(presence.progress()),
            }}
            onClick={(event: { stopPropagation(): void }) =>
              event.stopPropagation()
            }
            onPointerEnter={props.onContentPointerEnter}
            onPointerLeave={props.onContentPointerLeave}
            onFocusIn={props.onContentFocusIn}
            onFocusOut={props.onContentFocusOut}
            onKeyDown={handleEscape}
          >
            {props.children}
          </View>
        </Portal>
      </Show>
    </>
  );
}
