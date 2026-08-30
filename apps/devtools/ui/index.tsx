// Wabou DevTools UI entry.
import "@wabou/ui";
import "virtual:wabou-stylesheet";
import {
  Badge,
  Button,
  ComponentsProvider,
  createMeasuredSize,
  createOwnedImageResource,
  Image,
  Input,
  mount,
  PrimitiveButton,
  PrimitivePopover,
  ScrollArea,
  Sidebar,
  SidebarContent,
  SidebarHeader,
  StatusBar,
  StatusBarItem,
  StatusBarSeparator,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
  Text,
  Toolbar,
  ToolbarToggle,
  TreeView,
  View,
  type WabouPointerEvent,
} from "@wabou/ui";
import {
  createEffect,
  createMemo,
  createSignal,
  For as ForValue,
  type JSX,
  onCleanup,
  onSettled,
  Show,
} from "solid-js";
import {
  type DebugCaptureCase,
  type DebugClip,
  type DebugFrame,
  type DebugNode,
  type DebugStatus,
  type DebugValidationReport,
  type NodeKey,
  useDevtoolsClient,
} from "./generated/host-bindings";
import {
  buildRetainedTree,
  containSize,
  createLatestRequestGate,
  EMPTY_OVERLAY_LAYERS,
  isExpectedDisconnectedError,
  type OverlayLayer,
  type OverlayLayers,
  overlayEvidenceLabel,
  overlayStyle,
  screenshotPoint,
  toggleOverlayLayer,
  validationStatusLabel,
} from "./model";

function shortText(node: DebugNode): string {
  const text = node.text?.replaceAll("\n", " ").trim();
  return text
    ? text.slice(0, 52)
    : node.classes
        .slice(0, 2)
        .map((c) => `.${c}`)
        .join("");
}

function sameNodeKey(left: NodeKey, right: NodeKey): boolean {
  return left.lo === right.lo && left.hi === right.hi;
}

function nodeKeyLabel(key: NodeKey): string {
  return `${key.lo}:${key.hi}`;
}

function parentNodeLabel(key: NodeKey | null | undefined): string {
  return key ? `#${nodeKeyLabel(key)}` : "—";
}

function clipLabel(clip: DebugClip): string {
  const rect = clip.rect;
  return `${clip.coordinateSpace} · ${rect.x}, ${rect.y} · ${rect.width}×${rect.height} · r${clip.radius} · [${clip.transform?.join(", ") ?? "1, 0, 0, 1, 0, 0"}]`;
}

function optionalClipLabel(
  clip: DebugClip | null | undefined,
  fallback: string,
): string {
  return clip ? clipLabel(clip) : fallback;
}

