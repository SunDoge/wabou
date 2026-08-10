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

import { Preview } from "../preview";
import { PropertyRow, ThemeText } from "./showcase";

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
      <Preview title="Multiline">
        <View class="w-96 flex flex-col gap-2">
          <TextArea
            placeholder="Describe your project…"
            value={value()}
            onInput={(event) => setValue(event.currentTarget.value)}
          />
          <TextArea
            readOnly
            value={"Read-only multiline\ncontent remains selectable."}
          />
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

export {
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
};
