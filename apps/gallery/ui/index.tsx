import {
  createMemoryHistory,
  MemoryRouter,
  Route,
  useLocation,
  useNavigate,
  useParams,
} from "@solidjs/router";
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
import { type Handle, mount, Portal } from "@wabou/solid-renderer";
import { px, rgba, shadow, number as styleNumber } from "@wabou/style";
import wabouUtilityManifest from "@wabou/unocss-preset/manifest";
import {
  createSignal,
  For,
  type JSX,
  Match,
  onCleanup,
  onMount,
  Show,
  Switch as ShowCase,
} from "solid-js";
import "virtual:wabou-stylesheet";

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

function Preview(props: { title?: string; children: JSX.Element }) {
  const theme = useComponentsTheme();
  return (
    <View
      class={
        theme() === "dark"
          ? "flex flex-col overflow-hidden rounded-xl border border-slate-800 bg-slate-950"
          : "flex flex-col overflow-hidden rounded-xl border border-slate-200 bg-white"
      }
    >
      {props.title && (
        <View
          class={
            theme() === "dark"
              ? "h-10 px-4 flex items-center border-b border-slate-800 bg-slate-900"
              : "h-10 px-4 flex items-center border-b border-slate-200 bg-slate-50"
          }
        >
          <Text
            class={
              theme() === "dark"
                ? "text-xs font-medium text-slate-400"
                : "text-xs font-medium text-slate-500"
            }
          >
            {props.title}
          </Text>
        </View>
      )}
      <View
        class={
          theme() === "dark"
            ? "min-h-40 p-8 flex flex-wrap items-center justify-center gap-3 bg-slate-950"
            : "min-h-40 p-8 flex flex-wrap items-center justify-center gap-3 bg-white"
        }
      >
        {props.children}
      </View>
    </View>
  );
}

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

function ButtonPage() {
  return (
    <View class="flex flex-col gap-5">
      <Preview title="Variants">
        <Button>Default</Button>
        <Button variant="secondary">Secondary</Button>
        <Button variant="outline">Outline</Button>
        <Button variant="ghost">Ghost</Button>
        <Button variant="destructive">Destructive</Button>
      </Preview>
      <Preview title="Sizes">
        <Button size="sm">Small</Button>
        <Button>Default</Button>
        <Button size="lg">Large button</Button>
        <Button size="icon">+</Button>
      </Preview>
      <Preview title="States">
        <Button disabled>Disabled</Button>
        <Button variant="outline">Keyboard focusable</Button>
      </Preview>
      <PropertyRow
        name="variant"
        value="default | secondary | outline | ghost | destructive"
      />
      <PropertyRow name="size" value="sm | default | lg | icon" />
    </View>
  );
}

function BadgePage() {
  return (
    <View class="flex flex-col gap-5">
      <Preview title="Variants">
        <Badge>Default</Badge>
        <Badge variant="secondary">Secondary</Badge>
        <Badge variant="outline">Outline</Badge>
        <Badge variant="success">Ready</Badge>
        <Badge variant="destructive">Failed</Badge>
      </Preview>
      <Preview title="In context">
        <View class="flex items-center gap-3">
          <ThemeText
            dark="text-sm text-slate-200"
            light="text-sm text-slate-700"
          >
            Production deployment
          </ThemeText>
          <Badge variant="success">Healthy</Badge>
          <Badge variant="outline">v0.1.0</Badge>
        </View>
      </Preview>
      <PropertyRow
        name="variant"
        value="default | secondary | outline | success | destructive"
      />
    </View>
  );
}

function CardPage() {
  return (
    <Preview title="Example">
      <Card class="w-96">
        <CardHeader>
          <CardTitle>Create project</CardTitle>
          <CardDescription>
            Deploy a new Wabou application from a template.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Input placeholder="Project name" />
        </CardContent>
        <CardFooter>
          <Button>Create project</Button>
          <Button variant="ghost">Cancel</Button>
        </CardFooter>
      </Card>
    </Preview>
  );
}

