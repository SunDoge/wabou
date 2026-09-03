// Loading-state list component.
import { Text, View } from "@wabou/ui";
import { For as ForValue, type JSX } from "solid-js";
import { useTheme } from "../contexts/ThemeContext";

export function LoadingList(): JSX.Element {
  const { palette } = useTheme();
  return (
    <View class="w-full">
      <ForValue each={[1, 2, 3, 4, 5, 6]}>
        {(item) => (
          <View
            class="h-18 px-3 flex items-center gap-4 border-b"
            style={{ "border-color": palette().borderSoft }}
          >
            <Text
              class="w-7 text-right font-mono text-xs"
              style={{ color: palette().textMuted }}
            >
              {item}
            </Text>
            <View class="flex-1 flex flex-col gap-2">
              <View
                class="block w-70% h-2 rounded-sm"
                style={{ "background-color": palette().skeleton }}
              />
              <View
                class="block w-38% h-1.5 rounded-sm"
                style={{ "background-color": palette().skeleton }}
              />
            </View>
          </View>
        )}
      </ForValue>
    </View>
  );
}
