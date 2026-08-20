import {
  Alert,
  type AnimationControls,
  animate,
  Badge,
  Button,
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
  Checkbox,
  ConfigEditor,
  createHover,
  createWindow,
  Fps,
  Input,
  Kbd,
  KbdGroup,
  MotionConfigProvider,
  NumberField,
  Progress,
  ProgressFill,
  ProgressLabel,
  ProgressRoot,
  ProgressTrack,
  ProgressValueLabel,
  px,
  RadioGroup,
  RadioGroupItem,
  ScrollArea,
  SearchField,
  Separator,
  Skeleton,
  Slider,
  Spinner,
  Switch,
  number as styleNumber,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  Text,
  TextArea,
  TitleBar,
  TitleBarDragRegion,
  Toggle,
  ToggleGroup,
  ToggleGroupItem,
  useComponentsTheme,
  useWindow,
  View,
} from "@wabou/ui";
import { createSignal, For, onCleanup } from "solid-js";
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
      <Card class="w-72">
        <CardHeader>
          <CardTitle>Create project</CardTitle>
          <CardDescription maxLines={2}>
            Deploy a new Wabou application from a reusable project template with
            predictable native text truncation.
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
  const [query, setQuery] = createSignal("native");
  const [config, setConfig] = createSignal('{\n  "enabled": true\n}');
  const [configEdited, setConfigEdited] = createSignal(false);
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
            aria-label="Workspace name"
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
      <Preview title="Search">
        <View class="w-96 flex flex-col gap-2">
          <SearchField
            aria-label="Search documentation"
            value={query()}
            placeholder="Search documentation"
            onValueChange={setQuery}
          />
          <Text
            role="status"
            aria-label="Search query"
            class="text-xs text-muted"
          >
            Query: {query() || "—"}
          </Text>
        </View>
      </Preview>
      <Preview title="Configuration editor">
        <View class="w-full flex flex-col gap-2">
          <ConfigEditor
            aria-label="JSON configuration"
            value={config()}
            onInput={(event) => {
              setConfig(event.currentTarget.value);
              setConfigEdited(true);
            }}
          />
          <Text class="text-xs text-muted">Config: {config()}</Text>
          <Text class="text-xs text-muted">
            {configEdited() ? "Config edited" : "Config unchanged"}
          </Text>
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
            <Switch
              checked={enabled()}
              aria-label="Desktop notifications"
              onCheckedChange={setEnabled}
            />
          </View>
          <Separator />
          <Switch disabled label="Experimental renderer" />
        </CardContent>
      </Card>
    </Preview>
  );
}

function CheckboxPage() {
  const [accepted, setAccepted] = createSignal(false);
  return (
    <View class="flex flex-col gap-5">
      <Preview title="Selection states">
        <View class="w-96 flex flex-col gap-4">
          <Checkbox
            checked={accepted()}
            onCheckedChange={setAccepted}
            label="Accept the terms and conditions"
          />
          <Checkbox defaultChecked label="Selected by default" />
          <Checkbox indeterminate label="Some child items selected" />
          <Checkbox disabled label="Unavailable option" />
        </View>
      </Preview>
      <PropertyRow
        name="state"
        value="controlled | uncontrolled | indeterminate | disabled"
      />
    </View>
  );
}

function RadioGroupPage() {
  const [plan, setPlan] = createSignal("pro");
  return (
    <Preview title="Plan selection">
      <Card class="w-96">
        <CardHeader>
          <CardTitle>Choose a plan</CardTitle>
          <CardDescription>Only one option can be selected.</CardDescription>
        </CardHeader>
        <CardContent>
          <RadioGroup
            value={plan()}
            onValueChange={setPlan}
            aria-label="Subscription plan"
          >
            <RadioGroupItem value="free" label="Free — local projects" />
            <RadioGroupItem value="pro" label="Pro — team collaboration" />
            <RadioGroupItem value="enterprise" label="Enterprise — managed" />
          </RadioGroup>
          <Text class="text-xs text-muted">{`Selected: ${plan()}`}</Text>
        </CardContent>
      </Card>
    </Preview>
  );
}

function TogglePage() {
  const [bold, setBold] = createSignal(true);
  const [italic, setItalic] = createSignal(false);
  const [mode, setMode] = createSignal("rule");
  return (
    <View class="flex flex-col gap-5">
      <Preview title="Formatting toolbar">
        <View class="flex items-center gap-1 p-1 rounded-lg bg-control">
          <Toggle
            pressed={bold()}
            onPressedChange={setBold}
            aria-label="Toggle bold"
          >
            B
          </Toggle>
          <Toggle
            pressed={italic()}
            onPressedChange={setItalic}
            aria-label="Toggle italic"
          >
            I
          </Toggle>
          <Toggle variant="outline" aria-label="Pin item">
            Pin
          </Toggle>
        </View>
      </Preview>
      <Preview title="Single selection">
        <ToggleGroup
          type="single"
          value={mode()}
          onValueChange={setMode}
          aria-label="Routing mode"
          class="w-80"
        >
          <ToggleGroupItem value="rule">Rule</ToggleGroupItem>
          <ToggleGroupItem value="global">Global</ToggleGroupItem>
          <ToggleGroupItem value="direct">Direct</ToggleGroupItem>
        </ToggleGroup>
      </Preview>
    </View>
  );
}

