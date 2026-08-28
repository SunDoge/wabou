import { For, type JSX } from "solid-js";
import { Text, View, type ViewProps } from "../primitives";
import { mergeClasses } from "@wabou/core/style";

export interface TimelineItem {
  id: string;
  title: string;
  description?: string;
  time?: string;
  status?: "complete" | "current" | "pending";
}
export interface TimelineProps extends Omit<ViewProps, "children"> {
  items: readonly TimelineItem[];
}

export function Timeline(props: TimelineProps): JSX.Element {
  return (
    <View
      role="group"
      aria-label={props["aria-label"] ?? "Timeline"}
      class={mergeClasses("min-w-0 flex flex-col", props.class)}
    >
      <For each={props.items}>
        {(item, index) => (
          <View class="min-w-0 flex flex-row gap-3">
            <View class="w-4 flex-none flex flex-col items-center">
              <View
                class={mergeClasses(
                  "w-3 h-3 flex-none rounded-full border",
                  item.status === "complete"
                    ? "border-accent bg-accent"
                    : item.status === "current"
                      ? "border-accent bg-surface"
                      : "border-subtle bg-control",
                )}
              />
              {index() < props.items.length - 1 && (
                <View class="w-px min-h-8 flex-1 bg-subtle" />
              )}
            </View>
            <View class="min-w-0 flex-1 pb-5">
              <View class="flex flex-row justify-between gap-3">
                <Text class="min-w-0 font-medium text-primary">
                  {item.title}
                </Text>
                {item.time && (
                  <Text class="flex-none text-xs text-muted">{item.time}</Text>
                )}
              </View>
              {item.description && (
                <Text class="mt-1 text-sm text-muted whitespace-normal">
                  {item.description}
                </Text>
              )}
            </View>
          </View>
        )}
      </For>
    </View>
  );
}
