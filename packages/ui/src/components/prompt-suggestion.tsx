import { mergeClasses } from "@wabou/core/style";
import { type JSX, omit, Show } from "solid-js";
import { Icon, Text, View } from "../primitives";
import { Button, type ButtonProps } from "./button";
import { ResponsiveGrid, type ResponsiveGridProps } from "./layout";

export interface PromptSuggestionsProps
  extends Omit<
    ResponsiveGridProps,
    | "gap"
    | "initialColumns"
    | "maxColumns"
    | "minColumnWidth"
    | "balanceLastRow"
  > {
  itemCount?: number;
  minColumnWidth?: number;
  maxColumns?: 1 | 2 | 3;
  gap?: number;
}

/** Container-responsive starter actions for empty conversations and assistants. */
export function PromptSuggestions(props: PromptSuggestionsProps): JSX.Element {
  const rest = omit(props, "minColumnWidth", "maxColumns", "gap", "class");
  return (
    <ResponsiveGrid
      {...rest}
      minColumnWidth={props.minColumnWidth ?? 176}
      maxColumns={props.maxColumns ?? 3}
      initialColumns={1}
      itemCount={props.itemCount}
      balanceLastRow
      gap={props.gap ?? 8}
      class={mergeClasses("w-full", props.class)}
    />
  );
}

export interface PromptSuggestionProps
  extends Omit<ButtonProps, "children" | "class"> {
  title: string;
  description?: string;
  icon?: string;
  class?: string;
}

/** One explicit prompt choice; the application remains responsible for its payload. */
export function PromptSuggestion(props: PromptSuggestionProps): JSX.Element {
  const rest = omit(props, "title", "description", "icon", "class");
  return (
    <Button
      {...rest}
      aria-label={props["aria-label"] ?? props.title}
      variant={props.variant ?? "outline"}
      class={mergeClasses(
        "w-full h-auto min-h-14 min-w-0 items-start justify-start gap-2 border-subtle bg-transparent p-3 text-left shadow-none",
        props.class,
      )}
    >
      <Show when={props.icon}>
        {(source) => (
          <Icon source={source()} size={15} class="flex-none text-accent" />
        )}
      </Show>
      <View class="min-w-0 flex-1 flex flex-col gap-1">
        <Text class="whitespace-normal text-sm font-medium text-primary">
          {props.title}
        </Text>
        <Show when={props.description}>
          {(description) => (
            <Text class="whitespace-normal text-xs text-secondary">
              {description()}
            </Text>
          )}
        </Show>
      </View>
    </Button>
  );
}
