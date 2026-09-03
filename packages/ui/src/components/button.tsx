import { mergeClasses } from "@wabou/core/style";
import { type JSX, omit, Show } from "solid-js";
import { match, P } from "ts-pattern";
import {
  type ButtonState,
  Button as HeadlessButton,
  type ButtonProps as HeadlessButtonProps,
  type WabouStyle,
} from "../primitives";
import {
  buttonGroupItemCorners,
  useButtonGroupItem,
} from "./button-group-context";
import { Spinner } from "./display";
import {
  componentsControlContentSize,
  componentsControlSize,
  componentsDisabledControlClass,
} from "./theme";

export type ButtonVariant =
  | "default"
  | "secondary"
  | "outline"
  | "ghost"
  | "destructive";
export type ButtonSize = "sm" | "default" | "lg" | "icon";

export interface ButtonProps
  extends Omit<HeadlessButtonProps, "variant" | "tone"> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  class?: string;
  style?: HeadlessButtonProps["style"];
  /** Disable activation and replace the leading content with a native spinner. */
  loading?: boolean;
  /** Visible label used while loading. Defaults to the ordinary children. */
  loadingLabel?: string;
}

function buttonColors(
  variant: ButtonVariant,
  state: ButtonState,
  visuallyDisabled = false,
): string {
  if (visuallyDisabled) return "bg-surface-muted border-subtle text-muted";
  const focus = state.focusVisible ? "border-focus" : "";
  // A selected button is a persistent active control. Match gpui-component's
  // state ordering: selected owns the active palette and hover must not wash
  // it out while an associated page, menu or mode remains active.
  const active = state.pressed || state.selected;
  const passiveBorder = (value: ButtonVariant) =>
    match(value)
      .with("outline", () => "border-strong")
      .with(
        P.union("default", "secondary", "ghost", "destructive"),
        () => "border-transparent",
      )
      .exhaustive();

  return match({ variant, pressed: active, hovered: state.hovered })
    .with({ variant: "default", pressed: true }, () =>
      mergeClasses(
        "bg-accent-pressed border-transparent text-on-accent",
        focus,
      ),
    )
    .with({ variant: "default", hovered: true }, () =>
      mergeClasses("bg-accent-hover border-transparent text-on-accent", focus),
    )
    .with({ variant: "default" }, () =>
      mergeClasses("bg-accent border-transparent text-on-accent", focus),
    )
    .with({ variant: "destructive", pressed: true }, () =>
      mergeClasses(
        "bg-danger-pressed border-transparent text-on-accent",
        focus,
      ),
    )
    .with({ variant: "destructive", hovered: true }, () =>
      mergeClasses("bg-danger-hover border-transparent text-on-accent", focus),
    )
    .with({ variant: "destructive" }, () =>
      mergeClasses("bg-danger border-transparent text-on-accent", focus),
    )
    .with({ variant: "secondary", pressed: true }, () =>
      mergeClasses("bg-control-pressed border-transparent text-primary", focus),
    )
    .with({ variant: "secondary", hovered: true }, () =>
      mergeClasses("bg-control-hover border-transparent text-primary", focus),
    )
    .with({ variant: "secondary" }, () =>
      mergeClasses("bg-control border-transparent text-primary", focus),
    )
    .with({ pressed: true }, ({ variant: value }) =>
      mergeClasses(
        "bg-control-pressed text-secondary",
        passiveBorder(value),
        focus,
      ),
    )
    .with({ hovered: true }, ({ variant: value }) =>
      mergeClasses(
        "bg-control-hover text-secondary",
        passiveBorder(value),
        focus,
      ),
    )
    .with({ variant: P.union("outline", "ghost") }, ({ variant: value }) =>
      mergeClasses(
        "bg-transparent text-secondary",
        passiveBorder(value),
        focus,
      ),
    )
    .exhaustive();
}

function buttonSize(size: ButtonSize, grouped: boolean): string {
  return grouped
    ? componentsControlContentSize(size)
    : componentsControlSize(size);
}

function buttonSpinnerColor(variant: ButtonVariant): string {
  return match(variant)
    .with("default", "destructive", () => "text-on-accent")
    .with("secondary", () => "text-primary")
    .with("outline", "ghost", () => "text-secondary")
    .exhaustive();
}

export function Button(props: ButtonProps): JSX.Element {
  const local = props;
  const forwarded = omit(
    props,
    "variant",
    "size",
    "class",
    "style",
    "loading",
    "loadingLabel",
    "children",
  );
  const groupItem = useButtonGroupItem();
  const variant = () => local.variant ?? groupItem?.variant() ?? "default";
  const size = () => local.size ?? groupItem?.size() ?? "default";
  const visuallyDisabled = () =>
    local.disabled || groupItem?.disabled() || false;
  const disabled = () =>
    local.disabled || local.loading || groupItem?.disabled() || false;
  return (
    <HeadlessButton
      {...forwarded}
      disabled={disabled()}
      aria-busy={local.loading ?? false}
      unstyled
      class={(state) =>
        mergeClasses(
          "inline-flex flex-none overflow-hidden whitespace-nowrap items-center justify-center border font-medium",
          buttonColors(variant(), state, visuallyDisabled()),
          buttonSize(size(), groupItem !== undefined),
          groupItem && buttonGroupItemCorners(groupItem),
          groupItem && "border-transparent",
          local.class,
          componentsDisabledControlClass(visuallyDisabled()),
        )
      }
      style={(state) =>
        ({
          "border-width": 1,
          // Loading is inert but remains visually prominent; only an actual
          // disabled policy fades the control.
          ...(visuallyDisabled() ? {} : { opacity: 1 }),
          ...(typeof local.style === "function"
            ? local.style(state)
            : local.style),
        }) as WabouStyle
      }
    >
      <Show when={local.loading} fallback={local.children}>
        <Spinner
          label={local.loadingLabel ?? "Loading"}
          class={buttonSpinnerColor(variant())}
        />
        {local.loadingLabel ?? local.children}
      </Show>
    </HeadlessButton>
  );
}
