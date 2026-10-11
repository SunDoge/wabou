import { Button as ButtonPrimitive } from "@wabou/ui/primitives";
import {
  Center,
  Icon,
  ScrollArea,
  Svg,
  Text,
  TitleBar,
  TitleBarDragRegion,
  View,
  createMeasuredSize,
  useWindow,
} from "@wabou/ui";
import { createSignal, For as ForValue } from "solid-js";
import cpu from "lucide-static/icons/cpu.svg?raw";
import memory from "lucide-static/icons/memory-stick.svg?raw";
import gpu from "lucide-static/icons/box.svg?raw";
import disk from "lucide-static/icons/hard-drive.svg?raw";
import network from "lucide-static/icons/globe.svg?raw";
import battery from "lucide-static/icons/battery.svg?raw";
import grid from "lucide-static/icons/layout-grid.svg?raw";
import list from "lucide-static/icons/list.svg?raw";
import laptop from "lucide-static/icons/laptop.svg?raw";
import pause from "lucide-static/icons/pause.svg?raw";
import play from "lucide-static/icons/play.svg?raw";
import closeIcon from "lucide-static/icons/x.svg?raw";
import minusIcon from "lucide-static/icons/minus.svg?raw";
import maximizeIcon from "lucide-static/icons/maximize-2.svg?raw";

const palette = {
  ink: "#171719",
  muted: "#8d8e93",
  surface: "#f4f4f6",
  blue: "#008df2",
  green: "#2bab4a",
  pink: "#ff4081",
  orange: "#ed9108",
  teal: "#19a1a8",
  yellow: "#d4a008",
};
const metrics = [
  {
    name: "CPU",
    icon: cpu,
    color: palette.blue,
    caption: "Now",
    value: "59",
    unit: "%",
    stats: [
      ["User", "37%"],
      ["System", "22%"],
      ["Temperature", "50 °C"],
    ],
    nav: "59%",
  },
  {
    name: "Memory",
    icon: memory,
    color: palette.green,
    caption: "In use of 32.0 GB",
    value: "27.4",
    unit: "GB",
    stats: [
      ["App", "7.98 GB"],
      ["Wired", "5.26 GB"],
      ["Compressed", "14.1 GB"],
    ],
    nav: "85%",
  },
  {
    name: "GPU",
    icon: gpu,
    color: palette.pink,
    caption: "Apple M5",
    value: "8.1",
    unit: "%",
    stats: [
      ["Memory", "1.11 GB"],
      ["Average", "16%"],
      ["Peak", "48%"],
    ],
    nav: "8.1%",
  },
  {
    name: "Disk",
    icon: disk,
    color: palette.orange,
    caption: "Free of 995 GB",
    value: "408",
    unit: "GB",
    stats: [
      ["Reading", "310 KB/s"],
      ["Writing", "210 KB/s"],
      ["Written", "1.01 TB"],
    ],
    nav: "310 KB/s",
  },
  {
    name: "Network",
    icon: network,
    color: palette.teal,
    caption: "Downloading",
    value: "2.39",
    unit: "MB/s",
    stats: [
      ["Uploading", "13.7 KB/s"],
      ["Received", "198 GB"],
      ["Sent", "24.7 GB"],
    ],
    nav: "2.41 MB/s",
  },
  {
    name: "Battery",
    icon: battery,
    color: palette.green,
    caption: "On Power Adapter",
    value: "80",
    unit: "%",
    stats: [
      ["Remaining", "On power"],
      ["Power Draw", "14.3 W"],
      ["Health", "99%"],
    ],
    nav: "80%",
  },
] as const;
type Metric = (typeof metrics)[number];

