import {
  createComponent,
  createContext,
  getOwner,
  type JSX,
  type ParentProps,
  useContext,
} from "solid-js";

export type ComponentsTheme = "light" | "dark";

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
