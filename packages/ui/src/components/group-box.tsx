import { mergeClasses } from "@wabou/core/style";
import { type JSX, omit, Show } from "solid-js";
import { match } from "ts-pattern";
import { Text, View, type ViewProps } from "../primitives";

export type GroupBoxVariant = "normal" | "fill" | "outline";

export interface GroupBoxProps extends Omit<ViewProps, "children" | "class"> {
  title?: JSX.Element;
  description?: JSX.Element;
  children?: JSX.Element;
  variant?: GroupBoxVariant;
  class?: string;
  headerClass?: string;
  contentClass?: string;
}

export function groupBoxContentClass(
  variant: GroupBoxVariant = "normal",
  className?: string,
): string {
  return mergeClasses(
    "w-full min-w-0 flex flex-col gap-4 rounded-lg",
    match(variant)
      .with("normal", () => "bg-transparent")
      .with("fill", () => "p-4 bg-control")
      .with("outline", () => "p-4 border border-subtle bg-transparent")
      .exhaustive(),
    className,
  );
}

/** A lightweight titled surface for related controls and settings rows. */
export function GroupBox(props: GroupBoxProps): JSX.Element {
  const variant = () => props.variant ?? "normal";
  const label = () =>
    props["aria-label"] ??
    (typeof props.title === "string" ? props.title : undefined);
  const rest = omit(
    props,
    "title",
    "description",
    "children",
    "variant",
    "class",
    "headerClass",
    "contentClass",
  );
  return (
    <View
      {...rest}
      role={props.role ?? "group"}
      aria-label={label()}
      class={mergeClasses(
        "w-full min-w-0 flex flex-col",
        variant() === "normal" ? "gap-4" : "gap-3",
        props.class,
      )}
    >
      <Show
        when={
          (props.title !== undefined && props.title !== null) ||
          (props.description !== undefined && props.description !== null)
        }
      >
        <View
          class={mergeClasses(
            "w-full min-w-0 flex flex-col gap-1",
            props.headerClass,
          )}
        >
          <Show when={props.title !== undefined && props.title !== null}>
            <Text
              role="heading"
              class="min-w-0 text-sm font-semibold text-primary"
            >
              {props.title}
            </Text>
          </Show>
          <Show
            when={props.description !== undefined && props.description !== null}
          >
            <Text class="w-full min-w-0 whitespace-normal text-sm text-muted">
              {props.description}
            </Text>
          </Show>
        </View>
      </Show>
      <View class={groupBoxContentClass(variant(), props.contentClass)}>
        {props.children}
      </View>
    </View>
  );
}
