import { useComponentsTheme } from "@wabou/components";
import { Text, View } from "@wabou/primitives";
import type { JSX } from "solid-js";

export function Preview(props: { title?: string; children: JSX.Element }) {
  const theme = useComponentsTheme();
  return (
    <View
      class={
        theme() === "dark"
          ? "flex flex-col overflow-hidden rounded-xl border border-slate-800 bg-slate-950"
          : "flex flex-col overflow-hidden rounded-xl border border-slate-200 bg-white"
      }
    >
      {props.title && (
        <View
          class={
            theme() === "dark"
              ? "h-10 px-4 flex items-center border-b border-slate-800 bg-slate-900"
              : "h-10 px-4 flex items-center border-b border-slate-200 bg-slate-50"
          }
        >
          <Text
            class={
              theme() === "dark"
                ? "text-xs font-medium text-slate-400"
                : "text-xs font-medium text-slate-500"
            }
          >
            {props.title}
          </Text>
        </View>
      )}
      <View
        class={
          theme() === "dark"
            ? "min-h-40 p-8 flex flex-wrap items-center justify-center gap-3 bg-slate-950"
            : "min-h-40 p-8 flex flex-wrap items-center justify-center gap-3 bg-white"
        }
      >
        {props.children}
      </View>
    </View>
  );
}