function InputPage() {
  const [value, setValue] = createSignal("");
  return (
    <View class="flex flex-col gap-5">
      <Preview title="Default">
        <View class="w-96 flex flex-col gap-2">
          <ThemeText
            dark="text-sm font-medium text-slate-200"
            light="text-sm font-medium text-slate-800"
          >
            Workspace name
          </ThemeText>
          <Input
            placeholder="my-terminal"
            value={value()}
            onInput={(event) => setValue(event.currentTarget.value)}
          />
          <ThemeText
            dark="text-xs text-slate-500"
            light="text-xs text-slate-500"
          >
            Value: {value() || "—"}
          </ThemeText>
        </View>
      </Preview>
      <Preview title="States">
        <View class="w-96 flex flex-col gap-3">
          <Input value="Editable value" />
          <Input disabled value="Disabled value" />
        </View>
      </Preview>
    </View>
  );
}

function SwitchPage() {
  const [enabled, setEnabled] = createSignal(true);
  return (
    <Preview title="Settings">
      <Card class="w-96">
        <CardContent>
          <View class="flex items-center justify-between gap-4">
            <View class="flex flex-col gap-1">
              <ThemeText
                dark="text-sm font-medium text-slate-100"
                light="text-sm font-medium text-slate-900"
              >
                Desktop notifications
              </ThemeText>
              <ThemeText
                dark="text-xs text-slate-400"
                light="text-xs text-slate-500"
              >
                Notify when a background task finishes.
              </ThemeText>
            </View>
            <Switch checked={enabled()} onCheckedChange={setEnabled} />
          </View>
          <Separator />
          <Switch disabled label="Experimental renderer" />
        </CardContent>
      </Card>
    </Preview>
  );
}

function ProgressPage() {
  const [value, setValue] = createSignal(64);
  let animation: AnimationControls | undefined;
  const moveTo = (target: number) => {
    animation?.stop();
    animation = animate(value(), target, {
      duration: 0.35,
      ease: [0.22, 1, 0.36, 1],
      onUpdate: setValue,
    });
  };
  onCleanup(() => animation?.stop());
  return (
    <Preview title="Interactive">
      <View class="w-96 flex flex-col gap-4">
        <View class="flex justify-between gap-4">
          <ThemeText
            dark="text-sm text-slate-200"
            light="text-sm text-slate-700"
          >
            Building application
          </ThemeText>
          <ThemeText
            dark="text-sm font-mono text-slate-400"
            light="text-sm font-mono text-slate-500"
          >
            {value()}%
          </ThemeText>
        </View>
        <Progress value={value()} />
        <View class="flex gap-2">
          <Button size="sm" onClick={() => moveTo(Math.min(100, value() + 10))}>
            Advance
          </Button>
          <Button size="sm" variant="ghost" onClick={() => moveTo(0)}>
            Reset
          </Button>
        </View>
      </View>
    </Preview>
  );
}

function FpsPage() {
  return (
    <View class="flex flex-col gap-5">
      <Preview title="Live host frame rate">
        <View class="flex items-center gap-3 p-4">
          <Fps />
          <ThemeText
            dark="text-sm text-slate-400"
            light="text-sm text-slate-600"
          >
            Updated once per second from the native animation clock.
          </ThemeText>
        </View>
      </Preview>
      <Preview title="Performance thresholds">
        <View class="flex items-center gap-3 p-4">
          <Fps value={60} />
          <Fps value={42} />
          <Fps value={18} />
          <Fps value={0} label="waiting" />
        </View>
      </Preview>
    </View>
  );
}

