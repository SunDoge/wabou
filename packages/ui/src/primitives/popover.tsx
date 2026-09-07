import {
  type Handle,
  observeGlobalPointerEvent,
  Portal,
  type WabouFloatingPosition,
} from "@wabou/core/renderer";
import { type Shadow, scale2d } from "@wabou/core/style";
import { createSignal, type JSX, onCleanup, Show, untrack } from "solid-js";
import { type Easing, useReducedMotion } from "../animation";
import {
  createOverlayLayer,
  type OverlayDismissReason,
  type OverlayPlane,
  useOverlayPlane,
} from "./overlay-layer";
import {
  floatingFromNode,
  floatingFromPoint,
  type Placement,
  type PointAnchor,
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
  const inheritedPlane = useOverlayPlane();
  const reducedMotion = useReducedMotion();
  const plane = () => props.plane ?? inheritedPlane;
  const [uncontrolledOpen, setUncontrolledOpen] = createSignal(
    untrack(() => props.defaultOpen ?? false),
  );
  const open = () => props.open ?? uncontrolledOpen();
  const motion = untrack(() => props.motion);
  const motionEnabled = motion !== undefined && motion !== false;
  const presence = createTransitionPresence(open, {
    initialProgress: untrack(open) ? 1 : 0,
    duration: motionEnabled ? (motion.duration ?? 0.14) : 0,
    ease: motionEnabled ? (motion.ease ?? "easeOut") : "linear",
    reducedMotion,
  });
  let anchor: Handle | undefined;
  let content: Handle | undefined;
  let suppressPointerClick = false;
  const motionFromScale = () =>
    motionEnabled ? (motion.fromScale ?? 0.98) : 1;

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
  const floatingPosition = (): WabouFloatingPosition | undefined => {
    const options = {
      placement: props.placement ?? "bottom-start",
      offset: props.offset ?? 6,
      margin: 8,
    } as const;
    const point = props.anchorPoint?.();
    if (point) return floatingFromPoint(point, options);
    return anchor ? floatingFromNode(anchor, options) : undefined;
  };

  onCleanup(() => {
    stopObservingPointer();
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
        >
          <View
            ref={(node) => {
              content = node;
            }}
            role={props.contentRole ?? "dialog"}
            aria-label={props["aria-label"]}
            class={props.contentClass}
            shadows={props.contentShadows}
            transform={scale2d(
              motionFromScale() + (1 - motionFromScale()) * presence.progress(),
            )}
            floatingPosition={floatingPosition()}
            interactionBlocked={!open() || props.contentInteractionBlocked}
            aria-hidden={open() ? undefined : "true"}
            style={{
              position: "absolute",
              ...props.contentStyle,
              opacity: presence.progress(),
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
