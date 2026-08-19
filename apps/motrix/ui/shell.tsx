import {
  application,
  Button,
  Checkbox,
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
  ColorThemeProvider,
  ComponentsProvider,
  createNotifications,
  createShortcuts,
  createTransition,
  dialog,
  Icon,
  Input,
  Modal,
  NotificationRegion,
  px,
  ScrollArea,
  Text,
  TextArea,
  useFileDrop,
  useLocation,
  useNavigate,
  useWindow,
  View,
  VirtualList,
} from "@wabou/ui";
import bell from "lucide-static/icons/bell.svg?raw";
import boxes from "lucide-static/icons/boxes.svg?raw";
import download from "lucide-static/icons/download.svg?raw";
import gauge from "lucide-static/icons/gauge.svg?raw";
import panelLeftClose from "lucide-static/icons/panel-left-close.svg?raw";
import panelLeftOpen from "lucide-static/icons/panel-left-open.svg?raw";
import plus from "lucide-static/icons/plus.svg?raw";
import radio from "lucide-static/icons/radio-tower.svg?raw";
import settings from "lucide-static/icons/settings.svg?raw";
import {
  createEffect,
  createSignal,
  For,
  type JSX,
  Show,
  untrack,
} from "solid-js";
import { match } from "ts-pattern";
import type { TorrentPreview } from "./aria2";
import { useAria2 } from "./aria2";
import { formatBytes } from "./lib/format";

const navigation = [
  ["/", "Dashboard", gauge],
  ["/downloads", "Downloads", download],
  ["/trackers", "Trackers", radio],
  ["/plugins", "Plugins", boxes],
] as const;

