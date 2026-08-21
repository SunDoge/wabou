import {
  createComponent,
  createContext,
  type JSX,
  omit,
  useContext,
} from "solid-js";
import { match } from "ts-pattern";
import { Text, type TextProps, View, type ViewProps } from "../primitives";
import { join } from "./class-names";

export type AlertVariant = "default" | "destructive";

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
    .with("destructive", () => ({
      container: "border-danger bg-danger-surface",
      title: "text-danger-primary",
      description: "text-danger-primary",
    }))
    .exhaustive();
}

export interface AlertProps extends Omit<ViewProps, "role"> {
  variant?: AlertVariant;
  /** Optional leading graphic with caller-owned size and color. */
  icon?: JSX.Element;
  /** Convenience form; compound usage can render AlertTitle directly. */
  title?: string;
}

/** A native status callout with shadcn-compatible compound composition. */
export function Alert(props: AlertProps): JSX.Element {
  const variant = () => props.variant ?? "default";
  const colors = () => alertColors(variant());
  const forwarded = omit(
    props,
    "variant",
    "icon",
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
          class={join(
            "w-full min-w-0 flex flex-row items-start gap-3 rounded-lg border p-4 shadow-xs",
            colors().container,
            props.class,
          )}
        >
          {props.icon === undefined ? null : (
            <View class="flex-none pt-0.5">{props.icon}</View>
          )}
          <View class="min-w-0 flex-1 flex flex-col gap-1">{content()}</View>
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
      class={join(
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
      class={join(
        "w-full min-w-0 whitespace-normal text-sm",
        alertColors(context.variant()).description,
        props.class,
      )}
    />
  );
}
