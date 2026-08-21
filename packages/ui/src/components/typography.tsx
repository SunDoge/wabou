import type { JSX } from "solid-js";
import { Text, type TextProps, View, type ViewProps } from "../primitives";
import { join } from "./class-names";

type TypographyTextProps = Omit<TextProps, "class"> & { class?: string };

function styledText(
  props: TypographyTextProps,
  className: string,
): JSX.Element {
  return <Text {...props} class={join(className, props.class)} />;
}

export const TypographyH1 = (props: TypographyTextProps) =>
  styledText(props, "text-4xl font-bold text-primary whitespace-normal");
export const TypographyH2 = (props: TypographyTextProps) =>
  styledText(props, "text-3xl font-semibold text-primary whitespace-normal");
export const TypographyH3 = (props: TypographyTextProps) =>
  styledText(props, "text-2xl font-semibold text-primary whitespace-normal");
export const TypographyH4 = (props: TypographyTextProps) =>
  styledText(props, "text-xl font-semibold text-primary whitespace-normal");
export const TypographyP = (props: TypographyTextProps) =>
  styledText(
    props,
    "text-base leading-relaxed text-secondary whitespace-normal",
  );
export const TypographyLead = (props: TypographyTextProps) =>
  styledText(props, "text-xl leading-relaxed text-muted whitespace-normal");
export const TypographyLarge = (props: TypographyTextProps) =>
  styledText(props, "text-lg font-semibold text-primary whitespace-normal");
export const TypographySmall = (props: TypographyTextProps) =>
  styledText(props, "text-sm font-medium text-secondary whitespace-normal");
export const TypographyMuted = (props: TypographyTextProps) =>
  styledText(props, "text-sm text-muted whitespace-normal");
export const TypographyInlineCode = (props: TypographyTextProps) =>
  styledText(
    props,
    "px-1.5 py-0.5 rounded bg-control font-mono text-sm font-medium text-primary",
  );

export function TypographyBlockquote(props: TypographyTextProps): JSX.Element {
  return (
    <View class="flex flex-row items-stretch gap-4">
      <View aria-hidden="true" class="w-1 flex-none rounded-full bg-strong" />
      {styledText(
        props,
        "min-w-0 flex-1 text-base leading-relaxed text-secondary whitespace-normal",
      )}
    </View>
  );
}

export function TypographyList(props: ViewProps): JSX.Element {
  return <View {...props} class={join("flex flex-col gap-2", props.class)} />;
}

export function TypographyListItem(props: TypographyTextProps): JSX.Element {
  return (
    <View class="min-w-0 flex flex-row items-start gap-2">
      <Text aria-hidden="true" class="flex-none text-secondary">
        •
      </Text>
      {styledText(
        props,
        "min-w-0 flex-1 text-base text-secondary whitespace-normal",
      )}
    </View>
  );
}
