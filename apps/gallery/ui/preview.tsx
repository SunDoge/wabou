import { Text, View } from "@wabou/primitives";
import type { JSX } from "solid-js";

export function Preview(props: { title?: string; children: JSX.Element }) {
  return (
    <View class="flex flex-col gap-3">
      {props.title && (
        <Text class="text-xs font-medium text-muted">{props.title}</Text>
      )}
      <View class="min-h-28 p-4 flex flex-wrap items-center justify-center gap-4 rounded-md border border-subtle">
        {props.children}
      </View>
    </View>
  );
}
