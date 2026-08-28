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
export type ComponentsControlSize = "sm" | "default" | "lg" | "icon";

/**
 * Geometry contract for Wabou's default desktop theme.
 *
 * Components consume these recipes instead of independently choosing height,
 * padding, radius, and icon rhythm. The values intentionally favor desktop
 * density over touch-first sizing.
 */
export const componentsThemeContract = Object.freeze({
  controlHeight: Object.freeze({ sm: 28, default: 32, lg: 40, icon: 32 }),
  iconSize: Object.freeze({ sm: 14, default: 16, lg: 18 }),
  controlRadius: 8,
  containerRadius: 12,
  containerPadding: 20,
  sectionGap: 16,
});

export function componentsControlContentSize(
  size: ComponentsControlSize,
): string {
  switch (size) {
    case "sm":
      return "h-7 px-2 gap-1.5 text-xs";
    case "lg":
      return "h-10 px-4 gap-2.5 text-base";
    case "icon":
      return "w-8 h-8 p-0 gap-0 text-sm";
    default:
      return "h-8 px-3 gap-2 text-sm";
  }
}

export function componentsControlSize(size: ComponentsControlSize): string {
  return `${componentsControlContentSize(size)} rounded-lg`;
}

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
    return [
      shadow({ spread: 1, stdDev: 0, color: 0x0000000d }),
      shadow({ offsetY: 1, stdDev: 2, color: 0x0000001f }),
      shadow({ offsetY: 4, stdDev: 7, spread: -4, color: 0x00000014 }),
    ];
  }
  if (elevation === "floating") {
    return [
      shadow({
        spread: 1,
        stdDev: 0,
        color: theme === "dark" ? 0xffffff1f : 0x00000014,
      }),
      shadow({ offsetY: 8, stdDev: 12, spread: -5, color: 0x00000024 }),
      shadow({ offsetY: 2, stdDev: 3, spread: -2, color: 0x0000001f }),
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
  theme: () => "light",
};
const ThemeContext = createContext<ComponentsThemeContext>(defaultTheme);

export type ComponentsProviderProps = ParentProps<{
  theme?: ComponentsTheme;
}>;

export function ComponentsProvider(
  props: ComponentsProviderProps,
): JSX.Element {
  return createComponent(ThemeContext, {
    value: { theme: () => props.theme ?? "light" },
    get children() {
      return props.children;
    },
  });
}

export function useComponentsTheme(): () => ComponentsTheme {
  return (getOwner() ? useContext(ThemeContext) : defaultTheme).theme;
}
