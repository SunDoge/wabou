import {
  Badge,
  Button,
  Card,
  CardContent,
  Fps,
  Icon,
  Kbd,
  Progress,
  Slider,
  SplitPane,
  SplitPaneAside,
  SplitPaneMain,
  Switch,
  Text,
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
import { createSignal, For, type JSX } from "solid-js";

const treeNodes = [
  { id: "shell", name: "ApplicationShell", detail: "flex row, 1440 by 900" },
  { id: "rail", name: "NavigationRail", detail: "fixed, 224 by 900" },
  { id: "hero", name: "HeroSurface", detail: "rounded, elevation raised" },
  { id: "chart", name: "FrameTimeline", detail: "native scene, 120 Hz" },
] as const;

const stages = [
  { label: "JS", value: "1.3 ms", width: 28 },
  { label: "Build", value: "2.1 ms", width: 46 },
  { label: "Scene", value: "0.8 ms", width: 18 },
  { label: "Present", value: "1.1 ms", width: 24 },
] as const;

export function OverviewPage(props: {
  theme: string;
  onCycleTheme: () => void;
  onExplore: () => void;
}) {
  const [selectedNode, setSelectedNode] = createSignal("hero");
  const [motion, setMotion] = createSignal(72);
  const [inspectLayout, setInspectLayout] = createSignal(true);
  const selected = () =>
    treeNodes.find((node) => node.id === selectedNode()) ?? treeNodes[0];

  return (
    <View class="w-full flex flex-col gap-6">
      <View class="flex flex-row items-end justify-between gap-6">
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
          <SplitPane class="min-h-64">
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
                <Metric icon={boxes} label="Scene nodes" value="1,284" />
                <Metric icon={cpu} label="Frame budget" value="8.3 ms" />
              </View>
            </SplitPaneMain>

            <SplitPaneAside class="w-96 p-6 flex flex-col justify-between gap-5 border-l border-subtle bg-surface-muted">
              <View class="flex items-center justify-between">
                <View class="flex items-center gap-2">
                  <Icon source={activity} size={15} class="text-accent" />
                  <Text class="text-sm font-semibold text-primary">
                    Native frame pipeline
                  </Text>
                </View>
                <Badge variant="outline">Live</Badge>
              </View>
              <View class="flex flex-col gap-3">
                <For each={stages}>
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
                        {stage.value}
                      </Text>
                    </View>
                  )}
                </For>
              </View>
              <View class="flex items-center justify-between pt-3 border-t border-subtle">
                <Text class="text-xs text-muted">Headroom</Text>
                <Text class="font-mono text-xs font-medium text-success-primary">
                  36 percent
                </Text>
              </View>
            </SplitPaneAside>
          </SplitPane>
        </CardContent>
      </Card>

      <View class="flex flex-row gap-5">
        <Card class="flex-1 min-w-0" shadows={null}>
          <CardContent class="p-0">
            <View class="h-11 px-4 flex items-center justify-between border-b border-subtle bg-surface-muted">
              <View class="flex items-center gap-2">
                <Icon source={scan} size={15} class="text-accent" />
                <Text class="text-sm font-semibold text-primary">
                  Live native inspector
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
                <For each={treeNodes}>
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
                </For>
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
                    <Text class="text-xs text-muted">Layout stability</Text>
                    <Text class="font-mono text-xs text-success-primary">
                      100 percent
                    </Text>
                  </View>
                  <Progress value={100} label="Layout stability" />
                </View>
              </SplitPaneMain>
            </SplitPane>
          </CardContent>
        </Card>

        <Card class="w-80 flex-none" shadows={null}>
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
                <Text class="text-xs text-muted">Inspect native bounds</Text>
              </View>
              <Switch
                checked={inspectLayout()}
                aria-label="Layout overlay"
                onCheckedChange={setInspectLayout}
              />
            </View>
            <View class="flex flex-col gap-3">
              <Capability icon={layers} text="Retained Vello scene" />
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
      <Text class="font-mono text-xs font-medium text-primary">
        {props.value}
      </Text>
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
