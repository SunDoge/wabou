import {
  Badge,
  Button,
  Card,
  CardContent,
  createWindowMatch,
  Fps,
  Icon,
  Kbd,
  Path,
  PathBuilder,
  Slider,
  SplitPane,
  SplitPaneAside,
  SplitPaneMain,
  Switch,
  Text,
  useHost,
  useWindow,
  View,
} from "@wabou/ui";
import activity from "lucide-static/icons/activity.svg?raw";
import boxes from "lucide-static/icons/boxes.svg?raw";
import code from "lucide-static/icons/code-2.svg?raw";
import cpu from "lucide-static/icons/cpu.svg?raw";
import layers from "lucide-static/icons/layers-3.svg?raw";
import palette from "lucide-static/icons/palette.svg?raw";
import scan from "lucide-static/icons/scan-line.svg?raw";
import sparkles from "lucide-static/icons/sparkles.svg?raw";
import {
  createMemo,
  createSignal,
  For as ForValue,
  type JSX,
  onCleanup,
} from "solid-js";
import {
  appendFrameSample,
  frameDuration,
  frameStages,
} from "./overview-metrics";

const treeNodes = [
  { id: "shell", name: "ApplicationShell", detail: "root layout region" },
  { id: "rail", name: "NavigationRail", detail: "persistent navigation" },
  { id: "hero", name: "HeroSurface", detail: "rounded paint surface" },
  { id: "chart", name: "FrameTimeline", detail: "retained vector scene" },
] as const;

function LiveFrameChart(props: { samples: readonly number[] }) {
  const source = createMemo(() => {
    const samples = props.samples.length >= 2 ? props.samples : [0, 0];
    const ceiling = Math.max(16.67, ...samples);
    const step = 306 / Math.max(1, samples.length - 1);
    const points = samples.map((value, index) => ({
      x: index * step,
      y: 56 - Math.min(1, value / ceiling) * 44,
    }));
    return new PathBuilder().splineThrough(points).build({
      stroke: 0x38bdf8ff,
      strokeWidth: 2.5,
      lineCap: "round",
      lineJoin: "round",
    });
  });
  return (
    <View class="relative h-16 overflow-hidden rounded-md bg-control">
      <Path class="absolute inset-0 w-full h-full" source={source()} />
    </View>
  );
}

