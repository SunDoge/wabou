import { Center, Icon, type IconProps, type ViewProps } from "../primitives";
import { omit } from "solid-js";
import { match } from "ts-pattern";
import { mergeClasses } from "@wabou/core/style";
import { componentToneClass, type ComponentTone } from "./tone";

export type IconFrameSize = "sm" | "default" | "lg" | "xl";
export type IconFrameVariant = "plain" | "soft" | "solid";

export interface IconFrameProps
  extends Omit<ViewProps, "children">,
    Pick<IconProps, "source" | "fill" | "label"> {
  /** Size of the square visual container. */
  size?: IconFrameSize;
  /** Explicit icon size. Defaults are tuned for each container size. */
  iconSize?: number | string;
  /** Color intent, independent from the frame's visual emphasis. */
  tone?: ComponentTone;
  variant?: IconFrameVariant;
  iconClass?: string;
}

const frameSizeClass = (size: IconFrameSize) =>
  match(size)
    .with("sm", () => "w-8 h-8 rounded-md")
    .with("default", () => "w-10 h-10 rounded-lg")
    .with("lg", () => "w-12 h-12 rounded-lg")
    .with("xl", () => "w-14 h-14 rounded-2xl")
    .exhaustive();

const defaultIconSize = (size: IconFrameSize) =>
  match(size)
    .with("sm", () => 16)
    .with("default", () => 20)
    .with("lg", () => 23)
    .with("xl", () => 26)
    .exhaustive();

const variantClass = (variant: IconFrameVariant, tone: ComponentTone) =>
  match(variant)
    .with("plain", () =>
      mergeClasses(
        "border-transparent bg-transparent",
        componentToneClass(tone, "text"),
      ),
    )
    .with("soft", () => componentToneClass(tone, "surface"))
    .with("solid", () => componentToneClass(tone, "solid"))
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
    "tone",
    "variant",
    "iconClass",
    "class",
  );
  const size = () => props.size ?? "default";
  return (
    <Center
      {...rest}
      role="presentation"
      class={mergeClasses(
        "flex-none",
        frameSizeClass(size()),
        variantClass(props.variant ?? "plain", props.tone ?? "neutral"),
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
