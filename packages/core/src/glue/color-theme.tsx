import {
  createComponent,
  createContext,
  createEffect,
  createSignal,
  type JSX,
  useContext,
} from "solid-js";

export interface ColorThemeController {
  current(): string | undefined;
  set(name: string): void;
}

const [current, setCurrent] = createSignal<string>();

export const colorTheme: ColorThemeController = {
  current,
  set(name) {
    if (!name) throw new Error("Wabou color theme name cannot be empty");
    globalThis.__wabou_set_color_theme(name);
    setCurrent(name);
  },
};

const ColorThemeContext = createContext<ColorThemeController>(colorTheme);

/**
 * Explicitly selects the compiled color palette for the current native window.
 * This is window-level; nested theme scopes are intentionally not supported.
 */
export function ColorThemeProvider(props: {
  theme: string;
  children: JSX.Element;
}): JSX.Element {
  createEffect(() => colorTheme.set(props.theme));
  return createComponent(ColorThemeContext.Provider, {
    value: colorTheme,
    get children() {
      return props.children;
    },
  });
}

export function useColorTheme(): ColorThemeController {
  return useContext(ColorThemeContext);
}