function ScrollAreaPage() {
  const theme = useComponentsTheme();
  return (
    <View class="flex flex-col gap-4">
      <Preview title="Custom native overlay scrollbar">
        <View
          class={
            theme() === "dark"
              ? "w-96 h-48 flex flex-col m-4 rounded-lg border border-slate-700 overflow-hidden"
              : "w-96 h-48 flex flex-col m-4 rounded-lg border border-slate-200 overflow-hidden"
          }
        >
          <ScrollArea
            contentClass="p-2 gap-1"
            scrollbar={{
              visibility: "always",
              thickness: 12,
              margin: 3,
              minThumbLength: 36,
              radius: 6,
              trackColor: 0x0f172a22,
              thumbColor: 0x38bdf8cc,
              hoverColor: 0x38bdf8ff,
              activeColor: 0x0284c7ff,
            }}
          >
            <For each={Array.from({ length: 16 }, (_, index) => index + 1)}>
              {(index) => (
                <View
                  class={
                    theme() === "dark"
                      ? "h-8 flex-none px-3 flex items-center rounded bg-slate-800"
                      : "h-8 flex-none px-3 flex items-center rounded bg-slate-100"
                  }
                >
                  <Text
                    class={
                      theme() === "dark"
                        ? "text-sm text-slate-300"
                        : "text-sm text-slate-700"
                    }
                  >
                    Scrollable row {index}
                  </Text>
                </View>
              )}
            </For>
          </ScrollArea>
        </View>
      </Preview>
    </View>
  );
}

function OverlayPage() {
  const [open, setOpen] = createSignal(false);
  return (
    <Preview title="Modal plane and semantic isolation">
      <View class="p-4 flex items-start">
        <Button onClick={() => setOpen(true)}>Open modal overlay</Button>
      </View>
      <Show when={open()}>
        <Portal
          plane="modal"
          role="dialog"
          aria-label="Overlay settings"
          class="absolute left-0 top-0 w-full h-full flex items-center justify-center bg-slate-950"
          onClick={() => setOpen(false)}
          onKeyDown={(event: { key?: string }) => {
            if (event.key === "Escape") setOpen(false);
          }}
        >
          <View
            class="w-96 p-6 flex flex-col gap-4 rounded-xl border border-slate-700 bg-slate-900"
            onClick={(event: { stopPropagation(): void }) =>
              event.stopPropagation()
            }
          >
            <Text class="text-xl font-semibold text-white">Modal overlay</Text>
            <Text class="text-sm text-slate-300">
              {
                "This subtree is painted and hit-tested above floating content. While open, AccessKit exposes only this modal plane beneath the window."
              }
            </Text>
            <View class="flex justify-end">
              <Button onClick={() => setOpen(false)}>Close</Button>
            </View>
          </View>
        </Portal>
      </Show>
    </Preview>
  );
}

