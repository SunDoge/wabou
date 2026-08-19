// Hacker News application shell.

import { createFps, Text, View } from "@wabou/ui";
import type { JSX } from "solid-js";
import { Sidebar } from "./components/Sidebar";
import { useTheme } from "./contexts/ThemeContext";

export function AppShell(props: { children?: JSX.Element }): JSX.Element {
  const { palette } = useTheme();
  const fps = createFps();
  return (
    <View
      class="w-full h-full overflow-hidden flex font-sans select-none"
      style={{
        "background-color": palette().background,
        color: palette().text,
      }}
    >
      <Sidebar />

      <View
        class="flex-1 min-w-0 min-h-0 overflow-hidden"
        style={{ "background-color": palette().background }}
      >
        {props.children}
      </View>

      {/* FPS overlay — exercises position:absolute + inset (taffy). */}
      <View
        class="absolute top-0 right-0 m-2 px-2 py-1 text-xs font-mono rounded border"
        style={{
          "background-color": palette().raised,
          color: palette().textMuted,
          "border-color": palette().border,
        }}
      >
        <Text>{fps()} fps</Text>
      </View>
    </View>
  );
}