function App() {
  const devtools = useDevtoolsClient();
  const [status, setStatus] = createSignal<DebugStatus>();
  const [nodes, setNodes] = createSignal<DebugNode[]>([]);
  const [selected, setSelected] = createSignal<DebugNode>();
  const [frames, setFrames] = createSignal<DebugFrame[]>([]);
  const [validation, setValidation] = createSignal<DebugValidationReport>();
  const [query, setQuery] = createSignal("");
  const [socket, setSocket] = createSignal("");
  const [connectedSocket, setConnectedSocket] = createSignal<string>();
  const [screenshot, setScreenshot] = createSignal<string>();
  const screenshotImage = createOwnedImageResource(() => {
    const path = screenshot();
    return path ? { kind: "file", path } : undefined;
  });
  const [error, setError] = createSignal<string>();
  const [busy, setBusy] = createSignal(false);
  const [workspace, setWorkspace] = createSignal<
    "elements" | "preview" | "protocol"
  >("elements");
  const [overlayLayers, setOverlayLayers] =
    createSignal<OverlayLayers>(EMPTY_OVERLAY_LAYERS);
  const screenshotSize = createMeasuredSize();
  const screenshotStageSize = createMeasuredSize();
  const nodeQueryGate = createLatestRequestGate();

  const retainedTree = createMemo(() =>
    buildRetainedTree(
      nodes().map((node) => ({
        id: nodeKeyLabel(node.id),
        parentId: node.parentId ? nodeKeyLabel(node.parentId) : null,
        label: `${node.tag}${shortText(node) ? ` ${shortText(node)}` : ""}`,
      })),
    ),
  );
  const selectedTreeId = createMemo(() => {
    const node = selected();
    return node ? nodeKeyLabel(node.id) : null;
  });

  const fittedScreenshotSize = createMemo(() => {
    const current = status();
    return current
      ? containSize(
          { width: current.viewportWidth, height: current.viewportHeight },
          {
            width: screenshotStageSize.width(),
            height: screenshotStageSize.height(),
          },
        )
      : undefined;
  });

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

  function reportFailure(cause: unknown): void {
    if (isExpectedDisconnectedError(cause)) {
      setConnectedSocket(undefined);
      setStatus(undefined);
      setError(undefined);
      return;
    }
    setError(String(cause));
  }

  async function refreshStatus(): Promise<void> {
    try {
      const value = await devtools.status();
      setStatus(value);
      const overlay = value.overlay ?? EMPTY_OVERLAY_LAYERS;
      setOverlayLayers({
        layout: overlay.layout,
        clips: overlay.clips,
        hitTarget: overlay.hitTarget,
      });
      setError(undefined);
    } catch (cause) {
      reportFailure(cause);
    }
  }

  async function refreshNodes(): Promise<void> {
    const token = nodeQueryGate.begin();
    const requestedQuery = query();
    try {
      const value = await devtools.queryNodes({
        query: requestedQuery,
        limit: 150,
      });
      if (!nodeQueryGate.isCurrent(token)) return;
      setNodes(value);
      const id = selected()?.id;
      if (id) {
        const next = value.find((node) => sameNodeKey(node.id, id));
        if (next) setSelected(next);
      }
      setError(undefined);
    } catch (cause) {
      if (!nodeQueryGate.isCurrent(token)) return;
      reportFailure(cause);
    }
  }

  async function refreshValidation(): Promise<void> {
    try {
      setValidation(await devtools.validateSnapshot());
      setError(undefined);
    } catch (cause) {
      reportFailure(cause);
    }
  }

  async function refreshFrames(): Promise<void> {
    try {
      const next = await devtools.recentFrames({ limit: 20 });
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
      reportFailure(cause);
    }
  }

  async function refreshAll(): Promise<void> {
    if (busy()) return;
    setBusy(true);
    await Promise.all([
      refreshStatus(),
      refreshNodes(),
      refreshFrames(),
      refreshValidation(),
    ]);
    setBusy(false);
  }

  async function selectNode(node: DebugNode | undefined): Promise<void> {
    await devtools.setOverlay({
      ...overlayLayers(),
      selectedNode: node?.id ?? null,
    });
    setSelected(node);
  }

  async function inspect(id: NodeKey): Promise<void> {
    try {
      const node = await devtools.inspectNode({ id });
      await selectNode(node);
      setError(undefined);
    } catch (cause) {
      reportFailure(cause);
    }
  }

  async function inspectScreenshot(event: WabouPointerEvent): Promise<void> {
    if (busy()) return;
    const current = status();
    const point = current
      ? screenshotPoint(
          { x: event.offsetX, y: event.offsetY },
          { width: screenshotSize.width(), height: screenshotSize.height() },
          {
            width: current.viewportWidth,
            height: current.viewportHeight,
          },
        )
      : undefined;
    if (!point) {
      setError("Screenshot geometry is not ready for point inspection");
      return;
    }
    try {
      setBusy(true);
      const captured = await devtools.captureCase(point);
      applyCapture(captured);
      await selectNode(captured.point?.node ?? undefined);
      setError(undefined);
    } catch (cause) {
      reportFailure(cause);
    } finally {
      setBusy(false);
    }
  }

  async function connect(): Promise<void> {
    if (!socket().trim()) {
      setError("Enter a Unix socket path");
      return;
    }
    try {
      const result = await devtools.connect({ path: socket().trim() });
      setSocket(result.path);
      setConnectedSocket(result.path);
      setSelected(undefined);
      setValidation(undefined);
      setOverlayLayers(EMPTY_OVERLAY_LAYERS);
      setScreenshot(undefined);
      await refreshAll();
      setError(undefined);
    } catch (cause) {
      reportFailure(cause);
    }
  }

  function applyCapture(value: DebugCaptureCase): void {
    setScreenshot(value.screenshotPath);
    setStatus(value.snapshot.status);
    setFrames(value.frames);
    const overlay = value.snapshot.status.overlay ?? EMPTY_OVERLAY_LAYERS;
    setOverlayLayers({
      layout: overlay.layout,
      clips: overlay.clips,
      hitTarget: overlay.hitTarget,
    });
    const selectedId = selected()?.id;
    if (selectedId) {
      setSelected(
        value.snapshot.nodes.find((node) => sameNodeKey(node.id, selectedId)),
      );
    }
  }

  async function capture(): Promise<void> {
    if (busy()) return;
    try {
      setBusy(true);
      const value = await devtools.captureCase({ x: null, y: null });
      applyCapture(value);
      await refreshValidation();
      setError(undefined);
    } catch (cause) {
      reportFailure(cause);
    } finally {
      setBusy(false);
    }
  }

  async function toggleOverlay(layer: OverlayLayer): Promise<void> {
    const next = toggleOverlayLayer(overlayLayers(), layer);
    try {
      await devtools.setOverlay({
        ...next,
        selectedNode: selected()?.id ?? null,
      });
      setOverlayLayers(next);
      setError(undefined);
    } catch (cause) {
      reportFailure(cause);
    }
  }

  let queryTimer: ReturnType<typeof setTimeout> | undefined;
  let polling = false;
  createEffect(query, () => {
    clearTimeout(queryTimer);
    queryTimer = setTimeout(() => void refreshNodes(), 180);
  });

  onSettled(() => {
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
    <View class="w-full h-full flex flex-col overflow-hidden bg-canvas text-primary font-sans">
      <View class="flex-none h-14 px-3 flex items-center gap-3 border-b border-subtle bg-surface shadow-sm">
        <View class="flex-none flex flex-col">
          <Text class="text-sm font-semibold whitespace-nowrap">
            Wabou DevTools
          </Text>
          <Text class="text-xs text-muted whitespace-nowrap">
            Native runtime inspector
          </Text>
        </View>
        <Input
          aria-label="Runtime socket"
          class="flex-1 min-w-0"
          value={socket()}
          placeholder="Auto-discover, or enter /run/user/.../wabou-123.sock"
          onInput={(event) => setSocket(event.currentTarget.value)}
        />
        <Toolbar aria-label="Debug overlay layers" class="flex-none">
          <ForValue
            each={
              [
                ["layout", "Bounds"],
                ["clips", "Clips"],
                ["hitTarget", "Hit"],
              ] as const
            }
          >
            {([layer, label]) => (
              <ToolbarToggle
                pressed={overlayLayers()[layer]}
                onPressedChange={() => void toggleOverlay(layer)}
              >
                {label}
              </ToolbarToggle>
            )}
          </ForValue>
        </Toolbar>
        <Button
          variant="secondary"
          disabled={busy()}
          onClick={() => void connect()}
        >
          Connect
        </Button>
        <Button
          variant="outline"
          disabled={busy()}
          onClick={() => void refreshAll()}
        >
          Refresh
        </Button>
        <Button disabled={busy()} onClick={() => void capture()}>
          {busy() ? "Working…" : "Capture"}
        </Button>
        <PrimitivePopover
          aria-label="Help"
          placement="bottom-end"
          trigger={(triggerProps) => (
            <Button {...triggerProps} variant="ghost">
              Help
            </Button>
          )}
          contentClass="w-72 p-3 flex flex-col gap-2 rounded-lg border border-subtle bg-surface shadow-lg text-sm"
        >
          <Text class="font-semibold">Inspect a native runtime</Text>
          <Text class="whitespace-normal text-xs text-muted">
            Connect to a Wabou DevTools socket, validate the retained tree, and
            capture pixels plus geometry from one frame.
          </Text>
        </PrimitivePopover>
        <Badge variant={status() ? "success" : "secondary"}>
          <Show when={status()} fallback="disconnected">
            {(current) => `pid ${current().pid} · r${current().revision}`}
          </Show>
        </Badge>
      </View>

      <Show when={error()}>
        <View
          role="alert"
          class="flex-none px-3 py-2 bg-danger-surface text-danger-primary text-xs border-b border-danger"
        >
          {error()} · the last successful snapshot remains visible
        </View>
      </Show>

      <Tabs
        value={workspace()}
        onValueChange={(value) =>
          setWorkspace(value as "elements" | "preview" | "protocol")
        }
        class="flex-none gap-0 border-b border-subtle bg-surface"
      >
        <TabsList
          variant="line"
          aria-label="DevTools workspaces"
          class="h-9 px-3 gap-1"
        >
          <TabsTrigger value="elements">Elements</TabsTrigger>
          <TabsTrigger value="preview">Preview</TabsTrigger>
          <TabsTrigger value="protocol">Protocol</TabsTrigger>
        </TabsList>
      </Tabs>

      <View role="tabpanel" class="flex-1 min-h-0 flex overflow-hidden">
        <Show when={workspace() === "elements"}>
          <Sidebar
            aria-label="Retained nodes"
            class="flex-1 min-w-0 border-r border-subtle"
          >
            <SidebarHeader class="p-3 flex flex-col gap-2">
              <View class="flex items-center justify-between">
                <Text class="text-sm font-semibold">Retained nodes</Text>
                <Badge variant="secondary">{nodes().length}</Badge>
              </View>
              <Input
                aria-label="Search retained nodes"
                class="w-full"
                value={query()}
                placeholder="Search tag, text or class"
                onInput={(event) => setQuery(event.currentTarget.value)}
              />
            </SidebarHeader>
            <SidebarContent contentClass="p-0">
              <TreeView
                aria-label="Retained node hierarchy"
                class="p-1"
                items={retainedTree()}
                selectedId={selectedTreeId()}
                onSelectedChange={(id) => {
                  const node = nodes().find(
                    (candidate) => nodeKeyLabel(candidate.id) === id,
                  );
                  if (node) void inspect(node.id);
                }}
                renderItem={(item) => {
                  const node = nodes().find(
                    (candidate) => nodeKeyLabel(candidate.id) === item.id,
                  );
                  return (
                    <View class="min-w-0 flex-1 flex items-center gap-2">
                      <Text class="flex-none text-sm text-accent">
                        {node?.tag ?? item.label}
                      </Text>
                      <Text class="min-w-0 flex-1 truncate text-xs text-muted">
                        {node ? shortText(node) : ""}
                      </Text>
                      <Text class="flex-none font-mono text-xs text-muted">
                        #{item.id}
                      </Text>
                    </View>
                  );
                }}
              />
            </SidebarContent>
          </Sidebar>
        </Show>

        <Show when={workspace() !== "elements"}>
          <View class="flex-1 min-w-0 min-h-0 flex flex-col bg-canvas">
            <Show when={workspace() === "preview"}>
              <View
                ref={screenshotStageSize.ref}
                class="flex-1 min-h-0 p-4 relative flex items-center justify-center overflow-hidden"
              >
                <Show
                  when={screenshot()}
                  fallback={
                    <View class="items-center gap-2 text-sm text-muted">
                      <Text class="font-medium">No captured frame</Text>
                      <Text class="text-xs text-muted">
                        Capture one frame to inspect pixels and hit targets.
                      </Text>
                    </View>
                  }
                >
                  <Show when={fittedScreenshotSize()}>
                    {(size) => (
                      <View
                        class="relative overflow-hidden rounded-lg border border-subtle bg-surface shadow-lg"
                        style={{
                          width: `${size().width}px`,
                          height: `${size().height}px`,
                        }}
                      >
                        <Image
                          ref={screenshotSize.ref}
                          class="w-full h-full"
                          resource={screenshotImage.resource()?.handle}
                          aria-label="Captured application frame; click to inspect"
                          onClick={(event) => void inspectScreenshot(event)}
                        />
                        <Show when={selectedRect()}>
                          {(rect) => (
                            <View
                              class="absolute border-2 border-danger pointer-events-none"
                              style={rect()}
                            />
                          )}
                        </Show>
                      </View>
                    )}
                  </Show>
                </Show>
              </View>
            </Show>

            <Show when={workspace() === "protocol"}>
              <View class="flex-1 min-h-0 bg-surface flex flex-col">
                <View class="h-9 flex-none px-3 flex items-center justify-between border-b border-subtle">
                  <Text class="text-xs font-semibold text-secondary">
                    Protocol frames
                  </Text>
                  <Badge variant="secondary">{frames().length}</Badge>
                </View>
                <ScrollArea class="flex-1 min-h-0" contentClass="px-2">
                  <ForValue each={frames()} keyed={false}>
                    {(frame) => (
                      <View class="h-8 flex-none flex items-center gap-3 border-b border-subtle text-xs font-mono">
                        <Text
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
                        </Text>
                        <Text class="w-20 text-muted">
                          seq {frame().sequence}
                        </Text>
                        <Text class="w-20">{frame().recordCount} records</Text>
                        <Text class="text-muted">{frame().byteLen} bytes</Text>
                      </View>
                    )}
                  </ForValue>
                </ScrollArea>
              </View>
            </Show>
          </View>
        </Show>

        <Show when={workspace() !== "protocol"}>
          <ScrollArea
            role="region"
            aria-label="Node inspector"
            class="w-[420px] flex-none min-h-0 border-l border-subtle bg-surface"
            contentClass="p-3"
          >
            <ValidationPanel
              report={validation()}
              currentRevision={status()?.revision}
              busy={busy()}
              onValidate={() => void refreshValidation()}
              onInspect={(id) => void inspect(id)}
            />
            <Show
              when={selected()}
              fallback={
                <View class="p-4 items-center rounded-lg border border-subtle bg-surface-muted">
                  <Text class="text-sm text-muted">Select a retained node</Text>
                </View>
              }
            >
              {(node) => (
                <>
                  <View class="flex items-center gap-2 mb-3">
                    <Text class="text-lg font-semibold text-accent">
                      {node().tag}
                    </Text>
                    <Text class="font-mono text-sm text-muted">
                      #{nodeKeyLabel(node().id)}
                    </Text>
                    <Show when={node().widget}>
                      <Badge variant="secondary">{node().widget}</Badge>
                    </Show>
                  </View>
                  <Tabs defaultValue="layout" class="min-h-0 flex-col gap-3">
                    <TabsList
                      variant="line"
                      aria-label="Node inspector sections"
                    >
                      <TabsTrigger value="layout">Layout</TabsTrigger>
                      <TabsTrigger value="styles">Styles</TabsTrigger>
                      <TabsTrigger value="semantics">Semantics</TabsTrigger>
                    </TabsList>
                    <TabsContent value="layout" class="min-h-0">
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
                          value={parentNodeLabel(node().parentId)}
                        />
                      </Panel>
                      <Panel title="Classes">
                        <View class="flex flex-wrap gap-1">
                          <ForValue each={node().classes}>
                            {(value) => (
                              <Text class="px-2 py-1 rounded bg-control text-xs text-muted">
                                .{value}
                              </Text>
                            )}
                          </ForValue>
                        </View>
                      </Panel>
                      <Panel title="Clip Coordinates">
                        <Row
                          label="widget local"
                          value={optionalClipLabel(
                            node().clip?.widgetLocal,
                            "—",
                          )}
                        />
                        <ForValue each={node().clip?.chain ?? []}>
                          {(clip) => (
                            <Row
                              label={`${clip.kind} #${clip.nodeId}`}
                              value={clipLabel(clip)}
                            />
                          )}
                        </ForValue>
                        <Row
                          label="effective"
                          value={optionalClipLabel(
                            node().clip?.effective,
                            "none",
                          )}
                        />
                        <Row
                          label="static transform"
                          value={
                            node().clip?.staticTransform?.join(", ") ?? "—"
                          }
                        />
                        <Row
                          label="runtime transform"
                          value={
                            node().clip?.runtimeTransform?.join(", ") ?? "none"
                          }
                        />
                        <Row
                          label="border → window"
                          value={
                            node().clip?.borderTransform?.join(", ") ?? "—"
                          }
                        />
                        <Row
                          label="content → window"
                          value={node().clip?.sceneTransform?.join(", ") ?? "—"}
                        />
                        <Row
                          label="device scale"
                          value={String(node().clip?.deviceScale ?? 1)}
                        />
                      </Panel>
                    </TabsContent>
                    <TabsContent value="styles" class="min-h-0">
                      <Show when={(node().styleDiagnostics?.length ?? 0) > 0}>
                        <Panel title="Style diagnostics">
                          <ForValue each={node().styleDiagnostics ?? []}>
                            {(diagnostic) => (
                              <Row label="rejected" value={diagnostic} />
                            )}
                          </ForValue>
                        </Panel>
                      </Show>
                      <Show when={(node().styleCascade?.length ?? 0) > 0}>
                        <Panel title="Style cascade">
                          <ForValue each={node().styleCascade ?? []}>
                            {(entry) => (
                              <Row
                                label={entry.property}
                                value={
                                  entry.overriddenSources.length === 0
                                    ? entry.source
                                    : `${entry.source} ← ${entry.overriddenSources
                                        .slice()
                                        .reverse()
                                        .join(" ← ")}`
                                }
                              />
                            )}
                          </ForValue>
                        </Panel>
                      </Show>
                      <Panel title="Computed">
                        <ForValue each={Object.entries(node().computed)}>
                          {([name, value]) => (
                            <Row label={name} value={String(value ?? "—")} />
                          )}
                        </ForValue>
                      </Panel>
                    </TabsContent>
                    <TabsContent value="semantics" class="min-h-0">
                      <Panel title="Attributes">
                        <ForValue each={node().attrs}>
                          {([name, value]) => (
                            <Row label={name} value={value} />
                          )}
                        </ForValue>
                      </Panel>
                      <Panel title="Events">
                        <Row
                          label="listeners"
                          value={node().listeners.join(", ") || "none"}
                        />
                        <Row
                          label="focusable"
                          value={String(node().focusable ?? false)}
                        />
                        <Row
                          label="focus order"
                          value={String(node().focusOrder ?? "none")}
                        />
                      </Panel>
                      <Show when={node().semantic}>
                        {(semantic) => (
                          <Panel title="Accessibility projection">
                            <Row label="role" value={semantic().role} />
                            <Row
                              label="label"
                              value={semantic().label ?? "none"}
                            />
                            <Row
                              label="exposed"
                              value={String(semantic().exposed)}
                            />
                            <Row
                              label="disabled"
                              value={String(semantic().disabled)}
                            />
                            <Row
                              label="controls"
                              value={
                                semantic()
                                  .controls.map(nodeKeyLabel)
                                  .join(", ") || "none"
                              }
                            />
                          </Panel>
                        )}
                      </Show>
                    </TabsContent>
                  </Tabs>
                </>
              )}
            </Show>
          </ScrollArea>
        </Show>
      </View>
      <StatusBar>
        <StatusBarItem grow>
          {connectedSocket()
            ? `Connected · ${connectedSocket()}`
            : "No runtime connected"}
        </StatusBarItem>
        <StatusBarSeparator />
        <StatusBarItem>
          {validationStatusLabel(validation(), status()?.revision)}
        </StatusBarItem>
        <StatusBarSeparator />
        <StatusBarItem>
          <Show when={status()} fallback="overlay unavailable">
            {(current) =>
              overlayEvidenceLabel(current().overlay, current().overlayPaint)
            }
          </Show>
        </StatusBarItem>
      </StatusBar>
    </View>
  );
}

function Panel(props: { title: string; children?: JSX.Element }) {
  return (
    <View class="mb-4 border border-subtle rounded-lg overflow-hidden bg-surface-muted">
      <View class="px-2 py-2 bg-control text-xs font-semibold text-secondary border-b border-subtle">
        <Text>{props.title}</Text>
      </View>
      <View class="p-2">{props.children}</View>
    </View>
  );
}

function Row(props: { label: string; value: string }) {
  return (
    <View class="flex gap-2 py-1 border-b border-subtle text-xs">
      <Text class="w-24 flex-none text-muted">{props.label}</Text>
      <Text class="flex-1 min-w-0 whitespace-normal text-secondary">
        {props.value}
      </Text>
    </View>
  );
}

function ValidationPanel(props: {
  report: DebugValidationReport | undefined;
  currentRevision: number | undefined;
  busy: boolean;
  onValidate(): void;
  onInspect(id: NodeKey): void;
}) {
  return (
    <View class="mb-4 rounded-lg border border-subtle bg-surface-muted overflow-hidden">
      <View class="p-2 flex items-center justify-between gap-2 border-b border-subtle bg-control">
        <View class="min-w-0 flex flex-col gap-0.5">
          <Text class="text-xs font-semibold text-secondary">
            Snapshot validation
          </Text>
          <Text class="truncate font-mono text-xs text-muted">
            {validationStatusLabel(props.report, props.currentRevision)}
          </Text>
        </View>
        <Button
          size="sm"
          variant="outline"
          disabled={props.busy}
          onClick={props.onValidate}
        >
          Validate
        </Button>
      </View>
      <Show
        when={props.report}
        fallback={
          <Text class="p-3 whitespace-normal text-xs text-muted">
            Validate the retained tree before trusting pixels or layout.
          </Text>
        }
      >
        {(report) => (
          <Show
            when={report().issues.length > 0}
            fallback={
              <View class="p-3 flex items-center gap-2">
                <Badge variant="success">Valid</Badge>
                <Text class="text-xs text-muted">
                  No structural or geometry findings.
                </Text>
              </View>
            }
          >
            <View class="flex flex-col">
              <ForValue each={report().issues}>
                {(issue) => (
                  <PrimitiveButton
                    unstyled
                    disabled={!issue.nodeId}
                    class="w-full min-w-0 p-2 flex items-start gap-2 text-left border-b border-subtle"
                    style={(state) => ({
                      "background-color": state.hovered
                        ? "#27272a"
                        : "#00000000",
                    })}
                    onClick={() => {
                      if (issue.nodeId) props.onInspect(issue.nodeId);
                    }}
                  >
                    <Badge
                      variant={
                        issue.level === "error" ? "destructive" : "secondary"
                      }
                    >
                      {issue.level}
                    </Badge>
                    <View class="min-w-0 flex-1 flex flex-col gap-0.5">
                      <Text class="font-mono text-xs text-secondary">
                        {issue.code}
                      </Text>
                      <Text class="whitespace-normal text-xs text-muted">
                        {issue.message}
                      </Text>
                    </View>
                  </PrimitiveButton>
                )}
              </ForValue>
            </View>
          </Show>
        )}
      </Show>
    </View>
  );
}

mount(() => (
  <ComponentsProvider theme="dark">
    <App />
  </ComponentsProvider>
));