function AnimationPage() {
  // w-72 (288px) track minus w-10 (40px) animated item.
  const trackTravel = 248;
  const [springX, setSpringX] = createSignal(16);
  const [linearX, setLinearX] = createSignal(0);
  const [easeX, setEaseX] = createSignal(0);
  const [keyframeX, setKeyframeX] = createSignal(0);
  const [color, setColor] = createSignal("#38bdf8");
  const [opacity, setOpacity] = createSignal(0.25);
  const [paused, setPaused] = createSignal(false);
  const [speed, setSpeed] = createSignal(1);
  let animations: AnimationControls[] = [];

  const stop = () => {
    for (const animation of animations) animation.stop();
    animations = [];
  };
  const restart = () => {
    stop();
    setSpringX(16);
    setLinearX(0);
    setEaseX(0);
    setKeyframeX(0);
    setColor("#38bdf8");
    setOpacity(0.25);
    setPaused(false);
    setSpeed(1);
    animations = [
      // Springs need symmetric headroom so overshoot remains visible instead
      // of being clipped against either rounded edge of the track.
      animate(16, trackTravel - 16, {
        type: "spring",
        stiffness: 120,
        damping: 18,
        repeat: Infinity,
        repeatType: "reverse",
        repeatDelay: 0.5,
        onUpdate: setSpringX,
      }),
      animate(0, trackTravel, {
        duration: 2.4,
        ease: "linear",
        repeat: Infinity,
        repeatType: "reverse",
        repeatDelay: 0.5,
        onUpdate: setLinearX,
      }),
      animate(0, trackTravel, {
        duration: 2.4,
        ease: [0.22, 1, 0.36, 1],
        repeat: Infinity,
        repeatType: "reverse",
        repeatDelay: 0.5,
        onUpdate: setEaseX,
      }),
      animateKeyframes([0, trackTravel, 72, 212, 0], {
        duration: 6,
        times: [0, 0.3, 0.5, 0.72, 1],
        ease: "easeInOut",
        repeat: Infinity,
        repeatDelay: 0.8,
        onUpdate: setKeyframeX,
      }),
      animate("#38bdf8", "#a855f7", {
        duration: 2.4,
        ease: "easeInOut",
        repeat: Infinity,
        repeatType: "reverse",
        repeatDelay: 0.5,
        onUpdate: setColor,
      }),
      animate(0.25, 1, {
        duration: 1.8,
        ease: "easeOut",
        repeat: Infinity,
        repeatType: "reverse",
        repeatDelay: 0.8,
        onUpdate: setOpacity,
      }),
    ];
  };
  const togglePaused = () => {
    const next = !paused();
    for (const animation of animations) {
      if (next) animation.pause();
      else animation.play();
    }
    setPaused(next);
  };
  const toggleSpeed = () => {
    const next = speed() === 1 ? 0.5 : 1;
    for (const animation of animations) animation.speed = next;
    setSpeed(next);
  };

  onMount(restart);
  onCleanup(stop);

  return (
    <View class="flex flex-col gap-5">
      <Preview title="Spring physics">
        <View class="p-4">
          <View class="w-72 flex flex-col gap-4">
            <View class="h-14 flex items-center rounded-full bg-slate-800 overflow-hidden">
              <View
                class="w-10 h-10 rounded-full bg-sky-400"
                transform={translate2d(springX(), 0)}
              />
            </View>
            <View class="flex justify-between">
              <ThemeText
                dark="text-xs text-slate-500"
                light="text-xs text-slate-500"
              >
                stiffness 120
              </ThemeText>
              <ThemeText
                dark="text-xs text-slate-500"
                light="text-xs text-slate-500"
              >
                damping 18
              </ThemeText>
            </View>
          </View>
        </View>
      </Preview>

      <Preview title="Easing comparison">
        <View class="p-4">
          <View class="w-72 flex flex-col gap-3">
            <View class="h-8 flex items-center rounded-md bg-slate-800">
              <View
                class="w-10 h-6 flex items-center justify-center rounded bg-slate-400"
                transform={translate2d(linearX(), 0)}
              >
                <Text class="text-xs font-bold text-slate-950">L</Text>
              </View>
            </View>
            <View class="h-8 flex items-center rounded-md bg-slate-800">
              <View
                class="w-10 h-6 flex items-center justify-center rounded bg-emerald-400"
                transform={translate2d(easeX(), 0)}
              >
                <Text class="text-xs font-bold text-emerald-950">E</Text>
              </View>
            </View>
            <ThemeText
              dark="text-xs text-slate-500"
              light="text-xs text-slate-500"
            >
              Linear versus cubic-bezier easing
            </ThemeText>
          </View>
        </View>
      </Preview>

      <Preview title="Color and opacity interpolation">
        <View class="p-4">
          <View
            class="w-44 h-24 flex items-center justify-center rounded-xl"
            style={{ "background-color": color(), opacity: opacity() }}
          >
            <Text class="text-sm font-semibold text-white">Native paint</Text>
          </View>
        </View>
      </Preview>

      <Preview title="Keyframes">
        <View class="p-4">
          <View class="w-72 flex flex-col gap-3">
            <View class="h-12 flex items-center rounded-lg bg-slate-800 overflow-hidden">
              <View
                class="w-10 h-8 flex items-center justify-center rounded-md bg-violet-400"
                transform={translate2d(keyframeX(), 0)}
              >
                <Text class="text-xs font-bold text-violet-950">K</Text>
              </View>
            </View>
            <ThemeText
              dark="text-xs text-slate-500"
              light="text-xs text-slate-500"
            >
              0 → 248 → 72 → 212 → 0
            </ThemeText>
          </View>
        </View>
      </Preview>

      <View class="flex justify-center gap-2">
        <Button variant="outline" onClick={togglePaused}>
          {paused() ? "Resume" : "Pause"}
        </Button>
        <Button variant="outline" onClick={toggleSpeed}>
          {speed()}× speed
        </Button>
        <Button onClick={restart}>Restart animations</Button>
      </View>
    </View>
  );
}

