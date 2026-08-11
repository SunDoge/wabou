import { dispatchEffect, effectOps } from "./effects";
import { usePlatformServices } from "./platform-context";

export interface NotificationOptions {
  readonly title: string;
  readonly body?: string;
  readonly icon?: string;
  readonly silent?: boolean;
}

export interface Notification {
  show(options: NotificationOptions): Promise<void>;
}

export const notification: Notification = Object.freeze({
  show(options: NotificationOptions) {
    return dispatchEffect<null>(effectOps.notificationShow, {
      ...options,
      title: String(options.title),
      silent: options.silent ?? false,
    }).then(() => undefined);
  },
});

export function useNotification(): Notification {
  return usePlatformServices().notification ?? notification;
}
