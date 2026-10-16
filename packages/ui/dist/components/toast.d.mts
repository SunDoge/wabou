import { NotificationDismissReason, NotificationMotionOptions, NotificationPlacement, Notifications } from "../primitives/notification.mjs";
import "../primitives.mjs";
import { JSX } from "solid-js";
//#region src/components/toast.d.ts
export type ToastVariant = "default" | "success" | "warning" | "destructive";
export interface ToastAction {
  label: string;
  onAction(): void;
  /** Defaults to true. */
  dismiss?: boolean;
}
export interface ToastInput {
  title: string;
  description?: string;
  variant?: ToastVariant;
  action?: ToastAction;
  duration?: number;
  onDismiss?: (reason: NotificationDismissReason) => void;
}
export interface Toasts {
  /** The unstyled queue remains available for advanced composition. */
  readonly notifications: Notifications;
  show(input: ToastInput): number;
  success(title: string, input?: Omit<ToastInput, "title" | "variant">): number;
  warning(title: string, input?: Omit<ToastInput, "title" | "variant">): number;
  error(title: string, input?: Omit<ToastInput, "title" | "variant">): number;
  dismiss(id: number): boolean;
  clear(): void;
}
export interface CreateToastsOptions {
  defaultDuration?: number;
  limit?: number;
}
/** Create an owner-scoped, styled toast queue over NotificationRegion. */
export declare function createToasts(options?: CreateToastsOptions): Toasts;
export interface ToasterProps {
  toasts: Toasts;
  placement?: NotificationPlacement;
  class?: string;
  itemClass?: string;
  motion?: false | NotificationMotionOptions;
}
/** Render a non-blocking stack of styled toasts on the floating plane. */
export declare function Toaster(props: ToasterProps): JSX.Element;
//#endregion
//# sourceMappingURL=toast.d.mts.map