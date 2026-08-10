import {
  type AnimationControls,
  animate,
  animateKeyframes,
} from "@wabou/animation";
import {
  Alert,
  Badge,
  Button,
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
  ComponentsProvider,
  Fps,
  Input,
  Progress,
  Separator,
  Switch,
  TextArea,
  useComponentsTheme,
} from "@wabou/components";
import { createWindow, useWindow } from "@wabou/core";
import {
  createHover,
  createScrollReset,
  Button as PrimitiveButton,
  ScrollArea,
  Text,
  translate2d,
  View,
} from "@wabou/primitives";
import {
  createMemoryHistory,
  MemoryRouter,
  Route,
  useLocation,
  useNavigate,
  useParams,
} from "@wabou/router";
import {
  type Handle,
  mount,
  px,
  rgba,
  shadow,
  number as styleNumber,
} from "@wabou/core";
import wabouUtilityManifest from "@wabou/vite/utility-manifest";
import {
  createSignal,
  For,
  type JSX,
  Match,
  onCleanup,
  onMount,
  Switch as ShowCase,
} from "solid-js";
import "virtual:wabou-stylesheet";

function PropertyRow(props: { name: string; value: string }) {
  const theme = useComponentsTheme();
  return (
    <View
      class={
        theme() === "dark"
          ? "h-10 px-3 flex items-center border-b border-slate-800"
          : "h-10 px-3 flex items-center border-b border-slate-200"
      }
    >
      <Text class="w-36 flex-none text-xs font-mono text-sky-400">
        {props.name}
      </Text>
      <Text
        class={
          theme() === "dark"
            ? "text-xs text-slate-400"
            : "text-xs text-slate-600"
        }
      >
        {props.value}
      </Text>
    </View>
  );
}

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
