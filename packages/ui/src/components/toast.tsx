import checkCircle from "lucide-static/icons/circle-check.svg?raw";
import info from "lucide-static/icons/info.svg?raw";
import triangleAlert from "lucide-static/icons/triangle-alert.svg?raw";
import x from "lucide-static/icons/x.svg?raw";
import type { JSX } from "solid-js";
import { match } from "ts-pattern";
import {
  createNotifications,
  Icon,
  type NotificationDismissReason,
  type NotificationMotionOptions,
  type NotificationPlacement,
  NotificationRegion,
  type Notifications,
  Text,
  View,
} from "../primitives";
import { Button } from "./button";
import { mergeClasses } from "@wabou/core/style";

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

const treatment = (variant: ToastVariant) =>
  match(variant)
    .with("default", () => ({ icon: info, color: "text-accent" }))
    .with("success", () => ({
      icon: checkCircle,
      color: "text-success-primary",
    }))
    .with("warning", () => ({
      icon: triangleAlert,
      color: "text-accent",
    }))
    .with("destructive", () => ({
      icon: triangleAlert,
      color: "text-danger-primary",
    }))
    .exhaustive();

function ToastContent(props: {
  input: ToastInput;
  dismiss(): void;
}): JSX.Element {
  const style = () => treatment(props.input.variant ?? "default");
  return (
    <View class="w-full min-w-0 flex items-start gap-3 rounded-lg border border-subtle bg-surface p-3 shadow-md">
      <Icon
        source={style().icon}
        class={mergeClasses("flex-none mt-0.5", style().color)}
        size={18}
      />
      <View class="min-w-0 flex-1 flex flex-col gap-1">
        <Text class="text-sm font-medium text-primary">
          {props.input.title}
        </Text>
        {props.input.description && (
          <Text class="text-sm whitespace-normal text-muted">
            {props.input.description}
          </Text>
        )}
        {props.input.action && (
          <View class="pt-1 flex items-center">
            <Button
              size="sm"
              variant="outline"
              onClick={() => {
                props.input.action?.onAction();
                if (props.input.action?.dismiss !== false) props.dismiss();
              }}
            >
              {props.input.action.label}
            </Button>
          </View>
        )}
      </View>
      <Button
        size="icon"
        variant="ghost"
        aria-label={`Dismiss ${props.input.title}`}
        class="w-7 h-7 flex-none"
        onClick={props.dismiss}
      >
        <Icon source={x} size={14} />
      </Button>
    </View>
  );
}

/** Create an owner-scoped, styled toast queue over NotificationRegion. */
export function createToasts(options: CreateToastsOptions = {}): Toasts {
  const notifications = createNotifications(options);
  const show = (input: ToastInput) =>
    notifications.show({
      "aria-label": input.title,
      priority: input.variant === "destructive" ? "assertive" : "polite",
      duration: input.duration,
      onDismiss: input.onDismiss,
      content: (controls) => (
        <ToastContent input={input} dismiss={controls.dismiss} />
      ),
    });
  return {
    notifications,
    show,
    success: (title, input = {}) =>
      show({ ...input, title, variant: "success" }),
    warning: (title, input = {}) =>
      show({ ...input, title, variant: "warning" }),
    error: (title, input = {}) =>
      show({ ...input, title, variant: "destructive" }),
    dismiss: (id) => notifications.dismiss(id, "dismiss"),
    clear: notifications.clear,
  };
}

export interface ToasterProps {
  toasts: Toasts;
  placement?: NotificationPlacement;
  class?: string;
  itemClass?: string;
  motion?: false | NotificationMotionOptions;
}

/** Render a non-blocking stack of styled toasts on the floating plane. */
export function Toaster(props: ToasterProps): JSX.Element {
  return (
    <NotificationRegion
      notifications={props.toasts.notifications}
      placement={props.placement ?? "bottom-end"}
      class={props.class}
      itemClass={mergeClasses("w-96 max-w-full", props.itemClass)}
      motion={
        props.motion === undefined
          ? {
              fromY: (props.placement ?? "bottom-end").startsWith("bottom")
                ? 12
                : -12,
            }
          : props.motion
      }
    />
  );
}
