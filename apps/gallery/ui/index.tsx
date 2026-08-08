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
import { type Handle, mount } from "@wabou/solid-renderer";
import { px, rgba, shadow, number as styleNumber } from "@wabou/style";
import wabouUtilityManifest from "@wabou/unocss-preset/manifest";
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

import { OverlayPage } from "./pages/overlay";
import { Preview } from "./preview";

type ComponentId =
  | "button"
  | "badge"
  | "card"
  | "input"
  | "switch"
  | "progress"
  | "fps"
  | "animation"
  | "platform"
  | "colors"
  | "shadows"
  | "utilities"
  | "scroll-area"
  | "overlay"
  | "alert"
  | "separator";

const groups: Array<{
  label: string;
  items: Array<{ id: ComponentId; name: string }>;
}> = [
  {
    label: "Actions",
    items: [
      { id: "button", name: "Button" },
      { id: "switch", name: "Switch" },
    ],
  },
  {
    label: "Forms",
    items: [{ id: "input", name: "Input" }],
  },
  {
    label: "Data display",
    items: [
      { id: "badge", name: "Badge" },
      { id: "card", name: "Card" },
      { id: "fps", name: "FPS" },
      { id: "progress", name: "Progress" },
    ],
  },
  {
    label: "Foundations",
    items: [
      { id: "colors", name: "Colors" },
      { id: "shadows", name: "Shadows" },
      { id: "overlay", name: "Overlay" },
    ],
  },
  {
    label: "Feedback",
    items: [
      { id: "alert", name: "Alert" },
      { id: "animation", name: "Animation" },
      { id: "separator", name: "Separator" },
    ],
  },
  {
    label: "Layout",
    items: [
      { id: "utilities", name: "Utilities" },
      { id: "scroll-area", name: "Scroll area" },
    ],
  },
  {
    label: "Platform",
    items: [{ id: "platform", name: "Native window" }],
  },
];

const descriptions: Record<ComponentId, string> = {
  button: "Displays a button or a component that looks like a button.",
  badge: "A compact label for statuses, categories and metadata.",
  card: "A flexible container for grouped content and actions.",
  input: "A native text input with consistent layout and visual treatment.",
  switch: "A control that lets users toggle a setting on or off.",
  progress: "Shows completion for a task or a long-running operation.",
  fps: "Measures native host frames and highlights performance regressions.",
  animation: "Pure JavaScript value animations rendered by the native host.",
  platform: "Native windows and Rust-powered custom widgets.",
  colors: "Every color token exported by the native Wabou utility theme.",
  shadows:
    "Vello-native blurred rounded rectangles with explicit Gaussian parameters.",
  utilities: "Tailwind-style static classes parsed by the native Rust preset.",
  "scroll-area": "A native scrolling viewport with intrinsic flex content.",
  overlay: "Explicit floating and modal planes shared by JavaScript portals.",
  alert: "Calls attention to information that needs user awareness.",
  separator: "Visually separates content in a list or layout.",
};

const history = createMemoryHistory();

import { AnimationPage } from "./pages/animation";
import {
  AlertPage,
  BadgePage,
  ButtonPage,
  CardPage,
  ChildWindowPage,
  FpsPage,
  InputPage,
  PlatformPage,
  ProgressPage,
  ScrollAreaPage,
  SeparatorPage,
  SwitchPage,
  UtilitiesPage,
} from "./pages/basics";
import { ColorsPage, ShadowsPage } from "./pages/foundations";

