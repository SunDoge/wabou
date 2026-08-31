import { Portal } from "@wabou/core/renderer";
import { number, translate2d } from "@wabou/core/style";
import {
  type Accessor,
  createComponent,
  createEffect,
  createSignal,
  For,
  type JSX,
  onCleanup,
  untrack,
} from "solid-js";
import { type Easing, useReducedMotion } from "../animation";
import { createRetainedItems, type RetainedItem } from "./retained-items";
import { createTransitionPresence } from "./transition-presence";
import { View, type WabouStyle } from "./view";

export type NotificationPriority = "polite" | "assertive";
export type NotificationDismissReason =
  | "dismiss"
  | "timeout"
  | "overflow"
  | "programmatic";

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

interface NotificationRecord {
  item: NotificationItem;
  autoDismiss: boolean;
  pauseCount: number;
  remaining: number;
  startedAt: number;
  timer?: ReturnType<typeof setTimeout>;
}

const finiteNonNegative = (value: number, fallback: number) =>
  Number.isFinite(value) ? Math.max(0, value) : fallback;

/** Create an owner-scoped notification queue with explicit JavaScript timers. */
export function createNotifications(
  options: NotificationsOptions = {},
): Notifications {
  const records = new Map<number, NotificationRecord>();
  // Keep the imperative queue authoritative. Solid 2 can defer signal writes
  // until the owner transaction commits, while callers of show/dismiss need
  // their following items() read to observe the command synchronously.
  const [revision, setRevision] = createSignal(0, { ownedWrite: true });
  const items = (): readonly NotificationItem[] => {
    revision();
    return [...records.values()].map((record) => record.item);
  };
  const defaultDuration = finiteNonNegative(
    options.defaultDuration ?? 5_000,
    5_000,
  );
  const configuredLimit = options.limit ?? 5;
  const limit = Number.isFinite(configuredLimit)
    ? Math.max(1, Math.floor(configuredLimit))
    : 5;
  let nextId = 1;

  const dismiss = (
    id: number,
    reason: NotificationDismissReason = "programmatic",
  ) => {
    const record = records.get(id);
    if (!record) return false;
    if (record.timer !== undefined) clearTimeout(record.timer);
    records.delete(id);
    setRevision((current) => current + 1);
    record.item.onDismiss?.(reason);
    return true;
  };

  const schedule = (record: NotificationRecord) => {
    if (!record.autoDismiss) return;
    if (record.remaining <= 0) {
      dismiss(record.item.id, "timeout");
      return;
    }
    record.startedAt = Date.now();
    record.timer = setTimeout(
      () => dismiss(record.item.id, "timeout"),
      record.remaining,
    );
  };

  const notifications: Notifications = {
    items,
    show(input) {
      while (records.size >= limit) {
        const oldest = records.values().next().value?.item;
        if (!oldest || !dismiss(oldest.id, "overflow")) break;
      }
      const duration = finiteNonNegative(
        input.duration ?? defaultDuration,
        defaultDuration,
      );
      const item: NotificationItem = {
        ...input,
        id: nextId++,
        duration,
        priority: input.priority ?? "polite",
      };
      const record: NotificationRecord = {
        item,
        autoDismiss: duration > 0,
        pauseCount: 0,
        remaining: duration,
        startedAt: 0,
      };
      records.set(item.id, record);
      setRevision((current) => current + 1);
      schedule(record);
      return item.id;
    },
    dismiss,
    pause(id) {
      const record = records.get(id);
      if (!record) return;
      record.pauseCount++;
      if (record.pauseCount > 1 || record.timer === undefined) return;
      clearTimeout(record.timer);
      record.timer = undefined;
      record.remaining = Math.max(
        0,
        record.remaining - (Date.now() - record.startedAt),
      );
    },
    resume(id) {
      const record = records.get(id);
      if (!record || record.pauseCount === 0) return;
      record.pauseCount--;
      if (record.pauseCount > 0 || record.timer !== undefined) return;
      schedule(record);
    },
    clear() {
      for (const id of [...records.keys()]) dismiss(id, "programmatic");
    },
  };

  onCleanup(() => {
    for (const record of records.values()) {
      if (record.timer !== undefined) clearTimeout(record.timer);
    }
    records.clear();
  });
  return notifications;
}

export type NotificationPlacement =
  | "top-start"
  | "top"
  | "top-end"
  | "bottom-start"
  | "bottom"
  | "bottom-end";

