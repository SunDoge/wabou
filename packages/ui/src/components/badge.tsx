import { type JSX, omit } from "solid-js";
import { match } from "ts-pattern";
import { Text, type TextProps } from "../primitives";
import { join } from "./class-names";

export type BadgeVariant =
  | "default"
  | "secondary"
  | "outline"
  | "ghost"
  | "link"
  | "success"
  | "destructive";

export interface BadgeProps extends Omit<TextProps, "class"> {
  variant?: BadgeVariant;
  /** Typography weight selected without competing utility declarations. */
  weight?: "normal" | "medium";
  class?: string;
}

export function badgeClass(
  variant: BadgeVariant = "default",
  weight: NonNullable<BadgeProps["weight"]> = "medium",
  className?: string,
): string {
  const colors = match(variant)
    .with("default", () => "bg-accent border-accent text-on-accent")
    .with("secondary", () => "bg-control border-subtle text-primary")
    .with("outline", () => "bg-transparent border-strong text-secondary")
    .with("ghost", () => "bg-transparent border-transparent text-secondary")
    .with("link", () => "bg-transparent border-transparent text-accent")
    .with(
      "success",
      () => "bg-success-surface border-success-primary text-success-primary",
    )
    .with(
      "destructive",
      () => "bg-danger-surface border-danger text-danger-primary",
    )
    .exhaustive();
  return join(
    "w-fit flex-none overflow-hidden whitespace-nowrap rounded-full border px-2 py-0.5 text-xs",
    weight === "normal" ? "font-normal" : "font-medium",
    colors,
    className,
  );
}

/** Compact status text with shadcn-compatible visual variants. */
export function Badge(props: BadgeProps): JSX.Element {
  const rest = omit(props, "variant", "weight", "class", "children");
  return (
    <Text
      {...rest}
      class={badgeClass(props.variant, props.weight, props.class)}
    >
      {props.children}
    </Text>
  );
}