export function AppShell(props: { children?: JSX.Element }) {
  const location = useLocation();
  const navigate = useNavigate();
  const aria2 = useAria2();
  const window = useWindow();
  const initialConfig = untrack(aria2.config);
  const [adding, setAdding] = createSignal(false);
  const [sidebarOpen, setSidebarOpen] = createSignal(true);
  const sidebarWidth = createTransition(() => (sidebarOpen() ? 208 : 52), {
    duration: 0.22,
    ease: "easeOut",
  });
  const toasts = createNotifications({ defaultDuration: 6_000, limit: 4 });
  let lastToastEventId = untrack(() => aria2.events()[0]?.id ?? 0);
  createEffect(
    () => aria2.events()[0],
    (event) => {
      if (!event || event.id <= lastToastEventId) return;
      lastToastEventId = event.id;
      const presentation = match(event.status)
        .with("complete", () => ({
          title: "Download complete",
          detail: "The task is ready in its destination folder.",
          priority: "polite" as const,
        }))
        .with("error", () => ({
          title: "Download failed",
          detail: "Open Downloads to inspect the aria2 error.",
          priority: "assertive" as const,
        }))
        .exhaustive();
      toasts.show({
        "aria-label": `${presentation.title}: ${event.name}`,
        priority: presentation.priority,
        content: ({ dismiss }) => (
          <View
            class="w-96 max-w-full p-4 flex items-center gap-3 rounded-xl border bg-surface shadow-xl"
            classList={{
              "border-success-primary": event.status === "complete",
              "border-danger-primary": event.status === "error",
            }}
          >
            <View class="min-w-0 flex-1 flex flex-col gap-1">
              <Text class="text-sm font-semibold">{presentation.title}</Text>
              <Text class="truncate text-sm">{event.name}</Text>
              <Text class="text-xs text-muted">{presentation.detail}</Text>
            </View>
            <Button size="sm" variant="ghost" onClick={dismiss}>
              Dismiss
            </Button>
          </View>
        ),
      });
    },
  );
  const [source, setSource] = createSignal<"links" | "torrent">("links");
  const [url, setUrl] = createSignal("");
  const [torrentPath, setTorrentPath] = createSignal("");
  const [torrentPreview, setTorrentPreview] = createSignal<TorrentPreview>();
  const [selectedTorrentFiles, setSelectedTorrentFiles] = createSignal<
    number[]
  >([]);
  const [inspectingTorrent, setInspectingTorrent] = createSignal(false);
  const [addError, setAddError] = createSignal("");
  const [draggingFile, setDraggingFile] = createSignal(false);
  const [confirmingQuit, setConfirmingQuit] = createSignal(false);
  const [directory, setDirectory] = createSignal(initialConfig.downloadDir);
  const [filename, setFilename] = createSignal("");
  const [split, setSplit] = createSignal(String(initialConfig.split));
  const [headers, setHeaders] = createSignal("");
  const [checksum, setChecksum] = createSignal("");
  const [taskProxy, setTaskProxy] = createSignal("");
  let torrentInspection = 0;
  const setAddTaskOpen = (open: boolean) => {
    setAdding(open);
    if (open) return;
    torrentInspection += 1;
    const config = aria2.config();
    setSource("links");
    setUrl("");
    setTorrentPath("");
    setTorrentPreview(undefined);
    setSelectedTorrentFiles([]);
    setInspectingTorrent(false);
    setFilename("");
    setHeaders("");
    setChecksum("");
    setTaskProxy("");
    setAddError("");
    setDirectory(config.downloadDir);
    setSplit(String(config.split));
  };
  const chooseTorrent = async (path: string) => {
    const inspection = ++torrentInspection;
    setTorrentPath(path);
    setTorrentPreview(undefined);
    setSelectedTorrentFiles([]);
    setAddError("");
    setInspectingTorrent(true);
    try {
      const preview = await aria2.inspectTorrent(path);
      if (inspection !== torrentInspection) return;
      setTorrentPreview(preview);
      setSelectedTorrentFiles(preview.files.map((file) => file.index));
    } catch (error) {
      if (inspection === torrentInspection) setAddError(String(error));
    } finally {
      if (inspection === torrentInspection) setInspectingTorrent(false);
    }
  };
  const torrentSummary = () => {
    if (inspectingTorrent()) return "Reading torrent metadata…";
    const preview = torrentPreview();
    if (preview)
      return `${preview.files.length} files · ${formatBytes(preview.totalLength)}`;
    return torrentPath() || "Inspect files before creating the task";
  };
  createEffect(
    () => [aria2.config(), adding()] as const,
    ([config, isAdding]) => {
      if (isAdding) return;
      setDirectory(config.downloadDir);
      setSplit(String(config.split));
    },
  );
  useFileDrop((event) => {
    if (event.phase === "entered" || event.phase === "moved")
      setDraggingFile(true);
    if (event.phase === "left") setDraggingFile(false);
    if (event.phase !== "dropped") return;
    setDraggingFile(false);
    const torrent = event.paths.find((path) =>
      path.toLowerCase().endsWith(".torrent"),
    );
    if (!torrent) return;
    void chooseTorrent(torrent);
    setSource("torrent");
    setAdding(true);
  });
  const navButton = (path: string, label: string, icon: string) => (
    <Button
      variant="ghost"
      selected={location().pathname === path}
      aria-label={label}
      class="w-full h-9 text-sm font-medium text-primary"
      classList={{
        "px-3 justify-start": sidebarOpen(),
        "px-0 justify-center": !sidebarOpen(),
        "bg-selected": location().pathname === path,
      }}
      onClick={() => navigate({ to: path })}
    >
      <Icon source={icon} size={17} />
      <Show when={sidebarOpen()}>
        <Text class="text-sm font-medium text-primary">{label}</Text>
      </Show>
    </Button>
  );
  const requestQuit = () => {
    const running = aria2
      .snapshot()
      .tasks.some((task) =>
        ["active", "waiting", "paused", "seeding"].includes(task.status),
      );
    if (aria2.config().warnBeforeQuit && running) {
      setConfirmingQuit(true);
      return;
    }
    application.exit();
  };
  createEffect(
    () => aria2.quitRequests(),
    (requests) => {
      if (requests > 0) requestQuit();
    },
  );
  const shortcuts = createShortcuts({
    "Primary+N": () => setAdding(true),
    "Primary+Shift+N": () => {
      setSource("torrent");
      setAdding(true);
    },
    "Primary+O": () => {
      setSource("torrent");
      setAdding(true);
    },
    "Primary+,": () => void navigate({ to: "/settings" }),
    "Primary+L": () => void navigate({ to: "/downloads" }),
    "Primary+Shift+P": () => void aria2.globalTaskAction("pauseAll"),
    "Primary+Shift+R": () => void aria2.globalTaskAction("resumeAll"),
    "Primary+B": () => setSidebarOpen((open) => !open),
    "Primary+Q": requestQuit,
  });
  const resolvedTheme = () => {
    const configured = aria2.config().theme;
    return configured === "system" ? window.colorScheme() : configured;
  };
  return (
    <ColorThemeProvider
      theme={resolvedTheme()}
      transition={{ duration: 0.18, easing: "ease-out" }}
    >
      <ComponentsProvider theme={resolvedTheme()}>
        <View
          {...shortcuts.bindings}
          class="w-full h-full p-2 flex gap-3 bg-canvas text-primary"
        >
          <Show when={draggingFile()}>
            <View
              overlayPlane="floating"
              class="absolute inset-4 z-50 flex items-center justify-center rounded-2xl border-2 border-accent bg-surface"
            >
              <View class="flex flex-col items-center gap-3">
                <Icon source={download} size={42} class="text-accent" />
                <Text class="text-xl font-semibold">
                  Drop a torrent file to create a task
                </Text>
              </View>
            </View>
          </Show>
          <View
            class="flex-none overflow-hidden px-2 pt-3 pb-2 flex flex-col rounded-xl bg-surface-muted"
            style={{ width: px(sidebarWidth.value()) }}
          >
            <Show
              when={sidebarOpen()}
              fallback={
                <View class="mb-4 flex flex-col items-center gap-2">
                  <Button
                    aria-label="Show sidebar"
                    size="icon"
                    variant="ghost"
                    onClick={() => setSidebarOpen(true)}
                  >
                    <Icon source={panelLeftOpen} size={18} />
                  </Button>
                  <Button
                    aria-label="New task"
                    size="icon"
                    variant="ghost"
                    onClick={() => setAdding(true)}
                  >
                    <Icon source={plus} size={18} />
                  </Button>
                </View>
              }
            >
              <View class="px-1 mb-4 flex items-center gap-2">
                <View class="w-8 h-8 rounded-lg bg-accent flex items-center justify-center">
                  <Icon source={download} size={17} class="text-on-accent" />
                </View>
                <View class="min-w-0 flex-1 flex flex-col">
                  <Text class="text-sm font-semibold text-primary">Motrix</Text>
                </View>
                <Button
                  aria-label="Hide sidebar"
                  size="icon"
                  variant="ghost"
                  onClick={() => setSidebarOpen(false)}
                >
                  <Icon source={panelLeftClose} size={17} />
                </Button>
                <Button
                  aria-label="New task"
                  size="icon"
                  variant="ghost"
                  onClick={() => setAdding(true)}
                >
                  <Icon source={plus} size={18} />
                </Button>
              </View>
            </Show>
            <View class="flex flex-col gap-1">
              <For each={navigation}>
                {([path, label, icon]) => navButton(path, label, icon)}
              </For>
            </View>
            <View class="flex-1" />
            {navButton("/notifications", "Notifications", bell)}
            <View class="my-2 border-t border-subtle" />
            {navButton("/settings", "Settings", settings)}
          </View>
          <View class="min-w-0 flex-1 flex flex-col overflow-hidden">
            <Show
              when={
                location().pathname !== "/" &&
                location().pathname !== "/downloads"
              }
              fallback={
                <View class="min-h-0 flex-1 overflow-hidden">
                  <View class="w-full min-w-0 h-full max-w-6xl mx-auto px-5 py-4">
                    {props.children}
                  </View>
                </View>
              }
            >
              <ScrollArea class="flex-1">
                <View class="w-full min-w-0 max-w-6xl mx-auto px-5 py-4">
                  {props.children}
                </View>
              </ScrollArea>
            </Show>
          </View>
          <Modal
            aria-label="Add download task"
            open={adding()}
            onOpenChange={setAddTaskOpen}
            contentClass="w-[560px] max-w-full p-6 flex flex-col gap-4 rounded-xl border border-subtle bg-surface shadow-xl"
          >
            {({ close }) => (
              <>
                <Text class="text-xl font-semibold">Add download task</Text>
                <View class="flex gap-2">
                  <Button
                    size="sm"
                    variant={source() === "links" ? "default" : "ghost"}
                    onClick={() => setSource("links")}
                  >
                    Links
                  </Button>
                  <Button
                    size="sm"
                    variant={source() === "torrent" ? "default" : "ghost"}
                    onClick={() => setSource("torrent")}
                  >
                    Torrent file
                  </Button>
                </View>
                <Show
                  when={source() === "links"}
                  fallback={
                    <View class="flex flex-col gap-3">
                      <View class="min-h-24 p-4 flex items-center justify-between gap-4 rounded-lg border border-strong bg-control">
                        <View class="min-w-0 flex-1 flex flex-col gap-1">
                          <Text class="truncate text-sm font-medium">
                            {torrentPreview()?.name ||
                              torrentPath() ||
                              "Choose a .torrent file"}
                          </Text>
                          <Text class="truncate text-xs text-muted">
                            {torrentSummary()}
                          </Text>
                        </View>
                        <Button
                          variant="outline"
                          onClick={async () => {
                            const paths = await dialog.open({
                              title: "Choose torrent",
                              filters: [
                                {
                                  name: "BitTorrent",
                                  extensions: ["torrent"],
                                },
                              ],
                            });
                            if (paths?.[0]) await chooseTorrent(paths[0]);
                          }}
                        >
                          Browse…
                        </Button>
                      </View>
                      <Show when={torrentPreview()} keyed>
                        {(preview) => (
                          <View class="flex flex-col gap-2">
                            <View class="flex items-center justify-between">
                              <Checkbox
                                label="Select all files"
                                checked={
                                  selectedTorrentFiles().length ===
                                  preview.files.length
                                }
                                indeterminate={
                                  selectedTorrentFiles().length > 0 &&
                                  selectedTorrentFiles().length <
                                    preview.files.length
                                }
                                onCheckedChange={(checked) =>
                                  setSelectedTorrentFiles(
                                    checked
                                      ? preview.files.map((file) => file.index)
                                      : [],
                                  )
                                }
                              />
                              <Text class="text-xs text-muted">
                                {selectedTorrentFiles().length} of{" "}
                                {preview.files.length}
                              </Text>
                            </View>
                            <View class="h-44 overflow-hidden rounded-lg border border-subtle bg-surface">
                              <VirtualList
                                items={() => preview.files}
                                itemHeight={32}
                                viewportHeight={176}
                                accessibilityLabel="Torrent files"
                              >
                                {(file) => (
                                  <View class="h-8 px-2 min-w-0 flex items-center gap-2">
                                    <Checkbox
                                      class="min-w-0 flex-1"
                                      label={file.path}
                                      checked={selectedTorrentFiles().includes(
                                        file.index,
                                      )}
                                      onCheckedChange={(checked) =>
                                        setSelectedTorrentFiles((indices) =>
                                          checked
                                            ? [...indices, file.index].sort(
                                                (left, right) => left - right,
                                              )
                                            : indices.filter(
                                                (index) => index !== file.index,
                                              ),
                                        )
                                      }
                                    />
                                    <Text class="flex-none text-xs text-muted">
                                      {formatBytes(file.length)}
                                    </Text>
                                  </View>
                                )}
                              </VirtualList>
                            </View>
                          </View>
                        )}
                      </Show>
                    </View>
                  }
                >
                  <Text class="text-sm text-muted">
                    Enter one HTTP, HTTPS or magnet link per line.
                  </Text>
                  <TextArea
                    class="h-28"
                    aria-label="Download URLs"
                    value={url()}
                    placeholder="https://example.com/file.iso"
                    onInput={(event) => setUrl(event.currentTarget.value)}
                  />
                </Show>
                <View class="flex gap-3">
                  <Input
                    class="flex-1"
                    aria-label="Save directory"
                    value={directory()}
                    placeholder="Default download folder"
                    onInput={(event) => setDirectory(event.currentTarget.value)}
                  />
                  <Input
                    class="w-28"
                    aria-label="Split count"
                    value={split()}
                    placeholder="16"
                    onInput={(event) => setSplit(event.currentTarget.value)}
                  />
                </View>
                <Show when={source() === "links"}>
                  <Input
                    aria-label="Output filename"
                    value={filename()}
                    placeholder="File name (automatic)"
                    onInput={(event) => setFilename(event.currentTarget.value)}
                  />
                  <Collapsible class="rounded-lg border border-subtle p-3">
                    <CollapsibleTrigger>
                      <Text class="text-sm font-medium">
                        Advanced HTTP options
                      </Text>
                    </CollapsibleTrigger>
                    <CollapsibleContent class="pt-3">
                      <View class="flex flex-col gap-3">
                        <TextArea
                          class="h-20"
                          aria-label="HTTP request headers"
                          value={headers()}
                          placeholder="Referer: https://example.com/"
                          onInput={(event) =>
                            setHeaders(event.currentTarget.value)
                          }
                        />
                        <View class="grid grid-cols-2 gap-3">
                          <Input
                            aria-label="Download checksum"
                            value={checksum()}
                            placeholder="sha-256=…"
                            onInput={(event) =>
                              setChecksum(event.currentTarget.value)
                            }
                          />
                          <Input
                            aria-label="Task HTTP proxy"
                            value={taskProxy()}
                            placeholder="http://127.0.0.1:8080"
                            onInput={(event) =>
                              setTaskProxy(event.currentTarget.value)
                            }
                          />
                        </View>
                        <Text class="text-xs text-muted">
                          Put one Name: value header on each line. The proxy
                          must be an HTTP forward proxy.
                        </Text>
                      </View>
                    </CollapsibleContent>
                  </Collapsible>
                </Show>
                <Show when={addError()}>
                  <Text class="text-sm text-danger-primary">{addError()}</Text>
                </Show>
                <View class="flex justify-end gap-2">
                  <Button variant="ghost" onClick={close}>
                    Cancel
                  </Button>
                  <Button
                    disabled={
                      source() === "links"
                        ? !url().trim()
                        : inspectingTorrent() ||
                          !torrentPreview() ||
                          selectedTorrentFiles().length === 0
                    }
                    onClick={async () => {
                      setAddError("");
                      try {
                        const options = {
                          dir: directory().trim() || undefined,
                          split: Number.parseInt(split(), 10) || undefined,
                        };
                        if (source() === "torrent")
                          await aria2.addTorrent({
                            path: torrentPath(),
                            selectedFiles: selectedTorrentFiles(),
                            ...options,
                          });
                        else {
                          const uris = url()
                            .split(/\r?\n/)
                            .map((value) => value.trim())
                            .filter((value) => value && !value.startsWith("#"));
                          await aria2.addUris({
                            uris,
                            out: filename().trim() || undefined,
                            headers: headers()
                              .split(/\r?\n/)
                              .map((value) => value.trim())
                              .filter(Boolean),
                            checksum: checksum().trim() || undefined,
                            proxy: taskProxy().trim() || undefined,
                            ...options,
                          });
                        }
                        await aria2.refresh();
                        setUrl("");
                        setTorrentPath("");
                        setTorrentPreview(undefined);
                        setSelectedTorrentFiles([]);
                        setFilename("");
                        setHeaders("");
                        setChecksum("");
                        setTaskProxy("");
                        close();
                        if (aria2.config().newTaskShowDownloading)
                          await navigate({ to: "/downloads" });
                      } catch (error) {
                        setAddError(String(error));
                      }
                    }}
                  >
                    Create task
                  </Button>
                </View>
              </>
            )}
          </Modal>
          <Modal
            aria-label="Confirm quit"
            open={confirmingQuit()}
            onOpenChange={setConfirmingQuit}
            contentClass="w-96 max-w-full p-6 flex flex-col gap-4 rounded-xl border border-subtle bg-surface shadow-xl"
          >
            {({ close }) => (
              <>
                <Text class="text-xl font-semibold">Quit Motrix?</Text>
                <Text class="text-sm text-muted">
                  Downloads are still running. Their session will be saved and
                  restored the next time Motrix starts.
                </Text>
                <View class="flex justify-end gap-2">
                  <Button variant="ghost" onClick={close}>
                    Keep running
                  </Button>
                  <Button
                    variant="destructive"
                    onClick={() => application.exit()}
                  >
                    Quit Motrix
                  </Button>
                </View>
              </>
            )}
          </Modal>
          <NotificationRegion notifications={toasts} placement="bottom-end" />
        </View>
      </ComponentsProvider>
    </ColorThemeProvider>
  );
}