function App() {
  const window = useWindow();
  if (window.id !== 1) return <ChildWindowPage />;
  const [theme, setTheme] = createSignal<"light" | "dark">("dark");
  const dark = () => theme() === "dark";
  const params = useParams<{ component?: string }>();
  const location = useLocation();
  let contentViewport: Handle | undefined;
  createScrollReset({
    target: () => contentViewport,
    key: () => location.pathname,
  });
  const navigate = useNavigate();
  const selected = (): ComponentId =>
    groups.some((group) =>
      group.items.some((item) => item.id === params.component),
    )
      ? (params.component as ComponentId)
      : "button";
  const selectedName = () =>
    groups
      .flatMap((group) => group.items)
      .find((item) => item.id === selected())?.name ?? "Component";

  return (
    <ComponentsProvider theme={theme()}>
      <View
        class={
          dark()
            ? "w-full h-full flex overflow-hidden bg-slate-950 text-slate-100 font-sans"
            : "w-full h-full flex overflow-hidden bg-slate-50 text-slate-900 font-sans"
        }
      >
        <View
          class={
            dark()
              ? "w-60 h-full flex-none flex flex-col border-r border-slate-800 bg-slate-950"
              : "w-60 h-full flex-none flex flex-col border-r border-slate-200 bg-white"
          }
        >
          <View
            class={
              dark()
                ? "h-16 flex-none px-5 flex items-center gap-3 border-b border-slate-800"
                : "h-16 flex-none px-5 flex items-center gap-3 border-b border-slate-200"
            }
          >
            <View class="w-8 h-8 flex items-center justify-center rounded-lg bg-sky-500">
              <Text class="text-sm font-bold text-white">W</Text>
            </View>
            <View class="flex flex-col">
              <Text
                class={
                  dark()
                    ? "text-sm font-semibold text-white"
                    : "text-sm font-semibold text-slate-950"
                }
              >
                Wabou
              </Text>
              <Text
                class={
                  dark() ? "text-xs text-slate-500" : "text-xs text-slate-500"
                }
              >
                Components & platform
              </Text>
            </View>
          </View>
          <ScrollArea contentClass="px-3 py-4">
            <For each={groups}>
              {(group) => (
                <View class="flex-none flex flex-col gap-1 mb-5">
                  <Text
                    class={
                      dark()
                        ? "px-2 py-1 text-xs font-medium text-slate-600"
                        : "px-2 py-1 text-xs font-medium text-slate-400"
                    }
                  >
                    {group.label}
                  </Text>
                  <For each={group.items}>
                    {(item) => (
                      <PrimitiveButton
                        unstyled
                        selected={selected() === item.id}
                        class="w-full h-9 px-3 rounded-md text-sm"
                        style={(state) => ({
                          "justify-content": "flex-start",
                          "background-color":
                            selected() === item.id
                              ? dark()
                                ? "#1e293b"
                                : "#e0f2fe"
                              : state.hovered
                                ? dark()
                                  ? "#0f172a"
                                  : "#f1f5f9"
                                : "transparent",
                          color:
                            selected() === item.id
                              ? dark()
                                ? "#f8fafc"
                                : "#0369a1"
                              : dark()
                                ? "#94a3b8"
                                : "#475569",
                        })}
                        onClick={() => navigate(`/components/${item.id}`)}
                      >
                        {item.name}
                      </PrimitiveButton>
                    )}
                  </For>
                </View>
              )}
            </For>
          </ScrollArea>
          <View
            class={
              dark()
                ? "flex-none p-4 border-t border-slate-800"
                : "flex-none p-4 border-t border-slate-200"
            }
          >
            <Badge variant="outline">
              {groups.reduce((total, group) => total + group.items.length, 0)}{" "}
              showcases
            </Badge>
          </View>
        </View>

        <View class="flex-1 min-w-0 h-full flex flex-col">
          <View
            class={
              dark()
                ? "h-16 flex-none px-7 flex items-center justify-between border-b border-slate-800 bg-slate-950"
                : "h-16 flex-none px-7 flex items-center justify-between border-b border-slate-200 bg-white"
            }
          >
            <View class="flex items-center gap-3">
              <View class="flex items-center gap-1">
                <PrimitiveButton
                  unstyled
                  class={
                    dark()
                      ? "w-8 h-8 justify-center rounded-md text-slate-400"
                      : "w-8 h-8 justify-center rounded-md text-slate-600"
                  }
                  style={(state) => ({
                    "background-color": state.hovered
                      ? dark()
                        ? "#1e293b"
                        : "#f1f5f9"
                      : "transparent",
                  })}
                  onClick={history.back}
                >
                  ‹
                </PrimitiveButton>
                <PrimitiveButton
                  unstyled
                  class={
                    dark()
                      ? "w-8 h-8 justify-center rounded-md text-slate-400"
                      : "w-8 h-8 justify-center rounded-md text-slate-600"
                  }
                  style={(state) => ({
                    "background-color": state.hovered
                      ? dark()
                        ? "#1e293b"
                        : "#f1f5f9"
                      : "transparent",
                  })}
                  onClick={history.forward}
                >
                  ›
                </PrimitiveButton>
              </View>
              <Text
                class={
                  dark() ? "text-sm text-slate-500" : "text-sm text-slate-500"
                }
              >
                Components / {selectedName()}
              </Text>
            </View>
            <View class="flex items-center gap-2">
              <Fps />
              <Button
                size="sm"
                variant="ghost"
                onClick={() => setTheme(dark() ? "light" : "dark")}
              >
                {dark() ? "Light" : "Dark"}
              </Button>
              <Badge variant="success">Native</Badge>
              <Badge variant="outline">UnoCSS</Badge>
            </View>
          </View>
          <View
            ref={(node) => (contentViewport = node)}
            class="flex-1 min-h-0 overflow-y-auto"
          >
            <View class="w-full max-w-4xl mx-auto px-8 py-10 flex flex-col gap-7">
              <View class="flex flex-col gap-2">
                <Text
                  class={
                    dark()
                      ? "text-3xl font-bold text-white"
                      : "text-3xl font-bold text-slate-950"
                  }
                >
                  {selectedName()}
                </Text>
                <Text
                  class={
                    dark()
                      ? "text-base text-slate-400"
                      : "text-base text-slate-600"
                  }
                >
                  {descriptions[selected()]}
                </Text>
              </View>
              <Separator />
              <ShowCase>
                <Match when={selected() === "button"}>
                  <ButtonPage />
                </Match>
                <Match when={selected() === "badge"}>
                  <BadgePage />
                </Match>
                <Match when={selected() === "card"}>
                  <CardPage />
                </Match>
                <Match when={selected() === "input"}>
                  <InputPage />
                </Match>
                <Match when={selected() === "switch"}>
                  <SwitchPage />
                </Match>
                <Match when={selected() === "progress"}>
                  <ProgressPage />
                </Match>
                <Match when={selected() === "fps"}>
                  <FpsPage />
                </Match>
                <Match when={selected() === "scroll-area"}>
                  <ScrollAreaPage />
                </Match>
                <Match when={selected() === "overlay"}>
                  <OverlayPage />
                </Match>
                <Match when={selected() === "utilities"}>
                  <UtilitiesPage />
                </Match>
                <Match when={selected() === "colors"}>
                  <ColorsPage />
                </Match>
                <Match when={selected() === "shadows"}>
                  <ShadowsPage />
                </Match>
                <Match when={selected() === "alert"}>
                  <AlertPage />
                </Match>
                <Match when={selected() === "animation"}>
                  <AnimationPage />
                </Match>
                <Match when={selected() === "platform"}>
                  <PlatformPage />
                </Match>
                <Match when={selected() === "separator"}>
                  <SeparatorPage />
                </Match>
              </ShowCase>
            </View>
          </View>
        </View>
      </View>
    </ComponentsProvider>
  );
}

mount(() => (
  <MemoryRouter history={history}>
    <Route path={["/", "/components/:component"]} component={App} />
  </MemoryRouter>
));