export function PulseWindowControls() {
  const window = useWindow();
  const controls = [
    {
      name: "Close window",
      color: "#ff6058",
      icon: closeIcon,
      run: () => window.close(),
    },
    {
      name: "Minimize window",
      color: "#ffbd2e",
      icon: minusIcon,
      run: () => window.minimize(),
    },
    {
      name: "Toggle window maximized",
      color: "#28c840",
      icon: maximizeIcon,
      run: () => window.setMaximized(!window.maximized()),
    },
  ];
  return (
    <TitleBar
      aria-label="Pulse window title bar"
      class="border-0"
      style={{ height: "28px", "margin-bottom": "10px" }}
    >
      <View class="flex flex-row flex-none" style={{ "padding-left": "4px" }}>
        <ForValue each={controls}>
          {(control) => (
            <ButtonPrimitive
              unstyled
              aria-label={control.name}
              onClick={control.run}
              class="items-center justify-center rounded-full"
              style={{ width: "24px", height: "28px" }}
              renderContent={(state) => (
                <Center
                  class="rounded-full"
                  style={{
                    width: "14px",
                    height: "14px",
                    "background-color": control.color,
                    "border-width": "0.5px",
                    "border-color": state().focusVisible
                      ? "#2257d6"
                      : "#00000020",
                  }}
                >
                  <Icon
                    source={control.icon}
                    size={8}
                    style={{
                      color: "#512a22",
                      opacity:
                        state().hovered || state().focusVisible ? 0.8 : 0,
                    }}
                  />
                </Center>
              )}
            />
          )}
        </ForValue>
      </View>
      <TitleBarDragRegion role="group" aria-label="Move Pulse window" />
    </TitleBar>
  );
}

function Glyph(props: { source: string; color: string; soft?: boolean }) {
  return (
    <Center
      aria-label="Pulse icon frame"
      class="flex-none rounded-lg"
      style={{
        width: "26px",
        height: "26px",
        "background-color": props.soft ? `${props.color}22` : props.color,
        color: props.soft ? props.color : "#ffffff",
      }}
    >
      <Icon source={props.source} size={16} />
    </Center>
  );
}

/** A single vector primitive keeps the dot history out of the retained node tree. */
function DotHistory(props: { metric: Metric; interval: string }) {
  const size = createMeasuredSize();
  const source = () => {
    const width = size.measured() ? size.width() : 277;
    const colors =
      props.metric.name === "Memory"
        ? [palette.green, palette.teal, "#e1b548"]
        : props.metric.name === "CPU"
          ? [palette.blue, "#fa7382"]
          : [props.metric.color];
    const circles: string[] = [];
    for (let x = 0; x < 40; x++)
      for (let y = 0; y < 5; y++) {
        const phase =
          props.interval === "1m"
            ? 0
            : props.interval === "5m"
              ? 7
              : props.interval === "15m"
                ? 13
                : 19;
        let height = 1 + Math.round((Math.sin((x + phase) * 0.22) + 1) * 0.7);
        if (props.metric.name === "Battery") height = x < 28 ? 4 : 3;
        if (props.metric.name === "GPU") height = 1;
        if (props.metric.name === "Memory") height = 4;
        if (props.metric.name === "Disk") height = x > 10 && x < 16 ? 4 : 1;
        if (props.metric.name === "CPU") height = x < 35 ? 2 : 3;
        const active = y >= 5 - height;
        const color =
          colors[
            props.metric.name === "CPU"
              ? y === 4
                ? 1
                : 0
              : props.metric.name === "Memory"
                ? y === 4
                  ? 2
                  : y === 3
                    ? 1
                    : 0
                : 0
          ] ?? colors[0];
        circles.push(
          `<circle cx="${11 + (x * (width - 22)) / 39}" cy="${16 + y * 6.5}" r="2.45" fill="${active ? color : props.metric.color}" opacity="${active ? 0.94 : 0.12}"/>`,
        );
      }
    return `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="54" viewBox="0 0 ${width} 54">${circles.join("")}</svg>`;
  };
  return (
    <View
      ref={size.ref}
      class="w-full rounded-xl overflow-hidden"
      style={{ height: "54px", "background-color": `${props.metric.color}12` }}
    >
      <Svg
        source={source()}
        class="w-full h-full"
        aria-label={`${props.metric.name} sample history`}
      />
    </View>
  );
}

