import { type JSX, omit } from "solid-js";
import { match, P } from "ts-pattern";
import {
  type ButtonState,
  Button as HeadlessButton,
  type ButtonProps as HeadlessButtonProps,
  type WabouStyle,
} from "../primitives";
import { join } from "./class-names";
import { useButtonGroupOrientation } from "./button-group-context";

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
}

function buttonColors(variant: ButtonVariant, state: ButtonState): string {
  const focus = state.focusVisible ? "border-focus" : "";
  const passiveBorder = (value: ButtonVariant) =>
    match(value)
      .with("outline", () => "border-strong")
      .with(
        P.union("default", "secondary", "ghost", "destructive"),
        () => "border-transparent",
      )
      .exhaustive();

  return match({ variant, pressed: state.pressed, hovered: state.hovered })
    .with({ variant: "default", pressed: true }, () =>
      join("bg-accent-pressed border-transparent text-on-accent", focus),
    )
    .with({ variant: "default", hovered: true }, () =>
      join("bg-accent-hover border-transparent text-on-accent", focus),
    )
    .with({ variant: "default" }, () =>
      join("bg-accent border-transparent text-on-accent", focus),
    )
    .with({ variant: "destructive", pressed: true }, () =>
      join("bg-danger-pressed border-transparent text-on-accent", focus),
    )
    .with({ variant: "destructive", hovered: true }, () =>
      join("bg-danger-hover border-transparent text-on-accent", focus),
    )
    .with({ variant: "destructive" }, () =>
      join("bg-danger border-transparent text-on-accent", focus),
    )
    .with({ variant: "secondary", pressed: true }, () =>
      join("bg-control-pressed border-transparent text-primary", focus),
    )
    .with({ variant: "secondary", hovered: true }, () =>
      join("bg-control-hover border-transparent text-primary", focus),
    )
    .with({ variant: "secondary" }, () =>
      join("bg-control border-transparent text-primary", focus),
    )
    .with({ pressed: true }, ({ variant: value }) =>
      join("bg-control-pressed text-secondary", passiveBorder(value), focus),
    )
    .with({ hovered: true }, ({ variant: value }) =>
      join("bg-control-hover text-secondary", passiveBorder(value), focus),
    )
    .with({ variant: P.union("outline", "ghost") }, ({ variant: value }) =>
      join("bg-transparent text-secondary", passiveBorder(value), focus),
    )
    .exhaustive();
}

function buttonSize(size: ButtonSize): string {
  return match(size)
    .with("sm", () => "h-6 px-2 text-xs")
    .with("default", () => "h-8 px-3 text-sm")
    .with("lg", () => "h-10 px-4 text-base")
    .with("icon", () => "w-8 h-8 p-0 text-sm")
    .exhaustive();
}

export function Button(props: ButtonProps): JSX.Element {
  const local = props;
  const forwarded = omit(props, "variant", "size", "class", "style");
  const variant = () => local.variant ?? "default";
  const size = () => local.size ?? "default";
  const groupOrientation = useButtonGroupOrientation();
  return (
    <HeadlessButton
      {...forwarded}
      unstyled
      class={(state) =>
        join(
          "inline-flex flex-none whitespace-nowrap items-center justify-center gap-2 rounded-md border font-medium",
          buttonColors(variant(), state),
          buttonSize(size()),
          groupOrientation && "rounded-none border-transparent",
          local.class,
        )
      }
      style={(state) =>
        ({
          "border-width": 1,
          opacity: state.disabled ? 0.45 : 1,
          ...(typeof local.style === "function"
            ? local.style(state)
            : local.style),
        }) as WabouStyle
      }
    />
  );
}
