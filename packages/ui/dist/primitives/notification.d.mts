import { Easing } from "../animation.mjs";
import { WabouStyle } from "./view.mjs";
import { Accessor, JSX } from "solid-js";
//#region src/primitives/notification.d.ts
export type NotificationPriority = "polite" | "assertive";
export type NotificationDismissReason = "dismiss" | "timeout" | "overflow" | "programmatic";
export interface NotificationInput {
  /** Accessible announcement independent of the rendered visual content. */
  "aria-label": string;
  content: (controls: NotificationControls) => JSX.Element;
  priority?: NotificationPriority;
  /** Milliseconds before dismissal. Zero disables automatic dismissal. */
  duration?: number;
  onDismiss?: (reason: NotificationDismissReason) => void;
}
export interface NotificationItem extends NotificationInput {
  id: number;
}
export interface NotificationControls {
  dismiss(): void;
}
export interface NotificationsOptions {
  defaultDuration?: number;
  limit?: number;
}
export interface Notifications {
  readonly items: Accessor<readonly NotificationItem[]>;
  show(input: NotificationInput): number;
  dismiss(id: number, reason?: NotificationDismissReason): boolean;
  pause(id: number): void;
  resume(id: number): void;
  clear(): void;
}
/** Create an owner-scoped notification queue with explicit JavaScript timers. */
export declare function createNotifications(options?: NotificationsOptions): Notifications;
export type NotificationPlacement = "top-start" | "top" | "top-end" | "bottom-start" | "bottom" | "bottom-end";
export interface NotificationRegionProps {
  notifications: Notifications;
  placement?: NotificationPlacement;
  class?: string;
  style?: WabouStyle;
  /** Width and presentation of the native notification stack container. */
  stackClass?: string;
  itemClass?: string;
  itemStyle?: WabouStyle;
  /** Headless regions are static unless motion is explicitly requested. */
  motion?: false | NotificationMotionOptions;
}
export interface NotificationMotionOptions {
  duration?: number;
  ease?: Easing;
  /** Initial horizontal offset in logical pixels. */
  fromX?: number;
  /** Initial vertical offset in logical pixels. */
  fromY?: number;
}
/** Render a non-blocking stack on the native floating overlay plane. */
export declare function NotificationRegion(props: NotificationRegionProps): JSX.Element;
//#endregion
//# sourceMappingURL=notification.d.mts.map