function TabsPage() {
  return (
    <Preview title="Account settings">
      <Tabs defaultValue="account" class="w-[460px]">
        <TabsList aria-label="Settings sections">
          <TabsTrigger value="account">Account</TabsTrigger>
          <TabsTrigger value="security">Security</TabsTrigger>
          <TabsTrigger value="billing">Billing</TabsTrigger>
        </TabsList>
        <TabsContent value="account">
          <Card>
            <CardContent>
              <Text class="text-sm font-medium text-primary">Account</Text>
              <Text class="text-sm text-muted">
                Update your public profile and contact details.
              </Text>
            </CardContent>
          </Card>
        </TabsContent>
        <TabsContent value="security">
          <Alert title="Two-factor authentication">
            Add another factor to protect this account.
          </Alert>
        </TabsContent>
        <TabsContent value="billing">
          <Card>
            <CardContent>
              <Text class="text-sm text-secondary">
                Your next invoice is due on September 1.
              </Text>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </Preview>
  );
}

function SkeletonPage() {
  return (
    <View class="flex flex-col gap-5">
      <Preview title="Loading card">
        <View class="w-96 flex items-center gap-4">
          <Skeleton class="w-12 h-12 rounded-full" />
          <View class="flex-1 flex flex-col gap-2">
            <Skeleton class="w-2/3 h-4" />
            <Skeleton class="w-full h-3" />
            <Skeleton class="w-4/5 h-3" />
          </View>
        </View>
      </Preview>
      <Preview title="Reduced motion policy">
        <MotionConfigProvider reducedMotion>
          <View class="w-96 flex flex-col gap-2">
            <Skeleton class="w-full h-4" />
            <Skeleton class="w-4/5 h-3" />
          </View>
        </MotionConfigProvider>
      </Preview>
    </View>
  );
}

function SpinnerPage() {
  return (
    <Preview title="Indeterminate progress">
      <View class="flex items-center gap-3">
        <Spinner label="Syncing workspace" />
        <Text class="text-sm text-secondary">Syncing workspace…</Text>
      </View>
      <Button disabled>
        <View class="flex items-center gap-2">
          <Spinner label="Saving workspace" class="text-on-accent" />
          <Text class="text-sm text-on-accent">Saving</Text>
        </View>
      </Button>
    </Preview>
  );
}

