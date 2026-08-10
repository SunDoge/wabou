import { Image, Text, View } from "@wabou/primitives";
import type { JSX } from "solid-js";
import { match } from "ts-pattern";
const join = (...values: Array<string | undefined | false>) =>
  values.filter(Boolean).join(" ");
export type AvatarSize = "sm" | "default" | "lg";
export interface AvatarProps {
  src?: string;
  alt?: string;
  fallback: string;
  size?: AvatarSize;
  class?: string;
}
export function Avatar(props: AvatarProps) {
  const size = () =>
    match(props.size ?? "default")
      .with("sm", () => "w-8 h-8 text-xs")
      .with("default", () => "w-10 h-10 text-sm")
      .with("lg", () => "w-12 h-12 text-base")
      .exhaustive();
  return (
    <View
      role="img"
      aria-label={props.alt ?? props.fallback}
      class={join(
        "flex-none overflow-hidden items-center justify-center rounded-full bg-control border border-subtle",
        size(),
        props.class,
      )}
    >
      {props.src ? (
        <Image src={props.src} class="w-full h-full" />
      ) : (
        <Text class="font-medium text-secondary">{props.fallback}</Text>
      )}
    </View>
  );
}
export function AvatarGroup(props: { children?: JSX.Element; class?: string }) {
  return (
    <View class={join("flex items-center gap-1", props.class)}>
      {props.children}
    </View>
  );
}
export function AvatarGroupCount(props: {
  children?: JSX.Element;
  class?: string;
}) {
  return (
    <View
      class={join(
        "w-10 h-10 flex-none items-center justify-center rounded-full bg-control border border-subtle",
        props.class,
      )}
    >
      <Text class="text-xs font-medium text-muted">{props.children}</Text>
    </View>
  );
}