function PlatformPage() {
  const window = useWindow();
  const [angle, setAngle] = createSignal(0);
  const animation = animate(0, Math.PI * 2, {
    duration: 20,
    ease: "linear",
    repeat: Infinity,
    onUpdate: setAngle,
  });
  onCleanup(() => animation.stop());
  const cx = () => (0.7885 * Math.cos(angle())).toFixed(6);
  const cy = () => (0.7885 * Math.sin(angle())).toFixed(6);

  return (
    <View class="flex flex-col gap-5">
      <Preview title="Rust custom widget">
        <View class="flex flex-col items-center gap-4 p-4">
          <fractal
            class="w-64 h-64 overflow-hidden rounded-xl"
            cx={cx()}
            cy={cy()}
          />
          <ThemeText
            dark="text-xs font-mono text-slate-400"
            light="text-xs font-mono text-slate-600"
          >
            c = {cx()} + {cy()}i · driven by @wabou/animation
          </ThemeText>
        </View>
      </Preview>
      <Preview title="Window capability">
        <View class="flex items-center justify-between gap-4 p-4">
          <View class="flex flex-col gap-1">
            <ThemeText
              dark="text-sm font-medium text-slate-100"
              light="text-sm font-medium text-slate-900"
            >
              Current window #{window.id}
            </ThemeText>
            <ThemeText
              dark="text-xs text-slate-500"
              light="text-xs text-slate-500"
            >
              {window.width()} × {window.height()} logical pixels
            </ThemeText>
          </View>
          <Button
            onClick={() =>
              createWindow({
                title: "Wabou child window",
                width: 640,
                height: 420,
                minWidth: 360,
                minHeight: 240,
              })
            }
          >
            Open native window
          </Button>
        </View>
      </Preview>
    </View>
  );
}

function ChildWindowPage() {
  const window = useWindow();
  return (
    <View class="w-full h-full flex flex-col items-center justify-center gap-4 bg-slate-900 text-white">
      <Text class="text-2xl font-semibold">Independent native window</Text>
      <Text class="text-sm text-slate-400">
        Window #{window.id} · {window.width()} × {window.height()}
      </Text>
      <View class="flex gap-2">
        <Button onClick={() => window.setMaximized(!window.maximized())}>
          Toggle maximize
        </Button>
        <Button
          variant="ghost"
          onClick={() => window.setTitle("Renamed child")}
        >
          Rename
        </Button>
        <Button variant="ghost" onClick={() => window.close()}>
          Close
        </Button>
      </View>
    </View>
  );
}

function AlertPage() {
  return (
    <View class="flex flex-col gap-5">
      <Preview title="Default">
        <View class="w-full max-w-xl">
          <Alert title="Heads up">A newer framework build is available.</Alert>
        </View>
      </Preview>
      <Preview title="Destructive">
        <View class="w-full max-w-xl">
          <Alert title="Build failed" variant="destructive">
            The native bundle could not be linked.
          </Alert>
        </View>
      </Preview>
    </View>
  );
}

