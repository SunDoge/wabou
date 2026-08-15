import { type Handle, Portal, useHost } from "@wabou/solid-renderer";
import {
  createEffect,
  createSignal,
  type JSX,
  onCleanup,
  Show,
} from "solid-js";
import {
  computeHostFloatingPosition,
  flip,
  LayoutTargetUnavailableError,
  offset,
  type Placement,
  shift,
} from "./positioner";
import { View, type WabouStyle } from "./view";
import { createOverlayLayer, type OverlayDismissReason } from "./overlay-layer";

export interface PopoverTriggerProps {
  ref: (node: Handle) => void;
  onClick: (event: { stopPropagation(): void }) => void;
  onKeyDown: (event: {
    key: string;
    preventDefault(): void;
    stopPropagation(): void;
  }) => void;
  "aria-haspopup": "dialog";
  "aria-expanded": boolean;
}

export interface PopoverProps {
  trigger: (props: PopoverTriggerProps) => JSX.Element;
  children?: JSX.Element;
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?: (open: boolean, reason?: OverlayDismissReason | "trigger") => void;
  placement?: Placement;
  offset?: number;
  contentClass?: string;
  contentStyle?: WabouStyle;
  closeOnEscape?: boolean;
  restoreFocus?: boolean;
}

/** A root-layer floating panel positioned from native layout snapshots. */
export function Popover(props: PopoverProps): JSX.Element {
  const host = useHost();
  const [uncontrolledOpen, setUncontrolledOpen] = createSignal(
    props.defaultOpen ?? false,
  );
  const [position, setPosition] = createSignal({ x: 0, y: 0 });
  const [positioned, setPositioned] = createSignal(false);
  const open = () => props.open ?? uncontrolledOpen();
  let anchor: Handle | undefined;
  let content: Handle | undefined;
  let frame = 0;
  let positionRequest = 0;
  let observer: ResizeObserver | undefined;

  const setOpen = (
    next: boolean,
    reason?: OverlayDismissReason | "trigger",
  ) => {
    if (props.open === undefined) setUncontrolledOpen(next);
    props.onOpenChange?.(next, reason);
  };
  const layer = createOverlayLayer({
    open,
    onDismiss: (reason) => setOpen(false, reason),
    closeOnEscape: () => props.closeOnEscape ?? true,
    returnFocus: () => anchor,
    restoreFocus: () => props.restoreFocus ?? true,
  });

  const updatePosition = async () => {
    if (!open() || !anchor || !content) return;
    const request = ++positionRequest;
    try {
      const result = await computeHostFloatingPosition(anchor, content, host, {
        placement: props.placement ?? "bottom-start",
        middleware: [offset(props.offset ?? 6), flip(), shift({ padding: 8 })],
      });
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
      setPositioned(false);
      observer?.disconnect();
      observer = undefined;
      return;
    }
    observer = new ResizeObserver(schedulePosition);
    if (anchor) observer.observe(anchor as never);
    // The first callback flushes the portal nodes; the second reads their
    // completed native layout if ResizeObserver has not fired first.
    frame = requestAnimationFrame(() => {
      frame = requestAnimationFrame(() => void updatePosition());
    });
  });

  onCleanup(() => {
    cancelAnimationFrame(frame);
    observer?.disconnect();
  });

  const handleEscape = (event: {
    key: string;
    preventDefault(): void;
    stopPropagation(): void;
  }) => layer.onEscape(event);

  return (
    <>
      {props.trigger({
        ref: (node) => {
          anchor = node;
          if (open()) observe(node);
        },
        onClick: (event) => {
          event.stopPropagation();
          setOpen(!open(), "trigger");
        },
        onKeyDown: handleEscape,
        "aria-haspopup": "dialog",
        "aria-expanded": open(),
      })}
      <Show when={open()}>
        <Portal
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
          }}
          onClick={layer.onOutside}
          onKeyDown={handleEscape}
          onWheel={schedulePosition}
        >
          <View
            ref={(node) => {
              content = node;
              observe(node);
            }}
            role="dialog"
            class={props.contentClass}
            style={{
              position: "absolute",
              // The panel must participate in layout before Floating UI can
              // measure it. Keep that measurement pass outside the viewport:
              // Keep the measurement pass outside the viewport so it is also
              // absent from hit testing while the position is unresolved.
              left: positioned() ? `${position().x}px` : "-100000px",
              top: positioned() ? `${position().y}px` : "-100000px",
              ...props.contentStyle,
            }}
            onClick={(event: { stopPropagation(): void }) =>
              event.stopPropagation()
            }
            onKeyDown={handleEscape}
          >
            {props.children}
          </View>
        </Portal>
      </Show>
    </>
  );
}
