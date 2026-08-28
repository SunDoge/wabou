import {
  Center,
  Image,
  type ImageResourceHandle,
  Text,
  View,
} from "../primitives";
import type { JSX } from "solid-js";
import { match } from "ts-pattern";
import { mergeClasses } from "@wabou/core/style";
export type AvatarSize = "sm" | "default" | "lg";
export interface AvatarProps {
  image?: ImageResourceHandle;
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
    <Center
      role="img"
      aria-label={props.alt ?? props.fallback}
      class={mergeClasses(
        "flex-none overflow-hidden rounded-full bg-control border border-subtle",
        size(),
        props.class,
      )}
    >
      {props.image ? (
        <Image
          aria-hidden="true"
          resource={props.image}
          class="w-full h-full"
        />
      ) : (
        <Text aria-hidden="true" class="font-medium text-secondary">
          {props.fallback}
        </Text>
      )}
    </Center>
  );
}
export function AvatarGroup(props: { children?: JSX.Element; class?: string }) {
  return (
    <View class={mergeClasses("flex items-center gap-1", props.class)}>
      {props.children}
    </View>
  );
}
export function AvatarGroupCount(props: {
  children?: JSX.Element;
  class?: string;
}) {
  return (
    <Center
      class={mergeClasses(
        "w-10 h-10 flex-none rounded-full bg-control border border-subtle",
        props.class,
      )}
    >
      <Text class="text-xs font-medium text-muted">{props.children}</Text>
    </Center>
  );
}
