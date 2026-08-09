// Wabou DevTools UI entry.
import "@wabou/core";
import "virtual:wabou-stylesheet";
import { Button, Popover, Text } from "@wabou/primitives";
import { mount, useHost } from "@wabou/solid-renderer";
import {
  createEffect,
  createMemo,
  createSignal,
  For,
  Index,
  type JSX,
  onCleanup,
  onMount,
  Show,
} from "solid-js";
import {
  createDevtoolsClient,
  type DebugClip,
  type DebugFrame,
  type DebugNode,
  type DebugStatus,
} from "./generated/host-bindings";
import { overlayStyle } from "./model";

function shortText(node: DebugNode): string {
  const text = node.text?.replaceAll("\n", " ").trim();
  return text
    ? text.slice(0, 52)
    : node.classes
        .slice(0, 2)
        .map((c) => `.${c}`)
        .join("");
}

function clipLabel(clip: DebugClip): string {
  const rect = clip.rect;
  return `${clip.coordinateSpace} · ${rect.x}, ${rect.y} · ${rect.width}×${rect.height} · r${clip.radius} · [${clip.transform?.join(", ") ?? "1, 0, 0, 1, 0, 0"}]`;
}

function App() {
  const devtools = createDevtoolsClient(useHost());
  const [status, setStatus] = createSignal<DebugStatus>();
  const [nodes, setNodes] = createSignal<DebugNode[]>([]);
  const [selected, setSelected] = createSignal<DebugNode>();
  const [frames, setFrames] = createSignal<DebugFrame[]>([]);
  const [query, setQuery] = createSignal("");
  const [socket, setSocket] = createSignal("");
  const [connectedSocket, setConnectedSocket] = createSignal<string>();
  const [screenshot, setScreenshot] = createSignal<string>();
  const [error, setError] = createSignal<string>();
  const [busy, setBusy] = createSignal(false);
  const [layoutOverlay, setLayoutOverlay] = createSignal(false);

  const selectedRect = createMemo(() => {
    const node = selected();
    const current = status();
    if (!node || !current?.viewportWidth || !current.viewportHeight)
      return undefined;
    return overlayStyle(
      node.rect,
      current.viewportWidth,
      current.viewportHeight,
    );
  });

  async function refreshStatus(): Promise<void> {
    try {
      const value = await devtools.status();
      setStatus(value);
      setError(undefined);
    } catch (cause) {
      setError(String(cause));
    }
  }

  async function refreshNodes(): Promise<void> {
    try {
      const value = await devtools.queryNodes(query(), 150);
      setNodes(value);
      const id = selected()?.id;
      if (id) {
        const next = value.find((node) => node.id === id);
        if (next) setSelected(next);
      }
      setError(undefined);
    } catch (cause) {
      setError(String(cause));
    }
  }

  async function refreshFrames(): Promise<void> {
    try {
      const next = await devtools.recentFrames(20);
      const current = frames();
      if (
        current.length === next.length &&
        current.every(
          (frame, index) =>
            frame.direction === next[index]?.direction &&
            frame.sequence === next[index]?.sequence,
        )
      ) {
        return;
      }
      setFrames(next);
    } catch (cause) {
      setError(String(cause));
    }
  }

  async function refreshAll(): Promise<void> {
    if (busy()) return;
    setBusy(true);
    await Promise.all([refreshStatus(), refreshNodes(), refreshFrames()]);
    setBusy(false);
  }

  async function inspect(id: number): Promise<void> {
    try {
      const node = await devtools.inspectNode(id);
      setSelected(node);
      if (layoutOverlay()) {
        await devtools.setOverlay(true, true, true, node.id);
      }
      setError(undefined);
    } catch (cause) {
      setError(String(cause));
    }
  }

  async function connect(): Promise<void> {
    if (!socket().trim()) {
      setError("Enter a Unix socket path");
      return;
    }
    try {
      const result = await devtools.connect(socket().trim());
      setSocket(result.path);
      setConnectedSocket(result.path);
      setSelected(undefined);
      setLayoutOverlay(false);
      setScreenshot(undefined);
      await refreshAll();
      setError(undefined);
    } catch (cause) {
      setError(String(cause));
    }
  }

  async function capture(): Promise<void> {
    try {
      setBusy(true);
      const value = await devtools.captureScreenshot();
      setScreenshot(value.path);
      setError(undefined);
    } catch (cause) {
      setError(String(cause));
    } finally {
      setBusy(false);
    }
  }

  async function toggleLayoutOverlay(): Promise<void> {
    const enabled = !layoutOverlay();
    try {
      await devtools.setOverlay(enabled, enabled, enabled, selected()?.id);
      setLayoutOverlay(enabled);
      setError(undefined);
    } catch (cause) {
      setError(String(cause));
    }
  }

  let queryTimer: ReturnType<typeof setTimeout> | undefined;
  let polling = false;
  createEffect(() => {
    query();
    clearTimeout(queryTimer);
    queryTimer = setTimeout(() => void refreshNodes(), 180);
  });

  onMount(() => {
    void refreshAll();
    const timer = setInterval(() => {
      if (polling) return;
      polling = true;
      void Promise.all([refreshStatus(), refreshFrames()]).finally(() => {
        polling = false;
      });
    }, 2000);
    onCleanup(() => clearInterval(timer));
  });
  onCleanup(() => clearTimeout(queryTimer));

  return (
    <div class="w-full h-full flex flex-col overflow-hidden bg-slate-950 text-slate-200 font-sans">
      <header class="flex-none h-14 px-3 flex items-center gap-3 border-b border-slate-700 bg-slate-900">
        <strong class="text-base text-white whitespace-nowrap">
          Wabou DevTools
        </strong>
        <input
          class="flex-1 min-w-0 h-8 px-2 rounded border border-slate-600 bg-slate-950 text-sm text-slate-200"
          value={socket()}
          placeholder="Auto-discover, or enter /run/user/.../wabou-123.sock"
          onInput={(event) => setSocket(event.currentTarget.value)}
        />
        <Button
          unstyled
          class="h-8 px-3 rounded text-white"
          style={(state) => ({
            "background-color": layoutOverlay()
              ? state.hovered
                ? "#7e22ce"
                : "#9333ea"
              : state.hovered
                ? "#475569"
                : "#334155",
          })}
          onClick={() => void toggleLayoutOverlay()}
        >
          Layout
        </Button>
        <Button
          unstyled
          variant="ghost"
          class="h-8 px-3 rounded bg-slate-700 text-white"
          style={(state) => ({
            "background-color": state.hovered ? "#475569" : "#334155",
          })}
          onClick={() => void connect()}
        >
          Connect
        </Button>
        <Button
          unstyled
          tone="sky"
          class="h-8 px-3 rounded bg-blue-600 text-white"
          style={(state) => ({
            "background-color": state.hovered ? "#2563eb" : "#1d4ed8",
          })}
          onClick={() => void refreshAll()}
        >
          Refresh
        </Button>
        <Button
          unstyled
          tone="neutral"
          class="h-8 px-3 rounded bg-emerald-700 text-white"
          style={(state) => ({
            "background-color": state.hovered ? "#047857" : "#065f46",
          })}
          onClick={() => void capture()}
        >
          Capture
        </Button>
        <Popover
          placement="bottom-end"
          trigger={(triggerProps) => (
            <Button
              {...triggerProps}
              unstyled
              variant="ghost"
              class="h-8 px-3 rounded text-white"
              style={(state) => ({
                "background-color": state.hovered ? "#475569" : "#334155",
              })}
            >
              Help
            </Button>
          )}
          contentClass="w-72 p-3 flex flex-col gap-2 rounded border text-sm"
          contentStyle={{
            "background-color": "#0f172a",
            "border-color": "#475569",
            color: "#e2e8f0",
          }}
        >
          <Text class="font-semibold">Native Popover</Text>
          <Text class="text-xs text-slate-400">
            Positioned from Wabou layout snapshots with Floating UI core.
          </Text>
        </Popover>
        <span class="text-xs text-slate-400 whitespace-nowrap">
          <Show when={status()} fallback="disconnected">
            {(current) => `pid ${current().pid} · r${current().revision}`}
          </Show>
        </span>
      </header>

      <Show when={error()}>
        <div class="flex-none px-3 py-2 bg-red-950 text-red-300 text-xs border-b border-red-800">
          {error()} · last snapshot retained
        </div>
      </Show>

      <Show when={connectedSocket()}>
        {(path) => (
          <div class="flex-none px-3 py-1 bg-emerald-950 text-emerald-300 text-xs border-b border-emerald-800">
            Connected to {path()}
          </div>
        )}
      </Show>

      <main class="flex-1 min-h-0 flex overflow-hidden">
        <section class="w-80 flex-none min-h-0 flex flex-col border-r border-slate-700 bg-slate-900">
          <div class="flex-none p-2 border-b border-slate-700">
            <input
              class="w-full h-8 px-2 rounded border border-slate-600 bg-slate-950 text-sm text-slate-200"
              value={query()}
              placeholder="Search tag, text or class"
              onInput={(event) => setQuery(event.currentTarget.value)}
            />
            <div class="mt-1 text-xs text-slate-500">
              {nodes().length} nodes
            </div>
          </div>
          <div class="flex-1 min-h-0 overflow-y-scroll">
            <For each={nodes()}>
              {(node) => (
                <Button
                  unstyled
                  variant="ghost"
                  class="w-full px-2 py-2 flex text-left border-b border-slate-800 bg-slate-900"
                  style={(state) => ({
                    "background-color":
                      selected()?.id === node.id
                        ? "#1e3a5f"
                        : state.hovered
                          ? "#172033"
                          : "#0f172a",
                  })}
                  onClick={() => void inspect(node.id)}
                >
                  <span class="w-12 flex-none text-xs text-slate-500">
                    #{node.id}
                  </span>
                  <span class="w-20 flex-none text-sm text-cyan-400">
                    {node.tag}
                  </span>
                  <span class="flex-1 min-w-0 text-xs text-slate-300">
                    {shortText(node)}
                  </span>
                </Button>
              )}
            </For>
          </div>
        </section>

        <section class="flex-1 min-w-0 min-h-0 flex flex-col bg-slate-950">
          <div class="flex-1 min-h-0 relative flex items-center justify-center overflow-hidden">
            <Show
              when={screenshot()}
              fallback={
                <div class="text-sm text-slate-600">
                  Capture a screenshot to inspect pixels
                </div>
              }
            >
              <div class="relative w-full h-full">
                <img
                  class="w-full h-full"
                  src={screenshot()}
                  alt="Captured application frame"
                />
                <Show when={selectedRect()}>
                  {(rect) => (
                    <div
                      class="absolute border-2 border-red-500 pointer-events-none"
                      style={rect()}
                    />
                  )}
                </Show>
              </div>
            </Show>
          </div>

          <div class="h-52 flex-none border-t border-slate-700 bg-slate-900 flex flex-col">
            <div class="flex-none px-3 py-2 text-xs font-semibold text-slate-400 border-b border-slate-700">
              Protocol frames
            </div>
            <div class="flex-1 min-h-0 overflow-y-scroll px-2">
              <Index each={frames()}>
                {(frame) => (
                  <div class="h-8 flex items-center gap-3 border-b border-slate-800 text-xs font-mono">
                    <span
                      class="w-20"
                      style={{
                        color:
                          frame().direction === "hostToJs"
                            ? "#a78bfa"
                            : "#22d3ee",
                      }}
                    >
                      {frame().direction === "hostToJs"
                        ? "Host → JS"
                        : "JS → Host"}
                    </span>
                    <span class="w-20 text-slate-500">
                      seq {frame().sequence}
                    </span>
                    <span class="w-20">{frame().recordCount} records</span>
                    <span class="text-slate-500">{frame().byteLen} bytes</span>
                  </div>
                )}
              </Index>
            </div>
          </div>
        </section>

        <section class="w-96 flex-none min-h-0 overflow-y-scroll border-l border-slate-700 bg-slate-900 p-3">
          <Show
            when={selected()}
            fallback={<div class="text-sm text-slate-500">Select a node</div>}
          >
            {(node) => (
              <>
                <div class="flex items-center gap-2 mb-3">
                  <strong class="text-lg text-cyan-400">{node().tag}</strong>
                  <span class="text-sm text-slate-500">#{node().id}</span>
                  <Show when={node().widget}>
                    <span class="px-2 py-1 rounded bg-purple-900 text-purple-300 text-xs">
                      {node().widget}
                    </span>
                  </Show>
                </div>
                <Panel title="Layout">
                  <Row
                    label="border box"
                    value={`${node().rect.x}, ${node().rect.y} · ${node().rect.width}×${node().rect.height}`}
                  />
                  <Row
                    label="content box"
                    value={`${node().contentRect.x}, ${node().contentRect.y} · ${node().contentRect.width}×${node().contentRect.height}`}
                  />
                  <Row
                    label="parent"
                    value={node().parentId ? `#${node().parentId}` : "—"}
                  />
                </Panel>
                <Panel title="Classes">
                  <div class="flex flex-wrap gap-1">
                    <For each={node().classes}>
                      {(value) => (
                        <span class="px-2 py-1 rounded bg-slate-800 text-xs text-amber-300">
                          .{value}
                        </span>
                      )}
                    </For>
                  </div>
                </Panel>
                <Panel title="Clip Coordinates">
                  <Row
                    label="widget local"
                    value={
                      node().clip?.widgetLocal
                        ? clipLabel(node().clip!.widgetLocal!)
                        : "—"
                    }
                  />
                  <For each={node().clip?.chain ?? []}>
                    {(clip) => (
                      <Row
                        label={`${clip.kind} #${clip.nodeId}`}
                        value={clipLabel(clip)}
                      />
                    )}
                  </For>
                  <Row
                    label="effective"
                    value={
                      node().clip?.effective
                        ? clipLabel(node().clip!.effective!)
                        : "none"
                    }
                  />
                  <Row
                    label="static transform"
                    value={node().clip?.staticTransform.join(", ") ?? "—"}
                  />
                  <Row
                    label="runtime transform"
                    value={node().clip?.runtimeTransform?.join(", ") ?? "none"}
                  />
                  <Row
                    label="border → window"
                    value={node().clip?.borderTransform.join(", ") ?? "—"}
                  />
                  <Row
                    label="content → window"
                    value={node().clip?.sceneTransform.join(", ") ?? "—"}
                  />
                  <Row
                    label="device scale"
                    value={String(node().clip?.deviceScale ?? 1)}
                  />
                </Panel>
                <Panel title="Attributes">
                  <For each={node().attrs}>
                    {([name, value]) => <Row label={name} value={value} />}
                  </For>
                </Panel>
                <Show when={(node().styleDiagnostics?.length ?? 0) > 0}>
                  <Panel title="Style diagnostics">
                    <For each={node().styleDiagnostics ?? []}>
                      {(diagnostic) => (
                        <Row label="rejected" value={diagnostic} />
                      )}
                    </For>
                  </Panel>
                </Show>
                <Panel title="Computed">
                  <For each={Object.entries(node().computed)}>
                    {([name, value]) => (
                      <Row label={name} value={String(value ?? "—")} />
                    )}
                  </For>
                </Panel>
                <Panel title="Events">
                  <Row
                    label="listeners"
                    value={node().listeners.join(", ") || "none"}
                  />
                </Panel>
              </>
            )}
          </Show>
        </section>
      </main>
    </div>
  );
}

function Panel(props: { title: string; children?: JSX.Element }) {
  return (
    <section class="mb-4 border border-slate-700 rounded overflow-hidden">
      <div class="px-2 py-2 bg-slate-800 text-xs font-semibold text-slate-300">
        {props.title}
      </div>
      <div class="p-2">{props.children}</div>
    </section>
  );
}

function Row(props: { label: string; value: string }) {
  return (
    <div class="flex gap-2 py-1 border-b border-slate-800 text-xs">
      <span class="w-24 flex-none text-slate-500">{props.label}</span>
      <span class="flex-1 min-w-0 text-slate-200">{props.value}</span>
    </div>
  );
}

mount(() => <App />);