export function OverviewPage(props: {
  theme: string;
  onCycleTheme: () => void;
  onExplore: () => void;
}) {
  const compact = createWindowMatch({ maxWidth: 1099 }, useWindow());
  const host = useHost();
  const [overlayPaint, setOverlayPaint] = createSignal(
    host.diagnostics.overlayPaintStats(),
  );
  const [frameStats, setFrameStats] = createSignal(
    host.diagnostics.frameStats(),
  );
  const [frameSamples, setFrameSamples] = createSignal<number[]>([]);
  const currentFrameDuration = createMemo(() => {
    const stats = frameStats();
    return stats ? frameDuration(stats) : null;
  });
  const currentFrameStages = createMemo(() => {
    const stats = frameStats();
    return stats ? frameStages(stats) : [];
  });
  const frameDurationLabel = createMemo(() => {
    const duration = currentFrameDuration();
    return duration === null ? "--" : `${duration.toFixed(1)} ms`;
  });
  const viewportLabel = createMemo(() => {
    const stats = frameStats();
    return stats
      ? `${Math.round(stats.viewport_w)} x ${Math.round(stats.viewport_h)}`
      : "Waiting for native frame";
  });
  const debugOverlayAvailable = () => overlayPaint() !== null;
  const [selectedNode, setSelectedNode] = createSignal("hero");
  const [motion, setMotion] = createSignal(72);
  const [inspectLayout, setInspectLayout] = createSignal(false);
  let overlayEvidenceTimer: ReturnType<typeof setInterval> | undefined;
  const frameEvidenceTimer = setInterval(() => {
    const next = host.diagnostics.frameStats();
    setFrameStats(next);
    if (next)
      setFrameSamples((samples) =>
        appendFrameSample(samples, frameDuration(next)),
      );
  }, 250);
  const stopOverlayEvidence = () => {
    clearInterval(overlayEvidenceTimer);
    overlayEvidenceTimer = undefined;
  };
  const refreshOverlayEvidence = () =>
    setOverlayPaint(host.diagnostics.overlayPaintStats());
  const toggleLayoutOverlay = (enabled: boolean) => {
    if (host.diagnostics.setOverlay({ layout: enabled })) {
      setInspectLayout(enabled);
      stopOverlayEvidence();
      if (enabled) {
        refreshOverlayEvidence();
        overlayEvidenceTimer = setInterval(refreshOverlayEvidence, 250);
      } else {
        refreshOverlayEvidence();
      }
    }
  };
  const layoutOverlayStatus = () => {
    if (!debugOverlayAvailable()) return "Available in debug builds";
    if (!inspectLayout()) return "Inspect native bounds";
    const paint = overlayPaint();
    if (!paint?.enabled || paint.layout_bounds === 0)
      return "Native paint requested";
    return `${paint.layout_bounds} native bounds · pass ${paint.sequence}`;
  };
  onCleanup(() => {
    clearInterval(frameEvidenceTimer);
    stopOverlayEvidence();
    if (inspectLayout()) host.diagnostics.setOverlay({});
  });
  const selected = () =>
    treeNodes.find((node) => node.id === selectedNode()) ?? treeNodes[0];

  return (
    <View class="w-full flex flex-col gap-6">
      <View
        class={
          compact()
            ? "flex flex-col items-start gap-4"
            : "flex flex-row items-end justify-between gap-6"
        }
      >
        <View class="min-w-0 flex flex-col gap-3">
          <View class="flex items-center gap-2">
            <Badge variant="success">Native renderer</Badge>
            <Badge variant="outline">Solid 2</Badge>
            <Badge variant="outline">Rust host</Badge>
          </View>
          <Text class="text-3xl font-bold text-primary">
            Desktop UI, without a browser engine.
          </Text>
          <Text class="max-w-3xl whitespace-normal text-base text-secondary">
            Solid reactivity drives a retained Rust scene with explicit layout,
            native widgets and predictable desktop behavior.
          </Text>
        </View>
        <View class="flex-none flex items-center gap-2">
          <Button variant="outline" onClick={props.onCycleTheme}>
            <Icon source={palette} size={15} />
            {`Theme: ${props.theme}`}
          </Button>
          <Button onClick={props.onExplore}>
            <Icon source={sparkles} size={15} />
            Explore components
          </Button>
        </View>
      </View>

      <Card class="relative overflow-hidden border-focus bg-surface">
        <CardContent class="p-0">
          <SplitPane
            class={compact() ? "min-h-64 flex-col" : "min-h-64 flex-row"}
          >
            <SplitPaneMain class="p-7 flex flex-col justify-between gap-6">
              <View class="flex items-start gap-4">
                <View class="flex items-center gap-4">
                  <View class="w-12 h-12 flex-none flex items-center justify-center rounded-lg bg-accent shadow-lg">
                    <Text class="text-xl font-bold text-on-accent">W</Text>
                  </View>
                  <View class="flex flex-col gap-1">
                    <Text class="text-lg font-semibold text-primary">
                      One reactive tree. One native scene.
                    </Text>
                    <Text class="text-sm text-muted">
                      Typed operations cross the QuickJS boundary once per
                      commit.
                    </Text>
                    <View class="mt-1 flex items-center gap-2">
                      <View class="w-2 h-2 rounded-full bg-success-primary" />
                      <Text class="text-xs font-medium text-success-primary">
                        Runtime healthy
                      </Text>
                    </View>
                  </View>
                </View>
              </View>

              <View class="flex flex-row gap-3">
                <Metric
                  icon={activity}
                  label="Frame rate"
                  value={<Fps label="" />}
                />
                <Metric
                  icon={boxes}
                  label="Scene nodes"
                  value={frameStats()?.node_count.toLocaleString() ?? "--"}
                />
                <Metric
                  icon={cpu}
                  label="Frame time"
                  value={frameDurationLabel()}
                />
              </View>
            </SplitPaneMain>

            <SplitPaneAside
              class={
                compact()
                  ? "w-full p-6 flex flex-col justify-between gap-5 border-t border-subtle bg-surface-muted"
                  : "w-96 p-6 flex flex-col justify-between gap-5 border-l border-subtle bg-surface-muted"
              }
            >
              <View class="flex items-center justify-between">
                <View class="flex items-center gap-2">
                  <Icon source={activity} size={15} class="text-accent" />
                  <Text class="text-sm font-semibold text-primary">
                    Native frame pipeline
                  </Text>
                </View>
                <Badge variant={frameStats() ? "success" : "outline"}>
                  {frameStats() ? "Live" : "Waiting"}
                </Badge>
              </View>
              <View class="flex flex-col gap-3">
                <LiveFrameChart samples={frameSamples()} />
                <ForValue each={currentFrameStages()}>
                  {(stage) => (
                    <View class="flex items-center gap-3">
                      <Text class="w-12 flex-none text-xs text-muted">
                        {stage.label}
                      </Text>
                      <View class="flex-1 h-1.5 overflow-hidden rounded-full bg-control">
                        <View
                          class="h-full rounded-full bg-accent"
                          style={{ width: `${stage.width}%` }}
                        />
                      </View>
                      <Text class="w-12 flex-none text-right font-mono text-xs text-secondary">
                        {`${stage.value.toFixed(1)} ms`}
                      </Text>
                    </View>
                  )}
                </ForValue>
              </View>
              <View class="flex items-center justify-between pt-3 border-t border-subtle">
                <Text class="text-xs text-muted">Viewport</Text>
                <Text class="font-mono text-xs text-success-primary">
                  {viewportLabel()}
                </Text>
              </View>
            </SplitPaneAside>
          </SplitPane>
        </CardContent>
      </Card>

      <View class={compact() ? "flex flex-col gap-5" : "flex flex-row gap-5"}>
        <Card class="flex-1 min-w-0" shadows={null}>
          <CardContent class="p-0">
            <View class="h-11 px-4 flex items-center justify-between border-b border-subtle bg-surface-muted">
              <View class="flex items-center gap-2">
                <Icon source={scan} size={15} class="text-accent" />
                <Text class="text-sm font-semibold text-primary">
                  Inspector anatomy
                </Text>
              </View>
              <View class="flex items-center gap-2">
                <Kbd>Ctrl</Kbd>
                <Kbd>Shift</Kbd>
                <Kbd>I</Kbd>
              </View>
            </View>
            <SplitPane class="min-h-64">
              <SplitPaneAside class="w-64 p-3 flex flex-col gap-1 border-r border-subtle">
                <ForValue each={treeNodes}>
                  {(node, index) => (
                    <Button
                      variant="ghost"
                      selected={selectedNode() === node.id}
                      class="w-full justify-start"
                      onClick={() => setSelectedNode(node.id)}
                    >
                      <View style={{ width: `${index() * 12}px` }} />
                      <Icon
                        source={index() === 0 ? layers : code}
                        size={14}
                        class={
                          selectedNode() === node.id
                            ? "text-accent"
                            : "text-muted"
                        }
                      />
                      {node.name}
                    </Button>
                  )}
                </ForValue>
              </SplitPaneAside>
              <SplitPaneMain class="p-5 flex flex-col gap-4">
                <View class="flex items-start justify-between gap-4">
                  <View class="flex flex-col gap-1">
                    <Text class="text-base font-semibold text-primary">
                      {selected().name}
                    </Text>
                    <Text class="font-mono text-xs text-muted">
                      {selected().detail}
                    </Text>
                  </View>
                  <Badge variant="success">Selected</Badge>
                </View>
                <View class="grid grid-cols-2 gap-3">
                  <Property label="Display" value="flex" />
                  <Property label="Position" value="relative" />
                  <Property label="Paint plane" value="content" />
                  <Property label="Hit testing" value="enabled" />
                </View>
                <View class="mt-1 p-3 flex flex-col gap-2 rounded-md border border-subtle bg-surface-muted">
                  <View class="flex items-center justify-between">
                    <Text class="text-xs text-muted">Runtime diagnostics</Text>
                    <Text class="font-mono text-xs text-success-primary">
                      DevTools queryable
                    </Text>
                  </View>
                  <View class="flex items-center gap-2">
                    <Badge variant="outline">Snapshot</Badge>
                    <Badge variant="outline">Overlay</Badge>
                    <Badge variant="outline">Trace</Badge>
                  </View>
                </View>
              </SplitPaneMain>
            </SplitPane>
          </CardContent>
        </Card>

        <Card class={compact() ? "w-full" : "w-80 flex-none"} shadows={null}>
          <CardContent class="p-5 flex flex-col gap-5">
            <View class="flex items-center gap-2">
              <Icon source={sparkles} size={15} class="text-accent" />
              <Text class="text-sm font-semibold text-primary">
                Runtime controls
              </Text>
            </View>
            <View class="flex flex-col gap-2">
              <View class="flex items-center justify-between">
                <Text class="text-xs text-muted">Motion intensity</Text>
                <Text class="font-mono text-xs text-secondary">
                  {`${motion()} percent`}
                </Text>
              </View>
              <Slider
                label="Motion intensity"
                value={motion()}
                onValueChange={setMotion}
              />
            </View>
            <View class="flex items-center justify-between py-3 border-t border-b border-subtle">
              <View class="flex flex-col gap-1">
                <Text class="text-sm text-primary">Layout overlay</Text>
                <Text class="text-xs text-muted">{layoutOverlayStatus()}</Text>
              </View>
              <Switch
                checked={inspectLayout()}
                disabled={!debugOverlayAvailable()}
                aria-label="Layout overlay"
                onCheckedChange={toggleLayoutOverlay}
              />
            </View>
            <View class="flex flex-col gap-3">
              <Capability icon={layers} text="Retained GPUI projection" />
              <Capability icon={cpu} text="Rust native widgets" />
              <Capability icon={boxes} text="Explicit overlay planes" />
            </View>
            <Button
              variant="outline"
              class="w-full"
              onClick={props.onCycleTheme}
            >
              <Icon source={palette} size={15} />
              Animate palette
            </Button>
          </CardContent>
        </Card>
      </View>
    </View>
  );
}

