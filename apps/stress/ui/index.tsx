// Emoji stress UI — N emojis bouncing around the viewport. Stresses the text /
// glyph/layout path and the animation path
// (per-frame transforms for every emoji → op protocol + apply + native paint).
// The selectable rAF and Motion drivers exercise the same render path, making
// animation scheduler overhead directly comparable. Bounce bounds come from
// host diagnostics (so the app self-sizes without a window-size API).

import "@wabou/ui";
import "virtual:wabou-stylesheet";
import {
  type AnimationControls,
  animate,
  PrimitiveButton as Button,
  createFps,
  type Handle,
  mount,
  setTransform2D,
  Text,
  translate2d,
  useHost,
  View,
} from "@wabou/ui";
import { createEffect, createSignal, For as ForValue, Show } from "solid-js";

const PRESETS = [100, 1_000, 5_000, 10_000, 25_000];
const JS_WORK = [0, 50_000, 500_000, 5_000_000];
const CHARS = [
  "🚀",
  "🔥",
  "✨",
  "🎨",
  "⚡",
  "🌟",
  "🎲",
  "🦀",
  "🐙",
  "🌈",
  "🍇",
  "🎈",
];
const SIZE = 28;
const HEADER_H = 120;
let _jsSink = 0;

type Stats = {
  build_frame_ms: number;
  js_tick_ms: number;
  scene_ms: number;
  present_ms: number;
  node_count: number;
  viewport_w: number;
  viewport_h: number;
} | null;

type Driver = "raf" | "motion";

const fmt = (ms: number) => ms.toFixed(1);
const rand = (a: number, b: number) => a + Math.random() * (b - a);

interface Body {
  x: number;
  y: number;
  dx: number;
  dy: number;
  char: string;
}

function makeBodies(n: number): Body[] {
  const arr: Body[] = [];
  for (let i = 0; i < n; i++) {
    arr.push({
      x: rand(0, 760),
      y: rand(0, 520),
      dx: rand(-3, 3) || 2,
      dy: rand(-3, 3) || 2,
      char: CHARS[i % CHARS.length],
    });
  }
  return arr;
}

function moveBody(b: Body, w: number, h: number): void {
  let { x, y, dx, dy } = b;
  x += dx;
  y += dy;
  const maxX = Math.max(1, w - SIZE);
  const maxY = Math.max(1, h - SIZE);
  if (x <= 0) {
    x = 0;
    dx = Math.abs(dx);
  } else if (x >= maxX) {
    x = maxX;
    dx = -Math.abs(dx);
  }
  if (y <= 0) {
    y = 0;
    dy = Math.abs(dy);
  } else if (y >= maxY) {
    y = maxY;
    dy = -Math.abs(dy);
  }
  b.x = x;
  b.y = y;
  b.dx = dx;
  b.dy = dy;
}