function UtilitiesPage() {
  const [typedWidth, setTypedWidth] = createSignal(72);
  const hover = createHover();
  return (
    <View class="flex flex-col gap-5">
      <Preview title="Taffy layout utilities">
        <View class="w-96 flex flex-col gap-3 p-[13px] rounded-lg bg-slate-900">
          <View class="flex items-center justify-between gap-3">
            <Text class="text-sm font-semibold text-slate-100">
              Native utility parser
            </Text>
            <Badge variant="success">Rust</Badge>
          </View>
          <View class="flex gap-2">
            <View class="w-38% h-8 rounded-md bg-sky-500" />
            <View class="flex-1 h-8 rounded-md bg-violet-400" />
          </View>
          <Text class="text-xs text-slate-400">
            flex · gap-2 · p-[13px] · w-38% · flex-1
          </Text>
        </View>
      </Preview>
      <Preview title="Strict arbitrary values">
        <View class="flex items-center gap-3 p-4">
          <View class="w-[12.5%] h-10 rounded-md bg-emerald-400" />
          <View class="flex-1 min-w-0">
            <Text class="text-sm font-medium text-slate-200">
              Typed, not arbitrary CSS
            </Text>
            <Text class="text-xs text-slate-500">
              px, rem and percent values compile to native Style IR.
            </Text>
          </View>
        </View>
      </Preview>
      <Preview title="Typed dynamic style">
        <View class="w-96 flex flex-col gap-3 p-4 rounded-lg bg-slate-900">
          <View
            class="h-8 rounded-md"
            classList={{
              "bg-sky-500": !hover.hovered(),
              "bg-violet-400": hover.hovered(),
            }}
            style={{
              width: px(typedWidth()),
              opacity: styleNumber(0.85),
            }}
            {...hover.bindings}
          />
          <View class="flex items-center gap-3">
            <Button
              size="sm"
              onClick={() =>
                setTypedWidth((value) => (value >= 280 ? 72 : value + 32))
              }
            >
              Resize
            </Button>
            <Text class="text-xs text-slate-400">
              width: px({typedWidth()}) — numeric binary protocol
            </Text>
          </View>
        </View>
      </Preview>
    </View>
  );
}

const COLOR_STOPS = [50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950];
const COLOR_FAMILIES = [
  "rose",
  "pink",
  "fuchsia",
  "purple",
  "violet",
  "indigo",
  "blue",
  "sky",
  "cyan",
  "teal",
  "emerald",
  "green",
  "lime",
  "yellow",
  "amber",
  "orange",
  "red",
  "gray",
  "slate",
  "zinc",
  "neutral",
  "stone",
] as const;

const colorHex = (value: number) =>
  `#${(value >>> 8).toString(16).padStart(6, "0")}`;

const darkTextOn = (value: number) => {
  const red = (value >>> 24) & 0xff;
  const green = (value >>> 16) & 0xff;
  const blue = (value >>> 8) & 0xff;
  return red * 299 + green * 587 + blue * 114 > 160_000;
};

function ColorSwatch(props: { value: number; label: string }) {
  return (
    <View
      class="h-16 flex-1 min-w-0 p-2 flex flex-col justify-between rounded-md"
      style={{ "background-color": rgba(props.value) }}
    >
      <Text
        class={
          darkTextOn(props.value)
            ? "text-xs font-mono font-semibold text-slate-950"
            : "text-xs font-mono font-semibold text-white"
        }
      >
        {props.label}
      </Text>
      <Text
        class={
          darkTextOn(props.value)
            ? "text-xs font-mono text-slate-700"
            : "text-xs font-mono text-slate-200"
        }
      >
        {colorHex(props.value)}
      </Text>
    </View>
  );
}