function Metric(props: {
  icon: string;
  label: string;
  value: string | JSX.Element;
}) {
  return (
    <View class="flex-1 min-w-0 p-4 flex items-center gap-3 rounded-lg border border-subtle bg-surface-muted shadow-xs">
      <View class="w-8 h-8 flex-none flex items-center justify-center rounded-md bg-control">
        <Icon source={props.icon} size={15} class="text-accent" />
      </View>
      <View class="min-w-0 flex flex-col gap-0.5">
        <Text class="text-xs text-muted">{props.label}</Text>
        {typeof props.value === "string" ? (
          <Text class="font-mono text-sm font-semibold text-primary">
            {props.value}
          </Text>
        ) : (
          props.value
        )}
      </View>
    </View>
  );
}

function Property(props: { label: string; value: string }) {
  return (
    <View class="p-3 flex flex-col gap-1 rounded-md border border-subtle bg-surface-muted">
      <Text class="text-xs text-muted">{props.label}</Text>
      <Text class="font-mono text-xs text-primary">{props.value}</Text>
    </View>
  );
}

function Capability(props: { icon: string; text: string }) {
  return (
    <View class="flex items-center gap-2">
      <Icon source={props.icon} size={14} class="text-success-primary" />
      <Text class="text-xs text-secondary">{props.text}</Text>
    </View>
  );
}