function KbdPage() {
  return (
    <Preview title="Keyboard shortcuts">
      <View class="w-96 flex flex-col gap-3">
        <View class="flex items-center justify-between">
          <Text class="text-sm text-secondary">Open command palette</Text>
          <KbdGroup>
            <Kbd>Ctrl</Kbd>
            <Text class="text-xs text-muted">+</Text>
            <Kbd>K</Kbd>
          </KbdGroup>
        </View>
        <View class="flex items-center justify-between">
          <Text class="text-sm text-secondary">Close overlay</Text>
          <Kbd>Esc</Kbd>
        </View>
      </View>
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
    <View class="flex flex-col gap-5">
      <Preview title="Interactive shorthand">
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
          <Progress label="Build progress" value={value()} />
          <View class="flex gap-2">
            <Button
              size="sm"
              onClick={() => moveTo(Math.min(100, value() + 10))}
            >
              Advance
            </Button>
            <Button size="sm" variant="ghost" onClick={() => moveTo(0)}>
              Reset
            </Button>
          </View>
        </View>
      </Preview>
      <Preview title="Composable and indeterminate">
        <View class="w-96 flex flex-col gap-5">
          <ProgressRoot
            label="Downloaded release archive"
            value={48}
            maxValue={64}
            getValueLabel={({ value: current, max }) =>
              `${current} of ${max} MiB`
            }
          >
            <View class="flex items-center justify-between gap-3">
              <ProgressLabel />
              <ProgressValueLabel />
            </View>
            <ProgressTrack>
              <ProgressFill />
            </ProgressTrack>
          </ProgressRoot>
          <ProgressRoot label="Resolving dependencies" indeterminate>
            <View class="flex items-center justify-between gap-3">
              <ProgressLabel />
              <ProgressValueLabel />
            </View>
            <ProgressTrack>
              <ProgressFill />
            </ProgressTrack>
          </ProgressRoot>
        </View>
      </Preview>
    </View>
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

function SliderPage() {
  const [value, setValue] = createSignal(35);
  return (
    <View class="flex flex-col gap-5">
      <Preview title="Controlled value">
        <View class="w-96 flex flex-col gap-3">
          <View class="flex items-center justify-between">
            <Text class="text-sm text-secondary">Volume</Text>
            <Text
              role="status"
              aria-label="Slider value"
              class="font-mono text-sm text-primary"
            >
              {`${value()}%`}
            </Text>
          </View>
          <Slider
            label="Volume"
            value={value()}
            onValueChange={setValue}
            valueText={(next) => `${next} percent`}
          />
        </View>
      </Preview>
      <Preview title="Steps and disabled state">
        <View class="w-96 flex flex-col gap-5">
          <Slider
            label="Temperature"
            min={-20}
            max={40}
            step={5}
            defaultValue={15}
          />
          <Slider label="Unavailable range" defaultValue={60} disabled />
        </View>
      </Preview>
      <PropertyRow name="keyboard" value="arrows | page up/down | home | end" />
      <PropertyRow name="pointer" value="click | captured drag" />
    </View>
  );
}

function NumberFieldPage() {
  const [concurrency, setConcurrency] = createSignal(4);
  const [price, setPrice] = createSignal<number | null>(12.5);
  return (
    <View class="flex flex-col gap-5">
      <Preview title="Controlled value">
        <View class="w-96 flex flex-col gap-3">
          <Text class="text-sm font-medium text-primary">
            Download concurrency
          </Text>
          <NumberField
            aria-label="Download concurrency"
            value={concurrency()}
            min={1}
            max={32}
            onValueChange={(value) => setConcurrency(value ?? 1)}
          />
          <Text
            role="status"
            aria-label="Download concurrency value"
            class="text-xs text-muted"
          >
            {`${concurrency()} concurrent tasks`}
          </Text>
        </View>
      </Preview>
      <Preview title="Locale-aware decimals">
        <View class="w-96 flex flex-col gap-3">
          <NumberField
            aria-label="Price in euros"
            value={price()}
            min={0}
            step={0.1}
            locale="de-DE"
            formatOptions={{
              style: "currency",
              currency: "EUR",
              currencyDisplay: "symbol",
            }}
            onValueChange={setPrice}
          />
          <Text class="text-xs text-muted">
            Locale-aware parsing is preserved while the field is being edited.
          </Text>
        </View>
      </Preview>
      <Preview title="States">
        <View class="w-96 flex flex-col gap-3">
          <NumberField aria-label="Optional amount" placeholder="No value" />
          <NumberField aria-label="Read-only amount" value={8} readOnly />
          <NumberField aria-label="Disabled amount" value={12} disabled />
        </View>
      </Preview>
      <PropertyRow name="keyboard" value="arrows | page up/down | home | end" />
      <PropertyRow
        name="semantics"
        value="native spinbutton | min | max | value"
      />
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
            class="flex-1"
            contentClass="p-2 gap-1"
            scrollbar={{
              visibility: "auto",
              hideDelay: 700,
              fadeDuration: 160,
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
            c = {cx()} + {cy()}i · driven by Wabou animation
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
              {`Current window #${window.id.lo}v${window.id.hi}`}
            </ThemeText>
            <ThemeText
              dark="text-xs text-slate-500"
              light="text-xs text-slate-500"
            >
              {window.width()} × {window.height()} logical pixels
            </ThemeText>
          </View>
          <Button
            onClick={() => {
              void createWindow({
                title: "Wabou child window",
                width: 640,
                height: 420,
                minWidth: 360,
                minHeight: 240,
                decorations: false,
              });
            }}
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
    <View class="w-full h-full flex flex-col bg-slate-900 text-white">
      <TitleBar class="border-slate-700 bg-slate-950">
        <TitleBarDragRegion class="px-3">
          <Text class="text-sm font-medium">Custom title bar</Text>
        </TitleBarDragRegion>
        <Button variant="ghost" size="sm" onClick={() => window.minimize()}>
          Minimize
        </Button>
        <Button variant="ghost" size="sm" onClick={() => window.close()}>
          Close
        </Button>
      </TitleBar>
      <View class="flex-1 flex flex-col items-center justify-center gap-4">
        <Text class="text-2xl font-semibold">Independent native window</Text>
        <Text class="text-sm text-slate-400">
          {`Window #${window.id.lo}v${window.id.hi} · ${window.width()} × ${window.height()}`}
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
        </View>
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
  CheckboxPage,
  ChildWindowPage,
  FpsPage,
  InputPage,
  KbdPage,
  NumberFieldPage,
  PlatformPage,
  ProgressPage,
  RadioGroupPage,
  ScrollAreaPage,
  SeparatorPage,
  SkeletonPage,
  SliderPage,
  SpinnerPage,
  SwitchPage,
  TabsPage,
  TogglePage,
  UtilitiesPage,
};
