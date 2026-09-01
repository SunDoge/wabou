import {
  type Handle,
  Portal,
  type WabouNativeTransition,
} from "@wabou/core/renderer";
import {
  type Affine2D,
  mergeClasses,
  rgba,
  type Shadow,
} from "@wabou/core/style";
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
import { createPresence } from "./presence";
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
  /** Initial horizontal offset in logical pixels. Defaults to 0. */
  fromX?: number;
  /** Initial vertical offset in logical pixels. Defaults to 0. */
  fromY?: number;
}

export interface ModalVisualState {
  /** The modal currently owns focus, pointer input, and the native scrim. */
  active: boolean;
  /** Keep authored backdrop visuals mounted while a fade-out completes. */
  retainBackdropVisuals: boolean;
  /** A closing edge panel remains mounted, but must expose the application immediately. */
  transparentBackdrop: boolean;
}

/**
 * Derive every modal-plane policy from the committed controlled state.
 *
 * Presence only controls how long the subtree remains mounted. It must never
 * prolong focus containment, hit testing, blur, or an opaque scrim after the
 * owner has committed `open=false`.
 */
export function modalVisualState(
  open: boolean,
  backdropFade: boolean | undefined,
): ModalVisualState {
  return {
    active: open,
    retainBackdropVisuals: open || backdropFade !== false,
    transparentBackdrop: !open && backdropFade === false,
  };
}

export function modalMotionTransform(
  options: ModalMotionOptions | undefined,
  progress: number,
): Affine2D {
  const scale =
    (options?.fromScale ?? 1) + progress * (1 - (options?.fromScale ?? 1));
  const remaining = 1 - progress;
  const offset = (value: number) => {
    const result = value * remaining;
    return result === 0 ? 0 : result;
  };
  return [
    scale,
    0,
    0,
    scale,
    offset(options?.fromX ?? 0),
    offset(options?.fromY ?? 0),
  ];
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
  /** Keep the backdrop visible while the content exits. Edge panels disable this. */
  backdropFade?: boolean;
  contentClass?: string;
  contentStyle?: WabouStyle;
  /** Fade the content with the backdrop. Edge panels disable this and slide as solid surfaces. */
  contentFade?: boolean;
  /** Composes component-specific movement with the modal presence transform. */
  contentTransform?: (base: Affine2D, presenceProgress: number) => Affine2D;
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
  const duration = motionOptions?.duration ?? (motionEnabled ? 0.16 : 0);
  const presence = createPresence(open);
  const visualState = () => modalVisualState(open(), props.backdropFade);
  const [transitionGeneration, setTransitionGeneration] = createSignal(0);
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

  createEffect(
    () => [open(), reducedMotion()] as const,
    ([isOpen, prefersReducedMotion]) => {
      setTransitionGeneration((value) => value + 1);
      if (!motionEnabled || prefersReducedMotion || duration <= 0) {
        if (isOpen) presence.finishEnter();
        else presence.finishExit();
      }
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
    },
  );
  onCleanup(() => {
    if (focusFrame) cancelAnimationFrame(focusFrame);
  });

  const nativeTransition = (
    fromTransform: Affine2D,
    toTransform: Affine2D,
    fromOpacity: number,
    toOpacity: number,
  ): WabouNativeTransition | undefined => {
    if (!motionEnabled || reducedMotion()) return undefined;
    const authoredEase = motionOptions?.ease;
    const easing =
      authoredEase === "linear" ||
      authoredEase === "easeInOut" ||
      authoredEase === "easeOut"
        ? authoredEase
        : "easeInOut";
    return {
      generation: transitionGeneration(),
      duration,
      easing,
      fromTransform,
      toTransform,
      fromOpacity,
      toOpacity,
    };
  };

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
            return visualState().active;
          },
          get interactionBlocked() {
            return !visualState().active;
          },
          get class() {
            const visual = visualState();
            return mergeClasses(
              visual.active && "backdrop-blur-sm",
              visual.retainBackdropVisuals && props.backdropClass,
            );
          },
          get style() {
            const visual = visualState();
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
              // An edge panel may remain mounted for its slide-out transition,
              // but its modal scrim must stop affecting the application at the
              // moment the logical open state changes.
              ...(visual.transparentBackdrop
                ? { "background-color": rgba(0x00000000) }
                : undefined),
              "pointer-events": visual.active ? "auto" : "none",
              // Portal containers share one native plane. Make open order
              // explicit so nested overlays paint above their owning modal.
              "z-index": layer.zIndex(),
            };
          },
          get nativeTransition() {
            if (props.backdropFade === false) return undefined;
            const entering = open();
            return nativeTransition(
              [1, 0, 0, 1, 0, 0],
              [1, 0, 0, 1, 0, 0],
              entering ? 0 : 1,
              entering ? 1 : 0,
            );
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
                const base = modalMotionTransform(
                  motionOptions,
                  open() ? 1 : 0,
                );
                return props.contentTransform?.(base, open() ? 1 : 0) ?? base;
              },
              get nativeTransition() {
                const entering = open();
                const fromProgress = entering ? 0 : 1;
                const toProgress = entering ? 1 : 0;
                const from = modalMotionTransform(motionOptions, fromProgress);
                const to = modalMotionTransform(motionOptions, toProgress);
                return nativeTransition(
                  props.contentTransform?.(from, fromProgress) ?? from,
                  props.contentTransform?.(to, toProgress) ?? to,
                  props.contentFade === false ? 1 : fromProgress,
                  props.contentFade === false ? 1 : toProgress,
                );
              },
              onTransitionEnd: (event) => {
                if (event.generation !== transitionGeneration()) return;
                if (open()) presence.finishEnter();
                else presence.finishExit();
              },
              get interactionBlocked() {
                return !visualState().active;
              },
              get "aria-hidden"() {
                return visualState().active ? undefined : "true";
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
