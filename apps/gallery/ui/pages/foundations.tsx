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

export { ColorsPage, ShadowsPage };
