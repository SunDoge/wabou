import { type Handle, Portal } from "@wabou/solid-renderer";
import {
  createComponent,
  createComputed,
  createSignal,
  type JSX,
  onCleanup,
  Show,
} from "solid-js";
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

export interface ModalProps {
  children?: JSX.Element | ((controls: ModalControls) => JSX.Element);
  trigger?: (props: ModalTriggerProps) => JSX.Element;
  open?: boolean;
  defaultOpen?: boolean;
  onOpenChange?: (open: boolean, reason: ModalOpenChangeReason) => void;
  /** Required until the native semantic tree resolves aria-labelledby. */
  "aria-label": string;
  backdropClass?: string;
  backdropStyle?: WabouStyle;
  contentClass?: string;
  contentStyle?: WabouStyle;
  contentRef?: (node: Handle) => void;
  closeOnBackdrop?: boolean;
  closeOnEscape?: boolean;
  /** Overrides the host's default of focusing the first focusable descendant. */
  initialFocus?: () => Handle | undefined;
  restoreFocus?: boolean;
}

/**
 * A native modal plane with host-enforced focus, hit-test, and accessibility
 * isolation. Visual styling remains explicit so applications can own it.
 */
export function Modal(props: ModalProps): JSX.Element {
  const [uncontrolledOpen, setUncontrolledOpen] = createSignal(
    props.defaultOpen ?? false,
  );
  const open = () => props.open ?? uncontrolledOpen();
  let trigger: Handle | undefined;
  let focusFrame = 0;
  let wasOpen = false;

  const setOpen = (next: boolean, reason: ModalOpenChangeReason) => {
    if (props.open === undefined) setUncontrolledOpen(next);
    props.onOpenChange?.(next, reason);
  };
  const close = (reason: ModalOpenChangeReason = "programmatic") =>
    setOpen(false, reason);
  const controls: ModalControls = {
    close: () => close("programmatic"),
  };
  const handleEscape = (event: ModalKeyEvent) => {
    if (!open() || event.key !== "Escape" || props.closeOnEscape === false)
      return;
    event.preventDefault();
    close("escape");
  };

  createComputed(() => {
    const isOpen = open();
    if (isOpen && !wasOpen && props.initialFocus) {
      cancelAnimationFrame(focusFrame);
      focusFrame = requestAnimationFrame(() => {
        focusFrame = 0;
        props.initialFocus?.()?.focus();
      });
    } else if (!isOpen && wasOpen) {
      if (focusFrame) cancelAnimationFrame(focusFrame);
      focusFrame = 0;
      if (props.restoreFocus ?? true) trigger?.focus();
    }
    wasOpen = isOpen;
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
        return open();
      },
      get children() {
        return createComponent(Portal, {
          plane: "modal",
          role: "presentation",
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
              alignItems: "center",
              justifyContent: "center",
              ...props.backdropStyle,
            };
          },
          onClick: () => {
            if (props.closeOnBackdrop ?? true) close("backdrop");
          },
          onKeyDown: handleEscape,
          get children() {
            return createComponent(View, {
              get ref() {
                return props.contentRef;
              },
              role: "dialog",
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
              onClick: (event: ModalEvent) => event.stopPropagation(),
              get children() {
                return typeof props.children === "function"
                  ? props.children(controls)
                  : props.children;
              },
            });
          },
        });
      },
    },
  );

  return [props.trigger?.(triggerProps), overlay] as JSX.Element;
}
