// Loading-state list component.
import { For, type JSX } from "solid-js";
import { useTheme } from "../contexts/ThemeContext";

export function LoadingList(): JSX.Element {
  const { palette } = useTheme();
  return (
    <div class="w-full">
      <For each={[1, 2, 3, 4, 5, 6]}>
        {(item) => (
          <div
            class="h-18 px-3 flex items-center gap-4 border-b"
            style={{ "border-color": palette().borderSoft }}
          >
            <span
              class="w-7 text-right font-mono text-xs"
              style={{ color: palette().textMuted }}
            >
              {item}
            </span>
            <div class="flex-1 flex flex-col gap-2">
              <i
                class="block w-70% h-2 rounded-sm"
                style={{ "background-color": palette().skeleton }}
              />
              <i
                class="block w-38% h-1.5 rounded-sm"
                style={{ "background-color": palette().skeleton }}
              />
            </div>
          </div>
        )}
      </For>
    </div>
  );
}
