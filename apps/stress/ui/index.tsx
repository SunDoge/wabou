// Emoji stress UI — N emojis bouncing around the viewport. Stresses the text /
// glyph path (parley + vello — the real bottleneck) AND the animation path
// (per-frame transforms for every emoji → op protocol + apply + native paint).
// The selectable rAF and Motion drivers exercise the same render path, making
// animation scheduler overhead directly comparable. Bounce bounds come from
// host diagnostics (so the app self-sizes without a window-size API).

import "@wabou/core";
import "virtual:wabou-stylesheet";
import { type AnimationControls, animate } from "@wabou/animation";
import { Button, Text, translate2d } from "@wabou/primitives";
import {
  type Handle,
  mount,
  setTransform2D,
  createFps,
  useHost,
} from "@wabou/core";
import {
  createEffect,
  createSignal,
  For,
  Index,
  onCleanup,
  Show,
} from "solid-js";

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
const HEADER_H = 68;
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

function moveBody(b: Body, w: number, h: number): Body {
  let { x, y, dx, dy, char } = b;
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
  return { x, y, dx, dy, char };
}

function App() {
  const host = useHost();
  const [n, setN] = createSignal(1_000);
  const [jsWork, setJsWork] = createSignal(0);
  const [driver, setDriver] = createSignal<Driver>("raf");
  const [stats, setStats] = createSignal<Stats>(null);
  const [statsError, setStatsError] = createSignal<string | null>(null);
  const [bodies, setBodies] = createSignal<Body[]>(makeBodies(1_000));
  let movingBodies = bodies();
  const handles: Array<Handle | undefined> = [];
  const fps = createFps();

  // Re-seed only when N changes (not every frame).
  createEffect(() => {
    movingBodies = makeBodies(n());
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
      const body = moveBody(movingBodies[i], w, h);
      movingBodies[i] = body;
      const handle = handles[i];
      if (handle) setTransform2D(handle, translate2d(body.x, body.y));
    }
    const iters = jsWork();
    if (iters > 0) {
      let acc = 0;
      for (let i = 0; i < iters; i++) acc += Math.sin(i);
      _jsSink = acc;
    }
    try {
      setStats(host.diagnostics.frameStats());
      setStatsError(null);
    } catch (error) {
      setStatsError(error instanceof Error ? error.message : String(error));
    }
  };
  createEffect(() => {
    const activeDriver = driver();
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
    onCleanup(() => {
      if (raf) cancelAnimationFrame(raf);
      motion?.stop();
    });
  });

  return (
    <div class="w-full h-full flex flex-col bg-slate-950 text-slate-100">
      <div class="flex-none p-2 flex items-center gap-2 border-b border-slate-700">
        <Text class="text-sm font-semibold mr-1">emoji stress</Text>
        <For each={PRESETS}>
          {(p) => (
            <Button
              class="px-2 py-1 text-xs rounded border border-slate-600"
              tone="sky"
              selected={n() === p}
              onClick={() => setN(p)}
            >
              {p.toLocaleString()}
            </Button>
          )}
        </For>
        <Text class="ml-1 text-xs text-slate-500">emojis</Text>
        <Text class="mx-2 text-slate-600">|</Text>
        <For each={JS_WORK}>
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
        </For>
        <Text class="ml-1 text-xs text-slate-500">js iters</Text>
        <Text class="mx-2 text-slate-600">|</Text>
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
        <Text class="ml-auto text-xs font-mono text-slate-400">
          {n().toLocaleString()}n · {driver()} · {fps()} fps
        </Text>
      </div>

      <div class="flex-none px-2 py-1 border-b border-slate-800 bg-slate-900 text-xs font-mono text-slate-300">
        <Show
          when={stats()}
          fallback={
            <Text class={statsError() ? "text-red-400" : "text-slate-500"}>
              {statsError()
                ? `frameStats error: ${statsError()}`
                : "frameStats: waiting for first completed frame"}
            </Text>
          }
        >
          {(current) => (
            <Text>
              js {fmt(current().js_tick_ms)}ms · build{" "}
              {fmt(current().build_frame_ms)}ms
              {" · "}scene {fmt(current().scene_ms)}ms · present{" "}
              {fmt(current().present_ms)}ms · nodes{" "}
              {current().node_count.toLocaleString()}
              {" · "}viewport {current().viewport_w}×{current().viewport_h}
            </Text>
          )}
        </Show>
      </div>

      <div class="flex-1 min-h-0 overflow-hidden relative">
        <Index each={bodies()}>
          {(body, index) => (
            <div
              class="text-[28px]"
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
            </div>
          )}
        </Index>
      </div>
    </div>
  );
}

mount(() => <App />);
