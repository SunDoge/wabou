import { PropertyRow, Text, useComponentsTheme } from "@wabou/ui";
import type { JSX } from "solid-js";
import "virtual:wabou-stylesheet";

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