function MetricCard(props: {
  metric: Metric;
  interval: string;
  width: number;
}) {
  const m = props.metric;
  return (
    <View
      role="group"
      aria-label={`${m.name} metric`}
      class="min-w-0 flex flex-col"
      style={{
        width: `${props.width}px`,
        padding: "18px",
        height: "250px",
        "border-radius": "18px",
        "background-color": palette.surface,
      }}
    >
      <View class="flex flex-row items-center gap-3">
        <Glyph source={m.icon} color={m.color} soft />
        <Text
          class="font-semibold"
          style={{ "font-size": "17px", color: m.color }}
        >
          {m.name}
        </Text>
      </View>
      <Text
        style={{
          "font-size": "14px",
          color: palette.muted,
          "margin-top": "14px",
        }}
      >
        {m.caption}
      </Text>
      <View
        class="flex flex-row items-baseline gap-1"
        style={{ height: "53px" }}
      >
        <Text
          class="font-bold"
          style={{
            "font-size": "40px",
            "line-height": "48px",
            color: palette.ink,
          }}
        >
          {m.value}
        </Text>
        <Text
          class="font-semibold"
          style={{ "font-size": "17px", color: palette.muted }}
        >
          {m.unit}
        </Text>
        {m.name === "Memory" && (
          <View
            class="ml-auto rounded-full px-2 py-1"
            style={{ "background-color": "#faead8", "margin-top": "12px" }}
          >
            <Text
              class="font-semibold"
              style={{ "font-size": "11px", color: palette.orange }}
            >
              ! Elevated
            </Text>
          </View>
        )}
      </View>
      <View
        class="flex flex-row justify-between"
        style={{ "margin-top": "6px", "margin-bottom": "7px" }}
      >
        <ForValue each={m.stats}>
          {([label, value]) => (
            <View class="min-w-0 gap-1">
              <Text style={{ color: palette.muted, "font-size": "14px" }}>
                {label}
              </Text>
              <Text
                class="font-semibold"
                style={{ color: palette.ink, "font-size": "15px" }}
              >
                {value}
              </Text>
            </View>
          )}
        </ForValue>
      </View>
      <View class="mt-auto">
        <DotHistory metric={m} interval={props.interval} />
      </View>
    </View>
  );
}

function BreakdownCard(props: {
  title: string;
  icon: string;
  color: string;
  width: number;
  children: import("solid-js").JSX.Element;
}) {
  return (
    <View
      role="group"
      aria-label={props.title}
      class="rounded-2xl min-w-0 flex flex-col"
      style={{
        width: `${props.width}px`,
        height: "238px",
        padding: "18px",
        "background-color": palette.surface,
      }}
    >
      <View class="flex flex-row items-center gap-3">
        <Glyph source={props.icon} color={props.color} soft />
        <Text
          class="font-semibold"
          style={{ color: props.color, "font-size": "16px" }}
        >
          {props.title}
        </Text>
      </View>
      <View class="flex flex-col flex-1 min-h-0 justify-center">
        {props.children}
      </View>
    </View>
  );
}

function MemoryBreakdown() {
  const source = `<svg xmlns="http://www.w3.org/2000/svg" width="112" height="112" viewBox="0 0 112 112"><circle cx="56" cy="56" r="43" fill="none" stroke="#e4e4e8" stroke-width="11"/>${[
    [palette.green, 0, 78],
    ["#e1a617", 81, 39],
    ["#36acd3", 123, 111],
    ["#999ba5", 237, 27],
  ]
    .map(
      ([color, offset, length]) =>
        `<circle cx="56" cy="56" r="43" fill="none" stroke="${color}" stroke-width="11" stroke-linecap="round" stroke-dasharray="${length} 270" stroke-dashoffset="-${offset}" transform="rotate(-90 56 56)"/>`,
    )
    .join("")}</svg>`;
  return (
    <View class="flex flex-row items-center gap-3">
      <View
        class="relative flex flex-col flex-none items-center justify-center"
        style={{ width: "112px", height: "112px" }}
      >
        <Svg source={source} class="absolute inset-0 w-full h-full" />
        <Text
          class="font-bold"
          style={{ "font-size": "24px", color: palette.ink }}
        >
          85%
        </Text>
        <Text style={{ "font-size": "11px", color: palette.muted }}>
          in use
        </Text>
      </View>
      <View class="flex flex-col flex-1 gap-2">
        <ForValue
          each={[
            ["App", "7.98 GB", palette.green],
            ["Wired", "5.26 GB", "#e1a617"],
            ["Compressed", "14.1 GB", "#36acd3"],
            ["Cached", "4.69 GB", "#999ba5"],
            ["Free", "4.62 GB", "#e4e4e8"],
          ]}
        >
          {([name, amount, color]) => (
            <View class="flex flex-row items-center gap-2">
              <View
                class="rounded-full flex-none"
                style={{
                  width: "7px",
                  height: "7px",
                  "background-color": color,
                }}
              />
              <Text style={{ "font-size": "12px", color: palette.ink }}>
                {name}
              </Text>
              <Text
                class="ml-auto"
                style={{ "font-size": "12px", color: palette.muted }}
              >
                {amount}
              </Text>
            </View>
          )}
        </ForValue>
      </View>
    </View>
  );
}

