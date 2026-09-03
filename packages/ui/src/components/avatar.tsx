import { mergeClasses } from "@wabou/core/style";
import { type JSX, omit } from "solid-js";
import { match } from "ts-pattern";
import {
  Center,
  Image,
  type ImageResourceHandle,
  Text,
  View,
  type ViewProps,
} from "../primitives";

export type AvatarSize = "sm" | "default" | "lg";

export interface AvatarProps extends Omit<ViewProps, "children"> {
  image?: ImageResourceHandle;
  /** Full accessible name and source for generated initials. */
  name?: string;
  /** Accessible label override when it should differ from `name`. */
  alt?: string;
  /** Explicit visual fallback. Defaults to initials derived from `name`. */
  fallback?: string;
  size?: AvatarSize;
}

/** Derive at most two stable initials without depending on Intl. */
export function avatarInitials(name: string): string {
  const words = name.trim().split(/\s+/).filter(Boolean);
  if (words.length === 0) return "";
  const initials =
    words.length === 1
      ? [...words[0]].slice(0, 2).join("")
      : words
          .slice(0, 2)
          .map((word) => [...word][0] ?? "")
          .join("");
  return initials.toUpperCase();
}

export function Avatar(props: AvatarProps): JSX.Element {
  const forwarded = omit(
    props,
    "image",
    "name",
    "alt",
    "fallback",
    "size",
    "class",
  );
  const size = () =>
    match(props.size ?? "default")
      .with("sm", () => "w-8 h-8 text-xs")
      .with("default", () => "w-10 h-10 text-sm")
      .with("lg", () => "w-12 h-12 text-base")
      .exhaustive();
  const fallback = () => props.fallback ?? avatarInitials(props.name ?? "");
  const label = () => props.alt ?? props.name ?? props.fallback ?? "Avatar";

  return (
    <Center
      {...forwarded}
      role="img"
      aria-label={label()}
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
          {fallback()}
        </Text>
      )}
    </Center>
  );
}

export type AvatarGroupProps = ViewProps;

export function AvatarGroup(props: AvatarGroupProps): JSX.Element {
  return (
    <View
      {...props}
      role={props.role ?? "group"}
      class={mergeClasses("flex items-center gap-1", props.class)}
    />
  );
}

export type AvatarGroupCountProps = ViewProps;

export function AvatarGroupCount(props: AvatarGroupCountProps): JSX.Element {
  const forwarded = omit(props, "class", "children");
  return (
    <Center
      {...forwarded}
      class={mergeClasses(
        "w-10 h-10 flex-none rounded-full bg-control border border-subtle",
        props.class,
      )}
    >
      <Text class="text-xs font-medium text-muted">{props.children}</Text>
    </Center>
  );
}
