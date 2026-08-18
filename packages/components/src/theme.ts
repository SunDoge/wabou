import { type Shadow, shadow } from "@wabou/core/style";
import {
  createComponent,
  createContext,
  getOwner,
  type JSX,
  type ParentProps,
  useContext,
} from "solid-js";

export type ComponentsTheme = "light" | "dark";
export type ComponentsElevation = "raised" | "floating" | "modal";

/**
 * Native elevation recipes adapted from gpui-component. Wabou and GPUI both
 * pass standard deviation directly to their renderer, so these values should
 * not use CSS's doubled blur radius. Floating surfaces also carry a subtle
 * foreground-colored ring: black in light mode, white in dark mode.
 */
export function componentsElevation(
  theme: ComponentsTheme,
  elevation: ComponentsElevation,
): Shadow[] {
  if (elevation === "raised") {
    return [shadow({ offsetY: 1, stdDev: 2, color: 0x0000002e })];
  }
  if (elevation === "floating") {
    return [
      shadow({
        spread: 1,
        stdDev: 0,
        color: theme === "dark" ? 0xffffff1a : 0x0000001a,
      }),
      shadow({ offsetY: 4, stdDev: 3, spread: -1, color: 0x0000001a }),
      shadow({ offsetY: 2, stdDev: 2, spread: -2, color: 0x0000001a }),
    ];
  }
  return [
    shadow({ offsetY: 20, stdDev: 25, spread: -5, color: 0x0000001a }),
    shadow({ offsetY: 8, stdDev: 10, spread: -6, color: 0x0000001a }),
  ];
}

interface ComponentsThemeContext {
  theme: () => ComponentsTheme;
}

const defaultTheme: ComponentsThemeContext = {
  theme: () => "dark",
};
const ThemeContext = createContext<ComponentsThemeContext>(defaultTheme);

export type ComponentsProviderProps = ParentProps<{
  theme?: ComponentsTheme;
}>;

export function ComponentsProvider(
  props: ComponentsProviderProps,
): JSX.Element {
  return createComponent(ThemeContext, {
    value: { theme: () => props.theme ?? "dark" },
    get children() {
      return props.children;
    },
  });
}

export function useComponentsTheme(): () => ComponentsTheme {
  return (getOwner() ? useContext(ThemeContext) : defaultTheme).theme;
}
