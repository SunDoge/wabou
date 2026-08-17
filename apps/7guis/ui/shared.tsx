import { Badge, Card } from "@wabou/components";
import { Text, View } from "@wabou/primitives";
import type { JSX } from "solid-js";

export function TaskPage(props: {
  number: number;
  title: string;
  summary: string;
  children: JSX.Element;
}) {
  return (
    <View class="w-full max-w-4xl mx-auto flex flex-col gap-6">
      <View class="flex items-start justify-between gap-4">
        <View class="min-w-0 flex flex-col gap-2">
          <View class="flex items-center gap-3">
            <Badge variant="outline">Task {props.number}</Badge>
            <Text class="text-2xl font-semibold text-primary">
              {props.title}
            </Text>
          </View>
          <Text class="max-w-2xl whitespace-normal text-sm text-muted">
            {props.summary}
          </Text>
        </View>
        <Badge variant="success">Interactive</Badge>
      </View>
      <Card class="w-full min-h-64 p-6">{props.children}</Card>
    </View>
  );
}

export function FieldLabel(props: { children: JSX.Element }) {
  return (
    <Text class="text-sm font-medium text-secondary">{props.children}</Text>
  );
}

export interface LocalPointerEvent {
  offsetX: number;
  offsetY: number;
  button: number;
  preventDefault(): void;
  stopPropagation(): void;
}