function ColorsPage() {
  const theme = useComponentsTheme();
  const colors: Readonly<Record<string, number>> = wabouUtilityManifest.colors;
  return (
    <View class="flex flex-col gap-5">
      <Preview title="Base colors">
        <View class="w-full flex gap-3">
          <For each={["transparent", "black", "white"]}>
            {(token) => (
              <View
                class={
                  theme() === "dark"
                    ? "h-20 flex-1 p-3 flex flex-col justify-between rounded-lg border border-slate-700"
                    : "h-20 flex-1 p-3 flex flex-col justify-between rounded-lg border border-slate-300"
                }
                style={{ "background-color": rgba(colors[token]) }}
              >
                <Text
                  class={
                    token === "black"
                      ? "text-sm font-mono font-semibold text-white"
                      : token === "white" || theme() === "light"
                        ? "text-sm font-mono font-semibold text-slate-900"
                        : "text-sm font-mono font-semibold text-slate-100"
                  }
                >
                  {token}
                </Text>
                <Text
                  class={
                    token === "black"
                      ? "text-xs font-mono text-slate-300"
                      : token === "white" || theme() === "light"
                        ? "text-xs font-mono text-slate-600"
                        : "text-xs font-mono text-slate-400"
                  }
                >
                  0x{colors[token].toString(16).padStart(8, "0")}
                </Text>
              </View>
            )}
          </For>
        </View>
      </Preview>

      <View class="flex flex-col gap-5">
        <For each={COLOR_FAMILIES}>
          {(family) => (
            <View class="flex flex-col gap-2">
              <View class="flex items-center justify-between">
                <ThemeText
                  dark="text-sm font-semibold text-slate-200"
                  light="text-sm font-semibold text-slate-800"
                >
                  {family}
                </ThemeText>
                <ThemeText
                  dark="text-xs font-mono text-slate-500"
                  light="text-xs font-mono text-slate-500"
                >
                  text-{family}-* · bg-{family}-* · border-{family}-*
                </ThemeText>
              </View>
              <View class="flex gap-1">
                <For each={COLOR_STOPS}>
                  {(stop) => {
                    const token = `${family}-${stop}`;
                    return (
                      <ColorSwatch value={colors[token]} label={String(stop)} />
                    );
                  }}
                </For>
              </View>
            </View>
          )}
        </For>
      </View>
    </View>
  );
}

function ShadowTile(props: {
  title: string;
  detail: string;
  shadows: Parameters<typeof View>[0]["shadows"];
  shape?: "rounded" | "square" | "rotated";
}) {
  return (
    <View class="flex-1 min-w-40 flex flex-col items-center gap-5 p-6">
      <View
        class={
          props.shape === "square"
            ? "w-32 h-24 flex items-center justify-center rounded-none bg-slate-50"
            : props.shape === "rotated"
              ? "w-32 h-24 flex items-center justify-center rounded-xl bg-slate-50 rotate-6"
              : "w-32 h-24 flex items-center justify-center rounded-xl bg-slate-50"
        }
        shadows={props.shadows}
      >
        <Text class="text-xs font-semibold text-slate-700">{props.title}</Text>
      </View>
      <Text class="text-xs font-mono text-slate-500">{props.detail}</Text>
    </View>
  );
}