export function PulseOverview() {
  const [interval, setInterval] = createSignal("1m");
  const [paused, setPaused] = createSignal(false);
  const size = createMeasuredSize();
  const columns = () => (size.measured() && size.width() < 980 ? 2 : 3);
  const cardWidth = () =>
    ((size.measured() ? size.width() : 1240) - 268 - 16 * (columns() - 1)) /
    columns();
  return (
    <View
      ref={size.ref}
      aria-label="Pulse reference overview"
      class="w-full h-full flex flex-row"
      style={{
        "background-color": "#ffffff",
        color: palette.ink,
        "font-family": "Inter, sans-serif",
      }}
    >
      <View
        role="group"
        aria-label="Sample system summary"
        class="h-full flex-none flex flex-col"
        style={{
          width: "220px",
          "padding-top": "14px",
          "padding-bottom": "14px",
          "padding-left": "10px",
          "padding-right": "10px",
          "background-color": palette.surface,
        }}
      >
        <PulseWindowControls />
        <View
          class="flex flex-row items-center gap-2 rounded-lg px-2"
          style={{
            height: "29px",
            "background-color": "#2860ef",
            "margin-top": "4px",
            "margin-bottom": "4px",
          }}
        >
          <Glyph source={grid} color="#707682" />
          <Text style={{ color: "white", "font-size": "14px" }}>Overview</Text>
        </View>
        <View class="flex flex-row items-center gap-2 px-2">
          <Glyph source={list} color="#909196" />
          <Text style={{ "font-size": "14px" }}>Processes</Text>
          <Text
            class="ml-auto"
            style={{ color: palette.muted, "font-size": "12px" }}
          >
            24
          </Text>
        </View>
        <Text
          class="font-semibold px-2"
          style={{
            "font-size": "11px",
            color: "#aaaab0",
            "margin-top": "20px",
            "margin-bottom": "8px",
          }}
        >
          Hardware
        </Text>
        <ForValue each={metrics}>
          {(m) => (
            <View
              class="flex flex-row items-center gap-2 px-2"
              style={{ height: "29px", "margin-bottom": "2px" }}
            >
              <Glyph source={m.icon} color={m.color} />
              <Text style={{ "font-size": "14px" }}>
                {m.name === "Battery" ? "Energy" : m.name}
              </Text>
              <Text
                class="ml-auto"
                style={{ "font-size": "12px", color: palette.muted }}
              >
                {m.nav}
              </Text>
            </View>
          )}
        </ForValue>
        <Text
          class="font-semibold px-2"
          style={{
            "font-size": "11px",
            color: "#aaaab0",
            "margin-top": "18px",
            "margin-bottom": "8px",
          }}
        >
          System
        </Text>
        <View class="flex flex-row items-center gap-2 px-2">
          <Glyph source={laptop} color="#909196" />
          <Text style={{ "font-size": "14px" }}>About This Mac</Text>
        </View>
        <View class="mt-auto px-2 gap-1">
          <Text class="font-semibold" style={{ "font-size": "13px" }}>
            MacBook Air
          </Text>
          <Text style={{ "font-size": "11px", color: palette.muted }}>
            Apple M5 · Sample data
          </Text>
        </View>
      </View>
      <View
        class="flex-1 min-w-0 h-full flex flex-col"
        style={{
          "padding-top": "12px",
          "padding-left": "24px",
          "padding-right": "24px",
        }}
      >
        <View
          class="flex-none flex flex-row items-center"
          style={{ height: "42px", "margin-bottom": "0px" }}
        >
          <View>
            <Text class="font-bold" style={{ "font-size": "17px" }}>
              Overview
            </Text>
            <Text style={{ "font-size": "12px", color: palette.muted }}>
              MacBook Air
            </Text>
          </View>
          <View class="ml-auto flex flex-row items-center gap-2">
            <View
              class="flex flex-row rounded-full p-1"
              style={{ "background-color": "#f1f1f3" }}
            >
              <ForValue each={["1m", "5m", "15m", "1h"]}>
                {(value) => (
                  <ButtonPrimitive
                    unstyled
                    aria-label={`History ${value}`}
                    aria-pressed={interval() === value}
                    onClick={() => setInterval(value)}
                    class={(state) =>
                      `rounded-full items-center justify-center border ${state.focusVisible ? "border-blue-500" : "border-transparent"}`
                    }
                    style={{
                      width: "43px",
                      height: "26px",
                      "background-color": "transparent",
                    }}
                  >
                    <Center
                      class="w-full h-full rounded-full"
                      style={{
                        "background-color":
                          interval() === value ? "white" : "transparent",
                      }}
                      shadows={
                        interval() === value
                          ? [
                              {
                                offsetX: 0,
                                offsetY: 1,
                                spread: 0,
                                stdDev: 1,
                                color: 0x0000001a,
                              },
                              {
                                offsetX: 0,
                                offsetY: 0,
                                spread: 0.5,
                                stdDev: 0,
                                color: 0x00000008,
                              },
                            ]
                          : []
                      }
                    >
                      <Text
                        class="font-semibold"
                        style={{
                          "font-size": "12px",
                          color:
                            interval() === value ? palette.ink : palette.muted,
                        }}
                      >
                        {value}
                      </Text>
                    </Center>
                  </ButtonPrimitive>
                )}
              </ForValue>
            </View>
            <ButtonPrimitive
              unstyled
              aria-label={
                paused() ? "Resume sample history" : "Pause sample history"
              }
              aria-pressed={paused()}
              onClick={() => setPaused(!paused())}
              class="rounded-full items-center justify-center"
              style={(state) => ({
                width: "38px",
                height: "34px",
                "background-color": state.hovered ? "#e7e7ea" : "#f1f1f3",
              })}
            >
              <Icon source={paused() ? play : pause} size={15} />
            </ButtonPrimitive>
          </View>
        </View>
        <ScrollArea class="flex-1 min-h-0" aria-label="System metrics">
          <View
            style={{
              display: "flex",
              "flex-direction": "row",
              "flex-wrap": "wrap",
              gap: "16px",
              "padding-bottom": "16px",
            }}
          >
            <ForValue each={metrics}>
              {(metric) => (
                <MetricCard
                  width={cardWidth()}
                  metric={metric}
                  interval={interval()}
                />
              )}
            </ForValue>
            <BreakdownCard
              width={cardWidth()}
              title="Memory by Type"
              icon={memory}
              color={palette.green}
            >
              <MemoryBreakdown />
            </BreakdownCard>
            <BreakdownCard
              width={cardWidth()}
              title="Memory by App"
              icon={grid}
              color={palette.green}
            >
              <View class="flex flex-row gap-2" style={{ height: "146px" }}>
                <View
                  class="flex flex-col flex-1 rounded-xl justify-end p-3"
                  style={{ "background-color": "white" }}
                >
                  <Text class="font-semibold" style={{ "font-size": "15px" }}>
                    Xcode
                  </Text>
                  <Text style={{ color: palette.muted, "font-size": "12px" }}>
                    3.40 GB
                  </Text>
                </View>
                <View class="flex flex-col flex-1 gap-2">
                  <ForValue
                    each={[
                      ["Safari", "1.78 GB"],
                      ["Photos", "1.11 GB"],
                    ]}
                  >
                    {([name, value]) => (
                      <View
                        class="flex flex-col flex-1 rounded-xl justify-end p-3"
                        style={{ "background-color": "white" }}
                      >
                        <Text
                          class="font-semibold"
                          style={{ "font-size": "13px" }}
                        >
                          {name}
                        </Text>
                        <Text
                          style={{ color: palette.muted, "font-size": "12px" }}
                        >
                          {value}
                        </Text>
                      </View>
                    )}
                  </ForValue>
                </View>
              </View>
            </BreakdownCard>
            <BreakdownCard
              width={cardWidth()}
              title="Energy by App"
              icon={battery}
              color={palette.yellow}
            >
              <View class="gap-3">
                <ForValue
                  each={[
                    ["Xcode", "2.60 W", "95%"],
                    ["Safari", "1.12 W", "43%"],
                    ["node", "900 mW", "34%"],
                    ["Photos", "380 mW", "15%"],
                  ]}
                >
                  {([name, value, width]) => (
                    <View class="gap-1">
                      <View class="flex flex-row">
                        <Text style={{ "font-size": "12px" }}>{name}</Text>
                        <Text
                          class="ml-auto"
                          style={{ "font-size": "12px", color: palette.muted }}
                        >
                          {value}
                        </Text>
                      </View>
                      <View
                        class="h-1 rounded-full"
                        style={{ "background-color": "#ece5cc" }}
                      >
                        <View
                          class="h-full rounded-full"
                          style={{ width, "background-color": palette.yellow }}
                        />
                      </View>
                    </View>
                  )}
                </ForValue>
              </View>
            </BreakdownCard>
          </View>
        </ScrollArea>
      </View>
    </View>
  );
}
