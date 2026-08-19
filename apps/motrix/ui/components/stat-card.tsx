import { Card, CardContent, Text, View } from "@wabou/ui";
import type { JSX } from "solid-js";

export function StatCard(props: {
  label: string;
  value: string;
  detail: string;
  children?: JSX.Element;
}) {
  return (
    <Card class="flex-1 min-w-48 rounded-2xl shadow-xl">
      <CardContent class="h-full p-6 flex flex-col gap-3">
        <Text class="text-xs font-medium text-muted">{props.label}</Text>
        <Text class="text-2xl font-semibold">{props.value}</Text>
        <View class="flex-1">{props.children}</View>
        <Text class="text-xs text-muted">{props.detail}</Text>
      </CardContent>
    </Card>
  );
}
