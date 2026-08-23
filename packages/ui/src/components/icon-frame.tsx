import {
  Center,
  Icon,
  type IconProps,
  type ViewProps,
} from "../primitives";
import { omit } from "solid-js";
import { match } from "ts-pattern";
import { join } from "./class-names";

export type IconFrameSize = "sm" | "default" | "lg";
export type IconFrameVariant = "plain" | "muted" | "selected" | "solid";

export interface IconFrameProps
  extends Omit<ViewProps, "children">,
    Pick<IconProps, "source" | "fill" | "label"> {
  /** Size of the square visual container. */
  size?: IconFrameSize;
  /** Explicit icon size. Defaults are tuned for each container size. */
  iconSize?: number | string;
  variant?: IconFrameVariant;
  iconClass?: string;
}

const frameSizeClass = (size: IconFrameSize) =>
  match(size)
    .with("sm", () => "w-8 h-8 rounded-md")
    .with("default", () => "w-10 h-10 rounded-lg")
    .with("lg", () => "w-12 h-12 rounded-lg")
    .exhaustive();

const defaultIconSize = (size: IconFrameSize) =>
  match(size)
    .with("sm", () => 16)
    .with("default", () => 20)
    .with("lg", () => 23)
    .exhaustive();

const variantClass = (variant: IconFrameVariant) =>
  match(variant)
    .with("plain", () => "bg-transparent")
    .with("muted", () => "bg-control text-secondary")
    .with("selected", () => "bg-selected text-accent")
    .with("solid", () => "bg-accent text-on-accent")
    .exhaustive();

/**
 * A square icon surface whose geometry is correct by construction.
 *
 * Use this for standalone icon tiles. Buttons and menu items already own their
 * icon alignment and should continue to use `Icon` directly.
 */
export function IconFrame(props: IconFrameProps) {
  const rest = omit(
    props,
    "source",
    "fill",
    "label",
    "size",
    "iconSize",
    "variant",
    "iconClass",
    "class",
  );
  const size = () => props.size ?? "default";
  return (
    <Center
      {...rest}
      role="presentation"
      class={join(
        "flex-none",
        frameSizeClass(size()),
        variantClass(props.variant ?? "plain"),
        props.class,
      )}
    >
      <Icon
        source={props.source}
        size={props.iconSize ?? defaultIconSize(size())}
        fill={props.fill}
        label={props.label}
        class={props.iconClass}
      />
    </Center>
  );
}
