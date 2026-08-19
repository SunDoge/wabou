import { Text, useComponentsTheme, View } from "@wabou/ui";
import type { JSX } from "solid-js";
import "virtual:wabou-stylesheet";

function PropertyRow(props: { name: string; value: string }) {
  const theme = useComponentsTheme();
  return (
    <View
      class={
        theme() === "dark"
          ? "h-10 px-3 flex items-center border-b border-slate-800"
          : "h-10 px-3 flex items-center border-b border-slate-200"
      }
    >
      <Text class="w-48 flex-none text-xs font-mono text-sky-400">
        {props.name}
      </Text>
      <Text
        class={
          theme() === "dark"
            ? "text-xs text-slate-400"
            : "text-xs text-slate-600"
        }
      >
        {props.value}
      </Text>
    </View>
  );
}

function ThemeText(props: {
  dark: string;
  light: string;
  children: JSX.Element;
}) {
  const theme = useComponentsTheme();
  return (
    <Text class={theme() === "dark" ? props.dark : props.light}>
      {props.children}
    </Text>
  );
}

export { PropertyRow, ThemeText };
