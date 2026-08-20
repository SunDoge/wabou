import { type Handle, Portal } from "@wabou/core/renderer";
import { number, type Shadow, scale2d } from "@wabou/core/style";
import {
  createComponent,
  createEffect,
  createSignal,
  type JSX,
  onCleanup,
  Show,
  untrack,
} from "solid-js";
import { type Easing, useReducedMotion } from "../animation";
import { createOverlayLayer, OverlayPlaneProvider } from "./overlay-layer";
import { createTransitionPresence } from "./transition-presence";
import type { WabouStyle } from "./view";
import { View } from "./view";

export interface ModalEvent {
  stopPropagation(): void;
  preventDefault(): void;
}

export interface ModalKeyEvent extends ModalEvent {
  key: string;
}

export interface ModalTriggerProps {
  ref: (node: Handle) => void;
  onClick: (event: ModalEvent) => void;
  onKeyDown: (event: ModalKeyEvent) => void;
  "aria-haspopup": "dialog";
  "aria-expanded": boolean;
}

export type ModalOpenChangeReason =
  | "trigger"
  | "escape"
  | "backdrop"
  | "programmatic";

export interface ModalControls {
  close(): void;
}

export interface ModalMotionOptions {
  duration?: number;
  ease?: Easing;
  /** Initial content scale around its center. Defaults to 1. */
  fromScale?: number;
}

export interface ModalProps {
  children?: JSX.Element | ((controls: ModalControls) => JSX.Element);
  trigger?: (props: ModalTriggerProps) => JSX.Element;
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?: (open: boolean, reason: ModalOpenChangeReason) => void;
  /** Required until the native semantic tree resolves aria-labelledby. */
  "aria-label": string;
  /** Use `alertdialog` for an interruption that requires an explicit choice. */
  contentRole?: "dialog" | "alertdialog";
  backdropClass?: string;
  backdropStyle?: WabouStyle;
  contentClass?: string;
  contentStyle?: WabouStyle;
  contentShadows?: readonly Shadow[] | null;
  contentRef?: (node: Handle) => void;
  closeOnBackdrop?: boolean;
  closeOnEscape?: boolean;
  /** Overrides the host's default of focusing the first focusable descendant. */
  initialFocus?: () => Handle | undefined;
  restoreFocus?: boolean;
  /** Headless Modal is static by default; styled dialogs opt into motion. */
  motion?: false | ModalMotionOptions;
}

/**
 * A native modal plane with host-enforced focus, hit-test, and accessibility
 * isolation. Visual styling remains explicit so applications can own it.
 */
export function Modal(props: ModalProps): JSX.Element {
  const reducedMotion = useReducedMotion();
  const [uncontrolledOpen, setUncontrolledOpen] = createSignal(
    untrack(() => props.defaultOpen ?? false),
  );
  const open = () => props.open ?? uncontrolledOpen();
  const motion = untrack(() => props.motion);
  const motionOptions = motion === false ? undefined : motion;
  const motionEnabled = motionOptions !== undefined;
  const presence = createTransitionPresence(open, {
    duration: motionOptions?.duration ?? (motionEnabled ? 0.16 : 0),
    ease: motionOptions?.ease ?? (motionEnabled ? "easeOut" : "linear"),
    reducedMotion: () => !motionEnabled || reducedMotion(),
  });
  const motionFromScale = () => motionOptions?.fromScale ?? 1;
  let trigger: Handle | undefined;
  let focusFrame = 0;
  let wasOpenForInitialFocus = false;

  const setOpen = (next: boolean, reason: ModalOpenChangeReason) => {
    if (props.open === undefined) setUncontrolledOpen(next);
    props.onOpenChange?.(next, reason);
  };
  const close = (reason: ModalOpenChangeReason = "programmatic") =>
    setOpen(false, reason);
  const controls: ModalControls = {
    close: () => close("programmatic"),
  };
  const layer = createOverlayLayer({
    open,
    plane: () => "modal",
    onDismiss: (reason) => close(reason === "outside" ? "backdrop" : "escape"),
    closeOnEscape: () => props.closeOnEscape ?? true,
    closeOnOutside: () => props.closeOnBackdrop ?? true,
    restoreFocus: () => props.restoreFocus ?? true,
    returnFocus: () => trigger,
  });
  const handleEscape = (event: ModalKeyEvent) => layer.onEscape(event);

  createEffect(open, (isOpen) => {
    if (isOpen && !wasOpenForInitialFocus && props.initialFocus) {
      cancelAnimationFrame(focusFrame);
      focusFrame = requestAnimationFrame(() => {
        focusFrame = 0;
        props.initialFocus?.()?.focus();
      });
    } else if (!isOpen) {
      if (focusFrame) cancelAnimationFrame(focusFrame);
      focusFrame = 0;
    }
    wasOpenForInitialFocus = isOpen;
  });
  onCleanup(() => {
    if (focusFrame) cancelAnimationFrame(focusFrame);
  });

  const triggerProps: ModalTriggerProps = {
    ref: (node) => {
      trigger = node;
    },
    onClick: (event) => {
      event.stopPropagation();
      setOpen(true, "trigger");
    },
    onKeyDown: handleEscape,
    "aria-haspopup": "dialog",
    get "aria-expanded"() {
      return open();
    },
  };

  const overlay = createComponent(
    Show as unknown as (props: {
      when: boolean;
      children: JSX.Element;
    }) => JSX.Element,
    {
      get when() {
        return presence.mounted();
      },
      get children() {
        return createComponent(Portal, {
          plane: "modal",
          role: "presentation",
          "aria-modal": "true",
          get focusContained() {
            return open();
          },
          get interactionBlocked() {
            return !open();
          },
          get class() {
            return props.backdropClass;
          },
          get style() {
            return {
              position: "absolute",
              left: 0,
              top: 0,
              width: "100%",
              height: "100%",
              display: "flex",
              "align-items": "center",
              "justify-content": "center",
              ...props.backdropStyle,
              opacity: number(presence.progress()),
              "pointer-events": open() ? "auto" : "none",
              // Portal containers share one native plane. Make open order
              // explicit so nested overlays paint above their owning modal.
              "z-index": layer.zIndex(),
            };
          },
          onClick: layer.onOutside,
          onKeyDown: handleEscape,
          get children() {
            return createComponent(View, {
              get ref() {
                return props.contentRef;
              },
              get role() {
                return props.contentRole ?? "dialog";
              },
              "aria-modal": "true",
              get "aria-label"() {
                return props["aria-label"];
              },
              get class() {
                return props.contentClass;
              },
              get style() {
                return props.contentStyle;
              },
              get shadows() {
                return props.contentShadows;
              },
              get transform() {
                return scale2d(
                  motionFromScale() +
                    presence.progress() * (1 - motionFromScale()),
                );
              },
              get interactionBlocked() {
                return !open();
              },
              get "aria-hidden"() {
                return open() ? undefined : "true";
              },
              onClick: (event: ModalEvent) => event.stopPropagation(),
              get children() {
                return createComponent(OverlayPlaneProvider, {
                  plane: "modal",
                  get children() {
                    return typeof props.children === "function"
                      ? props.children(controls)
                      : props.children;
                  },
                });
              },
            });
          },
        });
      },
    },
  );

  const triggerElement = untrack(() => props.trigger?.(triggerProps));
  return [triggerElement, overlay] as JSX.Element;
}