function App() {
  const host = useHost();
  const [n, setN] = createSignal(1_000);
  const [jsWork, setJsWork] = createSignal<number>(0, {});
  const [driver, setDriver] = createSignal<Driver>("raf");
  const [stats, setStats] = createSignal<Stats>(null);
  const [statsError, setStatsError] = createSignal<string | null>(null, {});
  const [bodies, setBodies] = createSignal<Body[]>(makeBodies(1_000));
  let movingBodies = bodies();
  const handles: Array<Handle | undefined> = [];
  const fps = createFps();
  let lastStatsSample = -Infinity;

  // Re-seed only when N changes (not every frame).
  createEffect(n, (count) => {
    movingBodies = makeBodies(count);
    // `<Index>` retains handles for existing slots. Replacing this array when
    // the initial effect runs would discard refs without remounting the nodes,
    // leaving the first render frozen until N changes.
    handles.length = movingBodies.length;
    setBodies(movingBodies);
  });

  const renderFrame = () => {
    const s = stats();
    const w = s?.viewport_w ?? 800;
    const h = (s?.viewport_h ?? 600) - HEADER_H;
    for (let i = 0; i < movingBodies.length; i++) {
      const body = movingBodies[i];
      moveBody(body, w, h);
      const handle = handles[i];
      if (handle) setTransform2D(handle, translate2d(body.x, body.y));
    }
    const iters = jsWork();
    if (iters > 0) {
      let acc = 0;
      for (let i = 0; i < iters; i++) acc += Math.sin(i);
      _jsSink = acc;
    }
    const now = performance.now();
    if (now - lastStatsSample >= 500) {
      lastStatsSample = now;
      try {
        setStats(host.diagnostics.frameStats());
        setStatsError(null);
      } catch (error) {
        setStatsError(error instanceof Error ? error.message : String(error));
      }
    }
  };
  createEffect(driver, (activeDriver) => {
    let raf = 0;
    let motion: AnimationControls | undefined;
    if (activeDriver === "motion") {
      motion = animate(0, 1, {
        duration: 1,
        ease: "linear",
        repeat: Infinity,
        onUpdate: renderFrame,
      });
    } else {
      const loop = () => {
        renderFrame();
        raf = requestAnimationFrame(loop);
      };
      raf = requestAnimationFrame(loop);
    }
    return () => {
      if (raf) cancelAnimationFrame(raf);
      motion?.stop();
    };
  });

  return (
    <View class="w-full h-full flex flex-col bg-canvas text-primary">
      <View class="h-9 flex-none px-3 flex items-center gap-3 border-b border-subtle bg-surface">
        <Text class="text-sm font-semibold text-primary">Emoji stress</Text>
        <Text class="text-xs text-muted">
          GPUI retained projection · per-node transform updates
        </Text>
      </View>

      <View class="h-11 flex-none min-w-0 flex border-b border-subtle bg-surface-muted">
        <View class="w-28 flex-none px-3 justify-center border-r border-subtle">
          <Text class="text-[10px] text-muted">FPS</Text>
          <Text class="text-base font-mono font-semibold text-accent">
            {fps().toString()}
          </Text>
        </View>
        <View class="w-24 flex-none px-3 justify-center border-r border-subtle">
          <Text class="text-[10px] text-muted">NODES</Text>
          <Text class="text-sm font-mono text-primary">
            {n().toLocaleString()}
          </Text>
        </View>
        <View class="w-24 flex-none px-3 justify-center border-r border-subtle">
          <Text class="text-[10px] text-muted">DRIVER</Text>
          <Text class="text-sm font-mono text-primary">{driver()}</Text>
        </View>
        <Show
          when={stats()}
          fallback={
            <View class="min-w-0 flex-1 px-3 justify-center">
              <Text class={statsError() ? "text-danger-primary" : "text-muted"}>
                {statsError() ?? "Waiting for the first completed frame"}
              </Text>
            </View>
          }
        >
          {(current) => (
            <>
              <View class="w-24 flex-none px-3 justify-center border-r border-subtle">
                <Text class="text-[10px] text-muted">JS TICK</Text>
                <Text class="text-sm font-mono text-primary">{`${fmt(current().js_tick_ms)} ms`}</Text>
              </View>
              <View class="w-24 flex-none px-3 justify-center border-r border-subtle">
                <Text class="text-[10px] text-muted">BUILD</Text>
                <Text class="text-sm font-mono text-primary">{`${fmt(current().build_frame_ms)} ms`}</Text>
              </View>
              <View class="w-24 flex-none px-3 justify-center border-r border-subtle">
                <Text class="text-[10px] text-muted">SCENE</Text>
                <Text class="text-sm font-mono text-primary">{`${fmt(current().scene_ms)} ms`}</Text>
              </View>
              <View class="w-24 flex-none px-3 justify-center">
                <Text class="text-[10px] text-muted">PRESENT</Text>
                <Text class="text-sm font-mono text-primary">{`${fmt(current().present_ms)} ms`}</Text>
              </View>
            </>
          )}
        </Show>
      </View>

      <View class="h-10 flex-none min-w-0 overflow-x-auto overflow-y-hidden px-2 flex items-center gap-2 border-b border-subtle bg-surface">
        <ForValue each={PRESETS}>
          {(p) => (
            <Button
              class="px-2 py-1 text-xs rounded border border-slate-600"
              tone="sky"
              selected={n() === p}
              aria-label={`${p} emojis`}
              onClick={() => setN(p)}
            >
              {p.toLocaleString()}
            </Button>
          )}
        </ForValue>
        <Text class="ml-1 text-xs text-muted">emojis</Text>
        <Text class="mx-2 text-muted">|</Text>
        <ForValue each={JS_WORK}>
          {(w) => (
            <Button
              class="px-2 py-1 text-xs rounded border border-slate-600"
              tone="amber"
              selected={jsWork() === w}
              onClick={() => setJsWork(w)}
            >
              {w === 0 ? "0" : w.toLocaleString()}
            </Button>
          )}
        </ForValue>
        <Text class="ml-1 text-xs text-muted">js iters</Text>
        <Text class="mx-2 text-muted">|</Text>
        <Button
          class="px-2 py-1 text-xs rounded border border-slate-600"
          tone="sky"
          selected={driver() === "raf"}
          onClick={() => setDriver("raf")}
        >
          rAF baseline
        </Button>
        <Button
          class="px-2 py-1 text-xs rounded border border-slate-600"
          tone="sky"
          selected={driver() === "motion"}
          onClick={() => setDriver("motion")}
        >
          Motion
        </Button>
      </View>

      <View class="flex-1 min-h-0 overflow-hidden relative">
        <ForValue each={bodies()} keyed={false}>
          {(body, index) => (
            <View
              class="text-[28px] pointer-events-none"
              ref={(handle) => {
                handles[index] = handle as unknown as Handle;
              }}
              style={{
                position: "absolute",
                top: 0,
                left: 0,
                "line-height": "1",
              }}
            >
              {body().char}
            </View>
          )}
        </ForValue>
      </View>
    </View>
  );
}

mount(() => <App />);