function ShadowsPage() {
  return (
    <View class="flex flex-col gap-5">
      <Preview title="Wabou shadow scale">
        <View class="w-full flex flex-wrap gap-6 p-6">
          <View class="flex-1 min-w-32 h-24 flex items-center justify-center rounded-xl bg-slate-50 shadow-xs">
            <Text class="text-xs font-mono text-slate-700">shadow-xs</Text>
          </View>
          <View class="flex-1 min-w-32 h-24 flex items-center justify-center rounded-xl bg-slate-50 shadow-sm">
            <Text class="text-xs font-mono text-slate-700">shadow-sm</Text>
          </View>
          <View class="flex-1 min-w-32 h-24 flex items-center justify-center rounded-xl bg-slate-50 shadow">
            <Text class="text-xs font-mono text-slate-700">shadow</Text>
          </View>
          <View class="flex-1 min-w-32 h-24 flex items-center justify-center rounded-xl bg-slate-50 shadow-md">
            <Text class="text-xs font-mono text-slate-700">shadow-md</Text>
          </View>
          <View class="flex-1 min-w-32 h-24 flex items-center justify-center rounded-xl bg-slate-50 shadow-lg">
            <Text class="text-xs font-mono text-slate-700">shadow-lg</Text>
          </View>
          <View class="flex-1 min-w-32 h-24 flex items-center justify-center rounded-xl bg-slate-50 shadow-xl">
            <Text class="text-xs font-mono text-slate-700">shadow-xl</Text>
          </View>
        </View>
      </Preview>

      <Preview title="Gaussian standard deviation">
        <View class="w-full flex flex-wrap gap-2">
          <For each={[0, 2, 6, 12]}>
            {(stdDev) => (
              <ShadowTile
                title={`${stdDev}`}
                detail={`stdDev: ${stdDev}`}
                shadows={[
                  shadow({
                    offsetY: 6,
                    stdDev,
                    color: 0x0f172a66,
                  }),
                ]}
              />
            )}
          </For>
        </View>
      </Preview>

      <Preview title="Signed spread and two-axis offset">
        <View class="w-full flex flex-wrap gap-2">
          <ShadowTile
            title="contract"
            detail="spread: -5"
            shadows={[
              shadow({ offsetY: 8, spread: -5, stdDev: 5, color: 0x0f172a80 }),
            ]}
          />
          <ShadowTile
            title="neutral"
            detail="spread: 0"
            shadows={[shadow({ offsetY: 8, stdDev: 5, color: 0x0f172a80 })]}
          />
          <ShadowTile
            title="expand"
            detail="spread: 6"
            shadows={[
              shadow({ offsetY: 4, spread: 6, stdDev: 4, color: 0x0ea5e94d }),
            ]}
          />
          <ShadowTile
            title="offset"
            detail="offset: 12, -8"
            shadows={[
              shadow({
                offsetX: 12,
                offsetY: -8,
                stdDev: 5,
                color: 0x7c3aed66,
              }),
            ]}
          />
        </View>
      </Preview>

      <Preview title="Ordered layers, color, radius and transform">
        <View class="w-full flex flex-wrap gap-2">
          <ShadowTile
            title="layers"
            detail="3 ordered layers"
            shadows={[
              shadow({ offsetX: -8, stdDev: 8, color: 0x06b6d466 }),
              shadow({ offsetX: 8, stdDev: 8, color: 0xd946ef66 }),
              shadow({ offsetY: 10, spread: -3, stdDev: 4, color: 0x0f172a80 }),
            ]}
          />
          <ShadowTile
            title="radius"
            detail="radius: 24"
            shape="square"
            shadows={[
              shadow({
                offsetY: 6,
                spread: 2,
                stdDev: 5,
                radius: 24,
                color: 0x10b98180,
              }),
            ]}
          />
          <ShadowTile
            title="affine"
            detail="rotate: 6deg"
            shape="rotated"
            shadows={[
              shadow({
                offsetX: 8,
                offsetY: 8,
                stdDev: 5,
                color: 0x0f172a80,
              }),
            ]}
          />
        </View>
      </Preview>

      <PropertyRow name="offsetX / offsetY" value="finite logical pixels" />
      <PropertyRow name="spread" value="signed logical pixels" />
      <PropertyRow
        name="stdDev"
        value="Gaussian standard deviation passed directly to Vello"
      />
      <PropertyRow name="color" value="packed sRGBA (0xRRGGBBAA)" />
      <PropertyRow
        name="radius"
        value="optional independent rounded-rectangle radius"
      />
    </View>
  );
}

function SeparatorPage() {
  return (
    <Preview title="Orientations">
      <View class="w-96 flex flex-col gap-4">
        <ThemeText dark="text-sm text-slate-200" light="text-sm text-slate-700">
          Account settings
        </ThemeText>
        <Separator />
        <View class="h-8 flex items-center gap-4">
          <ThemeText
            dark="text-sm text-slate-400"
            light="text-sm text-slate-600"
          >
            Profile
          </ThemeText>
          <Separator orientation="vertical" />
          <ThemeText
            dark="text-sm text-slate-400"
            light="text-sm text-slate-600"
          >
            Security
          </ThemeText>
          <Separator orientation="vertical" />
          <ThemeText
            dark="text-sm text-slate-400"
            light="text-sm text-slate-600"
          >
            Billing
          </ThemeText>
        </View>
      </View>
    </Preview>
  );
}

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
  <MemoryRouter history={history} preload={false} explicitLinks>
    <Route path={["/", "/components/:component"]} component={App} />
  </MemoryRouter>
));
