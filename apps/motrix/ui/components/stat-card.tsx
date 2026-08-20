import { Card, CardContent, Text, View } from "@wabou/ui";
import { type JSX, Show } from "solid-js";
import { match } from "ts-pattern";

export function StatCard(props: {
  label: string;
  value: string;
  detail?: string;
  footer?: JSX.Element;
  description?: string;
  dense?: boolean;
  roomy?: boolean;
  accent?: "blue" | "purple" | "green" | "neutral";
  children?: JSX.Element;
}) {
  const accent = () =>
    match(props.accent ?? "neutral")
      .with("blue", () => ({ "bg-chart-download": true }))
      .with("purple", () => ({ "bg-chart-upload": true }))
      .with("green", () => ({ "bg-success-primary": true }))
      .with("neutral", () => ({ "bg-muted": true }))
      .exhaustive();
  return (
    <Card
      role="group"
      aria-label={`${props.label} statistic`}
      class={
        props.dense
          ? "relative h-36 flex-1 min-w-40 overflow-hidden rounded-2xl shadow-md"
          : props.roomy
            ? "relative h-56 flex-1 min-w-40 overflow-hidden rounded-2xl shadow-md"
            : "relative h-40 flex-1 min-w-40 overflow-hidden rounded-2xl shadow-md"
      }
    >
      <View class="absolute left-0 right-0 bottom-0 h-1" classList={accent()} />
      <CardContent
        class={
          props.dense
            ? "h-full px-4 py-3 pl-5 flex flex-col gap-1"
            : props.roomy
              ? "h-full p-6 flex flex-col gap-3"
              : "h-full p-4 flex flex-col gap-2"
        }
      >
        <View class="flex flex-row items-center gap-2">
          <View class="w-2 h-2 rounded-full" classList={accent()} />
          <Text class="text-xs font-semibold text-muted">{props.label}</Text>
        </View>
        <Text
          class={
            props.roomy ? "text-3xl font-semibold" : "text-xl font-semibold"
          }
        >
          {props.value}
        </Text>
        <View class="min-w-0 min-h-0 flex-1">
          <Show when={props.description} fallback={props.children}>
            {(description) => (
              <Text class="whitespace-normal text-xs text-secondary">
                {description()}
              </Text>
            )}
          </Show>
        </View>
        <Show
          when={props.footer}
          fallback={
            <Text class="truncate text-xs text-muted">{props.detail}</Text>
          }
        >
          {(footer) => footer()}
        </Show>
      </CardContent>
    </Card>
  );
}
