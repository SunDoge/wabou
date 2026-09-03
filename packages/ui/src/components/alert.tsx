import { mergeClasses } from "@wabou/core/style";
import checkCircle from "lucide-static/icons/circle-check.svg?raw";
import circleX from "lucide-static/icons/circle-x.svg?raw";
import info from "lucide-static/icons/info.svg?raw";
import triangleAlert from "lucide-static/icons/triangle-alert.svg?raw";
import x from "lucide-static/icons/x.svg?raw";
import {
  createComponent,
  createContext,
  type JSX,
  omit,
  useContext,
} from "solid-js";
import { match, P } from "ts-pattern";
import {
  Icon,
  Text,
  type TextProps,
  View,
  type ViewProps,
} from "../primitives";
import { Button } from "./button";
import { componentsSurfaceClass } from "./theme";

export type AlertVariant =
  | "default"
  | "info"
  | "success"
  | "warning"
  | "error"
  | "destructive";
export type AlertSize = "sm" | "default" | "lg";

interface AlertContextValue {
  variant(): AlertVariant;
}

const AlertContext = createContext<AlertContextValue>({
  variant: () => "default",
});

export function alertColors(variant: AlertVariant): {
  container: string;
  title: string;
  description: string;
} {
  return match(variant)
    .with("default", () => ({
      container: "border-subtle bg-surface",
      title: "text-primary",
      description: "text-secondary",
    }))
    .with("info", () => ({
      container: "border-accent bg-selected",
      title: "text-accent",
      description: "text-secondary",
    }))
    .with("success", () => ({
      container: "border-success-primary bg-success-surface",
      title: "text-success-primary",
      description: "text-success-primary",
    }))
    .with("warning", () => ({
      container: "border-strong bg-control",
      title: "text-primary",
      description: "text-secondary",
    }))
    .with(P.union("error", "destructive"), () => ({
      container: "border-danger bg-danger-surface",
      title: "text-danger-primary",
      description: "text-danger-primary",
    }))
    .exhaustive();
}

function alertGeometry(size: AlertSize): string {
  return match(size)
    .with("sm", () => "gap-2 px-3 py-2")
    .with("default", () => "gap-3 px-4 py-2.5")
    .with("lg", () => "gap-3 px-5 py-3")
    .exhaustive();
}

function alertIcon(variant: AlertVariant): string {
  return match(variant)
    .with(P.union("default", "info"), () => info)
    .with("success", () => checkCircle)
    .with("warning", () => triangleAlert)
    .with(P.union("error", "destructive"), () => circleX)
    .exhaustive();
}

export interface AlertProps extends Omit<ViewProps, "role"> {
  variant?: AlertVariant;
  size?: AlertSize;
  /** Optional leading graphic. Pass false to suppress the semantic default. */
  icon?: JSX.Element | false;
  /** Edge-to-edge status strip without container chrome. */
  banner?: boolean;
  /** Show a close control and delegate visibility to the owner. */
  onClose?: () => void;
  /** Convenience form; compound usage can render AlertTitle directly. */
  title?: string;
}

/** A native status callout with shadcn-compatible compound composition. */
export function Alert(props: AlertProps): JSX.Element {
  const variant = () => props.variant ?? "default";
  const size = () => props.size ?? "default";
  const colors = () => alertColors(variant());
  const forwarded = omit(
    props,
    "variant",
    "size",
    "icon",
    "banner",
    "onClose",
    "title",
    "class",
    "children",
  );
  const content = () =>
    props.title === undefined ? (
      props.children
    ) : (
      <>
        <AlertTitle>{props.title}</AlertTitle>
        {props.children === undefined ? null : (
          <AlertDescription>{props.children}</AlertDescription>
        )}
      </>
    );
  return createComponent(AlertContext, {
    value: { variant },
    get children() {
      return (
        <View
          {...forwarded}
          role="alert"
          aria-label={props["aria-label"] ?? props.title}
          class={mergeClasses(
            "w-full min-w-0 flex flex-row items-start",
            alertGeometry(size()),
            componentsSurfaceClass("raised"),
            colors().container,
            props.banner && "rounded-none border-0",
            props.class,
          )}
        >
          {props.icon === false ? null : (
            <View class="h-5 flex-none flex items-center">
              {props.icon ?? (
                <Icon
                  source={alertIcon(variant())}
                  size={16}
                  class={colors().title}
                />
              )}
            </View>
          )}
          <View class="min-w-0 flex-1 flex flex-col gap-1">{content()}</View>
          {props.onClose && (
            <Button
              size="icon"
              variant="ghost"
              aria-label={`Dismiss ${props["aria-label"] ?? props.title ?? "alert"}`}
              class="w-7 h-7 flex-none"
              onClick={props.onClose}
            >
              <Icon source={x} size={14} />
            </Button>
          )}
        </View>
      );
    },
  });
}

export function AlertTitle(props: TextProps): JSX.Element {
  const context = useContext(AlertContext);
  return (
    <Text
      {...props}
      class={mergeClasses(
        "w-full min-w-0 text-sm font-semibold",
        alertColors(context.variant()).title,
        props.class,
      )}
    />
  );
}

export function AlertDescription(props: TextProps): JSX.Element {
  const context = useContext(AlertContext);
  return (
    <Text
      {...props}
      class={mergeClasses(
        "w-full min-w-0 whitespace-normal text-sm",
        alertColors(context.variant()).description,
        props.class,
      )}
    />
  );
}

/** Recovery and acknowledgement controls belonging to an alert. */
export function AlertActions(props: ViewProps): JSX.Element {
  return (
    <View
      {...props}
      role={props.role ?? "group"}
      class={mergeClasses(
        "w-full min-w-0 mt-2 flex flex-row flex-wrap items-center gap-2",
        props.class,
      )}
    >
      {props.children}
    </View>
  );
}
