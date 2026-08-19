// Solid context owns application theme state.

import { View } from "@wabou/ui";
import {
  createContext,
  createSignal,
  type JSX,
  type Setter,
  useContext,
} from "solid-js";

export type Theme = "light" | "dark";

export interface ThemePalette {
  background: string;
  surface: string;
  raised: string;
  hover: string;
  border: string;
  borderSoft: string;
  text: string;
  textSecondary: string;
  textMuted: string;
  skeleton: string;
  accent: string;
  accentSoft: string;
  danger: string;
}

const palettes: Record<Theme, ThemePalette> = {
  light: {
    background: "#ffffff",
    surface: "#f6f8fa",
    raised: "#ffffff",
    hover: "#f0f2f5",
    border: "#d0d7de",
    borderSoft: "#e4e8ec",
    text: "#1f2328",
    textSecondary: "#59616b",
    textMuted: "#858e99",
    skeleton: "#e7eaee",
    accent: "#f05a18",
    accentSoft: "#fff0e8",
    danger: "#b42318",
  },
  dark: {
    background: "#15171a",
    surface: "#1d2024",
    raised: "#24282d",
    hover: "#2b3036",
    border: "#343a42",
    borderSoft: "#292e34",
    text: "#f1f3f5",
    textSecondary: "#c0c6ce",
    textMuted: "#8c959f",
    skeleton: "#30363d",
    accent: "#ff6a2a",
    accentSoft: "#3a271f",
    danger: "#ff8a80",
  },
};

interface ThemeContextValue {
  theme: () => Theme;
  palette: () => ThemePalette;
  setTheme: Setter<Theme>;
  toggleTheme: () => void;
}

const ThemeContext = createContext<ThemeContextValue>();

export function ThemeProvider(props: { children: JSX.Element }): JSX.Element {
  const [theme, setTheme] = createSignal<Theme>("light");
  const value: ThemeContextValue = {
    theme,
    palette: () => palettes[theme()],
    setTheme,
    toggleTheme: () =>
      setTheme((current) => (current === "light" ? "dark" : "light")),
  };

  return (
    <ThemeContext value={value}>
      <View
        class="w-full h-full"
        style={{
          "background-color": value.palette().background,
          color: value.palette().text,
        }}
      >
        {props.children}
      </View>
    </ThemeContext>
  );
}

export function useTheme(): ThemeContextValue {
  const context = useContext(ThemeContext);
  if (!context) throw new Error("useTheme must be used inside ThemeProvider");
  return context;
}