export interface NotificationRegionProps {
  notifications: Notifications;
  placement?: NotificationPlacement;
  class?: string;
  style?: WabouStyle;
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

const alignment = (placement: NotificationPlacement) => ({
  "align-items": placement.endsWith("start")
    ? "flex-start"
    : placement.endsWith("end")
      ? "flex-end"
      : "center",
  "justify-content": placement.startsWith("bottom") ? "flex-end" : "flex-start",
});

const renderNotificationPortal = (
  props: NotificationRegionProps,
  children: JSX.Element,
) =>
  createComponent(Portal, {
    plane: "floating",
    role: "presentation",
    get class() {
      return `pointer-events-none ${props.class ?? ""}`;
    },
    get style() {
      const placement = props.placement ?? "top-end";
      return {
        position: "absolute",
        left: 0,
        top: 0,
        width: "100%",
        height: "100%",
        display: "flex",
        "flex-direction": "column",
        gap: 8,
        padding: 16,
        ...alignment(placement),
        ...props.style,
      };
    },
    children,
  });

/** Render a non-blocking stack on the native floating overlay plane. */
export function NotificationRegion(
  props: NotificationRegionProps,
): JSX.Element {
  const motion = untrack(() => props.motion);
  if (motion === undefined || motion === false) {
    const items = createComponent(
      For as unknown as (props: {
        each: readonly NotificationItem[];
        children: (item: NotificationItem) => JSX.Element;
      }) => JSX.Element,
      {
        get each() {
          return props.notifications.items();
        },
        children: (item) =>
          createComponent(View, {
            role: item.priority === "assertive" ? "alert" : "status",
            "aria-label": item["aria-label"],
            get class() {
              return `pointer-events-auto ${props.itemClass ?? ""}`;
            },
            get style() {
              return props.itemStyle;
            },
            onPointerEnter: () => props.notifications.pause(item.id),
            onPointerLeave: () => props.notifications.resume(item.id),
            onFocusIn: () => props.notifications.pause(item.id),
            onFocusOut: () => props.notifications.resume(item.id),
            get children() {
              return item.content({
                dismiss: () => props.notifications.dismiss(item.id, "dismiss"),
              });
            },
          }),
      },
    );
    return renderNotificationPortal(props, items);
  }

  const reducedMotion = useReducedMotion();
  const retained = createRetainedItems(
    props.notifications.items,
    (item) => item.id,
  );
  const renderAnimatedItem = (
    retainedItem: RetainedItem<NotificationItem, number>,
  ) => {
    const logicallyPresent = retainedItem.present;
    const presence = createTransitionPresence(logicallyPresent, {
      initialProgress: 0,
      duration: motion.duration ?? 0.18,
      ease: motion.ease ?? "easeOut",
      reducedMotion,
    });
    createEffect(presence.phase, (phase) => {
      if (phase === "unmounted") retained.release(retainedItem.key);
    });
    const remaining = () => 1 - presence.progress();
    return createComponent(View, {
      get role() {
        return retainedItem.value().priority === "assertive"
          ? "alert"
          : "status";
      },
      get "aria-label"() {
        return retainedItem.value()["aria-label"];
      },
      get "aria-hidden"() {
        return logicallyPresent() ? undefined : "true";
      },
      get interactionBlocked() {
        return !logicallyPresent();
      },
      get transform() {
        return translate2d(
          (motion.fromX ?? 0) * remaining(),
          (motion.fromY ?? 0) * remaining(),
        );
      },
      get class() {
        return `pointer-events-auto ${props.itemClass ?? ""}`;
      },
      get style() {
        return {
          ...props.itemStyle,
          opacity: number(presence.progress()),
        };
      },
      onPointerEnter: () => props.notifications.pause(retainedItem.key),
      onPointerLeave: () => props.notifications.resume(retainedItem.key),
      onFocusIn: () => props.notifications.pause(retainedItem.key),
      onFocusOut: () => props.notifications.resume(retainedItem.key),
      get children() {
        const item = retainedItem.value();
        return item.content({
          dismiss: () => props.notifications.dismiss(item.id, "dismiss"),
        });
      },
    });
  };
  const items = createComponent(
    For as unknown as (props: {
      each: readonly RetainedItem<NotificationItem, number>[];
      children: (item: RetainedItem<NotificationItem, number>) => JSX.Element;
    }) => JSX.Element,
    {
      get each() {
        return retained.entries();
      },
      children: renderAnimatedItem,
    },
  );
  return renderNotificationPortal(props, items);
}
