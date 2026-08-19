import { Card, CardContent, Text, View } from "@wabou/ui";
import type { JSX } from "solid-js";
import { match } from "ts-pattern";

export function StatCard(props: {
  label: string;
  value: string;
  detail: string;
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
    <Card class="relative flex-1 min-w-40 overflow-hidden rounded-xl shadow-xl">
      <View
        class="absolute left-0 top-0 bottom-0 w-1"
        classList={accent()}
      />
      <CardContent class="h-full p-4 pl-5 flex flex-col gap-2">
        <View class="flex flex-row items-center gap-2">
          <View class="w-2 h-2 rounded-full" classList={accent()} />
          <Text class="text-xs font-semibold text-muted">{props.label}</Text>
        </View>
        <Text class="text-xl font-semibold">{props.value}</Text>
        <View class="flex-1">{props.children}</View>
        <Text class="text-xs text-muted">{props.detail}</Text>
      </CardContent>
    </Card>
  );
}
