import { Shadow } from "@wabou/core/style";
import { JSX, ParentProps } from "solid-js";
//#region src/components/theme.d.ts
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
export declare const componentsThemeContract: Readonly<{
  controlHeight: Readonly<{
    sm: 28;
    default: 32;
    lg: 40;
    icon: 32;
  }>;
  controlPaddingX: Readonly<{
    sm: 8;
    default: 10;
    lg: 12;
    icon: 0;
  }>;
  iconSize: Readonly<{
    sm: 14;
    default: 16;
    lg: 18;
  }>;
  typography: Readonly<{
    xs: Readonly<{
      size: 12;
      lineHeight: 16;
    }>;
    sm: Readonly<{
      size: 14;
      lineHeight: 20;
    }>;
    md: Readonly<{
      size: 16;
      lineHeight: 24;
    }>;
    lg: Readonly<{
      size: 18;
      lineHeight: 28;
    }>;
    xl: Readonly<{
      size: 20;
      lineHeight: 28;
    }>;
  }>;
  controlRadius: 6;
  containerRadius: 8;
  containerPadding: 20;
  sectionGap: 16;
  /** Disabled controls remain readable while losing active-surface emphasis. */
  disabledOpacity: 0.6;
}>;
export declare function componentsControlSize(size: ComponentsControlSize): string;
/**
 * Native elevation recipes adapted from gpui-component. Both implementations
 * pass standard deviation directly to their renderer, so these values should
 * not use CSS's doubled blur radius. Floating surfaces also carry a subtle
 * foreground-colored ring: black in light mode, white in dark mode.
 */
export declare function componentsElevation(theme: ComponentsTheme, elevation: ComponentsElevation): Shadow[];
export type ComponentsProviderProps = ParentProps<{
  theme?: ComponentsTheme;
}>;
export declare function ComponentsProvider(props: ComponentsProviderProps): JSX.Element;
export declare function useComponentsTheme(): () => ComponentsTheme;
//#endregion
//# sourceMappingURL=theme.d.mts.map