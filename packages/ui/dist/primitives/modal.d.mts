import { Easing } from "../animation.mjs";
import { WabouStyle as WabouStyle$1 } from "./view.mjs";
import { Affine2D, Shadow } from "@wabou/core/style";
import { JSX } from "solid-js";
import { Handle } from "@wabou/core/renderer";
//#region src/primitives/modal.d.ts
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
export type ModalOpenChangeReason = "trigger" | "escape" | "backdrop" | "programmatic";
export interface ModalControls {
  close(): void;
}
export interface ModalMotionOptions {
  duration?: number;
  ease?: Easing;
  /** Enter-only timing override. Falls back to `duration`. */
  enterDuration?: number;
  /** Exit-only timing override. Falls back to `duration`. */
  exitDuration?: number;
  /** Enter-only easing override. Falls back to `ease`. */
  enterEase?: Easing;
  /** Exit-only easing override. Falls back to `ease`. */
  exitEase?: Easing;
  /** Initial content scale around its center. Defaults to 1. */
  fromScale?: number;
  /** Initial horizontal offset in logical pixels. Defaults to 0. */
  fromX?: number;
  /** Initial vertical offset in logical pixels. Defaults to 0. */
  fromY?: number;
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
  backdropStyle?: WabouStyle$1;
  /** Keep the backdrop visible while the content exits. Edge panels disable this. */
  backdropFade?: boolean;
  contentClass?: string;
  contentStyle?: WabouStyle$1;
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
export declare function Modal(props: ModalProps): JSX.Element;
//#endregion
//# sourceMappingURL=modal.d.mts.map