import {
  application,
  Button,
  Checkbox,
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
  ColorThemeProvider,
  ComponentsProvider,
  clipboard,
  createEventEffect,
  createFormDraft,
  createLatestAsyncResource,
  createNotifications,
  createShortcuts,
  createTransition,
  DialogScrollBody,
  DirectoryPicker,
  dialog,
  Icon,
  Input,
  Modal,
  NotificationRegion,
  PageViewport,
  px,
  Text,
  TextArea,
  TitleBar,
  TitleBarDragRegion,
  ToggleGroup,
  ToggleGroupItem,
  useFileDrop,
  useLocation,
  useNavigate,
  useRouteActive,
  useWindow,
  View,
  VirtualList,
  WindowFrame,
} from "@wabou/ui";
import bell from "lucide-static/icons/bell.svg?raw";
import download from "lucide-static/icons/download.svg?raw";
import gauge from "lucide-static/icons/gauge.svg?raw";
import minus from "lucide-static/icons/minus.svg?raw";
import panelLeftClose from "lucide-static/icons/panel-left-close.svg?raw";
import panelLeftOpen from "lucide-static/icons/panel-left-open.svg?raw";
import plus from "lucide-static/icons/plus.svg?raw";
import settings from "lucide-static/icons/settings.svg?raw";
import square from "lucide-static/icons/square.svg?raw";
import x from "lucide-static/icons/x.svg?raw";
import {
  createEffect,
  createSignal,
  For,
  type JSX,
  Show,
  untrack,
} from "solid-js";
import { match } from "ts-pattern";
import { AppActionsProvider } from "./app-actions";
import type { MotrixConfig, TaskPriority, TorrentPreview } from "./downloads";
import { useDownloads } from "./downloads";
import { parseCurlDownload } from "./lib/curl";
import { downloadUriError, downloadUris } from "./lib/download-uri";
import { formatBytes } from "./lib/format";

const navigation = [
  ["/", "Dashboard", gauge],
  ["/downloads", "Downloads", download],
] as const;

function initialAddTaskDraft(config: MotrixConfig) {
  return {
    source: "links" as "links" | "torrent",
    url: "",
    torrentPath: "",
    selectedTorrentFiles: [] as number[],
    directory: config.downloadDir,
    filename: "",
    split: String(config.split),
    headers: "",
    priority: "normal" as TaskPriority,
  };
}

type AddTaskDraft = ReturnType<typeof initialAddTaskDraft>;

function validateAddTaskDraft(value: Readonly<AddTaskDraft>) {
  const errors: Partial<Record<keyof AddTaskDraft, string>> = {};
  const split = Number(value.split);
  if (!/^\d+$/.test(value.split.trim()) || !Number.isSafeInteger(split))
    errors.split = "Split count must be a whole number.";
  else if (split < 1 || split > 64)
    errors.split = "Split count must be between 1 and 64.";
  if (
    value.source === "links" &&
    value.filename.trim() &&
    downloadUris(value.url).length > 1
  )
    errors.filename = "A custom output filename can only be used with one URL.";
  if (value.source === "links" && value.url.trim()) {
    const error = downloadUriError(value.url);
    if (error) errors.url = error;
  }
  if (
    value.source === "links" &&
    value.headers
      .split(/\r?\n/)
      .map((header) => header.trim())
      .filter(Boolean)
      .some(
        (header) => !header.includes(":") || !header.split(":", 1)[0]?.trim(),
      )
  )
    errors.headers = "Each HTTP header must use the Name: value format.";
  return errors;
}

export function AppShell(props: { children?: JSX.Element }) {
  const navigate = useNavigate();
  const location = useLocation();
  const downloads = useDownloads();
  const window = useWindow();
  const initialConfig = untrack(downloads.config);
  const [adding, setAdding] = createSignal(false);
  const [sidebarOpen, setSidebarOpen] = createSignal(true);
  const sidebarWidth = createTransition(() => (sidebarOpen() ? 232 : 52), {
    duration: 0.22,
    ease: "easeOut",
  });
  const toasts = createNotifications({ defaultDuration: 6_000, limit: 4 });
  createEventEffect({
    source: downloads.events,
    sequence: (event) => event.id,
    onEvent: (event) => {
      const presentation = match(event.status)
        .with("complete", () => ({
          title: "Download complete",
          detail: "The task is ready in its destination folder.",
          priority: "polite" as const,
        }))
        .with("error", () => ({
          title: "Download failed",
          detail: "Open Downloads to inspect the downloads error.",
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
  });
  const addTask = createFormDraft(initialAddTaskDraft(initialConfig), {
    validate: validateAddTaskDraft,
  });
  const [source, setSource] = addTask.control("source");
  const [url, setUrl] = addTask.control("url");
  const [torrentPath, setTorrentPath] = addTask.control("torrentPath");
  const [selectedTorrentFiles, setSelectedTorrentFiles] = addTask.control(
    "selectedTorrentFiles",
  );
  const [addError, setAddError] = createSignal("");
  const [addNotice, setAddNotice] = createSignal("");
  const [draggingFile, setDraggingFile] = createSignal(false);
  const [dropError, setDropError] = createSignal("");
  const [confirmingQuit, setConfirmingQuit] = createSignal(false);
  const [directory, setDirectory] = addTask.control("directory");
  const [filename, setFilename] = addTask.control("filename");
  const [split, setSplit] = addTask.control("split");
  const [headers, setHeaders] = addTask.control("headers");
  const [priority, setPriority] = addTask.control("priority");
  const torrentInspection = createLatestAsyncResource<string, TorrentPreview>({
    source: () => torrentPath() || undefined,
    load: (path) => downloads.inspectTorrent(path),
  });
  const torrentPreview = torrentInspection.value;
  const inspectingTorrent = torrentInspection.loading;
  createEffect(torrentInspection.value, (preview) => {
    if (preview)
      setSelectedTorrentFiles(preview.files.map((file) => file.index));
  });
  createEffect(torrentInspection.error, (error) => {
    if (error) setAddError(String(error));
  });
  const setAddTaskOpen = (open: boolean) => {
    setAdding(open);
    if (open) return;
    const config = downloads.config();
    addTask.resetTo(initialAddTaskDraft(config));
    setAddError("");
    setAddNotice("");
  };
  const chooseTorrent = (path: string) => {
    setSelectedTorrentFiles([]);
    setAddError("");
    if (untrack(torrentPath) === path) void torrentInspection.refresh();
    else setTorrentPath(path);
  };
  const torrentSummary = () => {
    if (inspectingTorrent()) return "Reading torrent metadata…";
    const preview = torrentPreview();
    if (preview)
      return `${preview.files.length} files · ${formatBytes(preview.totalLength)}`;
    return torrentPath() || "Inspect files before creating the task";
  };
  const pasteLinks = async () => {
    setAddError("");
    setAddNotice("");
    try {
      const text = await clipboard.readText();
      if (!text?.trim()) {
        setAddNotice("The clipboard does not contain text.");
        return;
      }
      const curl = parseCurlDownload(text);
      if (!curl) {
        setUrl(text.trim());
        setAddNotice("Pasted clipboard text.");
        return;
      }
      setUrl(curl.urls.join("\n"));
      setHeaders(curl.headers.join("\n"));
      setFilename(curl.output ?? "");
      setAddNotice(
        curl.proxy
          ? "Imported URLs and headers. Per-task proxies are not supported."
          : "Imported URLs and request headers from cURL.",
      );
    } catch (error) {
      setAddError(`Cannot read clipboard: ${String(error)}`);
    }
  };
  createEffect(
    () => [downloads.config(), adding()] as const,
    ([config, isAdding]) => {
      if (isAdding) return;
      addTask.resetTo(initialAddTaskDraft(config));
    },
  );
  useFileDrop((event) => {
    if (event.phase === "entered" || event.phase === "moved") {
      setDraggingFile(true);
      setDropError("");
    }
    if (event.phase === "left") setDraggingFile(false);
    if (event.phase !== "dropped") return;
    setDraggingFile(false);
    const torrent = event.paths.find((path) =>
      path.toLowerCase().endsWith(".torrent"),
    );
    if (!torrent) {
      setDropError("Only .torrent files can be dropped here.");
      return;
    }
    setDropError("");
    chooseTorrent(torrent);
    setSource("torrent");
    setAdding(true);
  });
  const navButton = (path: string, label: string, icon: string) => {
    const active = useRouteActive(path);
    return (
      <Button
        variant="ghost"
        selected={active()}
        aria-current={active() ? "page" : undefined}
        aria-label={label}
        class="w-full h-12 text-base font-medium text-primary"
        classList={{
          "px-3 justify-start": sidebarOpen(),
          "px-0 justify-center": !sidebarOpen(),
          "bg-selected": active(),
        }}
        onClick={() => navigate({ to: path })}
      >
        <Icon source={icon} size={19} />
        <Show when={sidebarOpen()}>
          <Text class="text-base font-medium text-primary">{label}</Text>
        </Show>
      </Button>
    );
  };
  const requestQuit = () => {
    const running = downloads
      .snapshot()
      .tasks.some((task) =>
        ["active", "waiting", "paused", "seeding"].includes(task.status),
      );
    if (downloads.config().warnBeforeQuit && running) {
      window.show();
      setConfirmingQuit(true);
      return;
    }
    application.exit();
  };
  createEffect(
    () => downloads.quitRequests(),
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
    "Primary+,": () => navigate({ to: "/settings" }),
    "Primary+L": () => navigate({ to: "/downloads" }),
    "Primary+Shift+P": () => downloads.globalTaskAction("pauseAll"),
    "Primary+Shift+R": () => downloads.globalTaskAction("resumeAll"),
    "Primary+B": () => setSidebarOpen((open) => !open),
    "Primary+Q": requestQuit,
  });
  const resolvedTheme = () => {
    const configured = downloads.config().theme;
    return configured === "system" ? window.colorScheme() : configured;
  };
  return (
    <ColorThemeProvider
      theme={resolvedTheme()}
      transition={{ duration: 0.18, easing: "ease-out" }}
    >
      <ComponentsProvider theme={resolvedTheme()}>
        <WindowFrame
          {...shortcuts.bindings}
          class="flex flex-col bg-canvas text-primary"
        >
          <TitleBar class="px-2 bg-canvas">
            <View
              class="h-full flex-none px-2 flex items-center gap-2"
              style={{ width: px(sidebarWidth.value()) }}
            >
              <Show
                when={sidebarOpen()}
                fallback={
                  <Button
                    aria-label="Show sidebar"
                    size="icon"
                    variant="ghost"
                    onClick={() => setSidebarOpen(true)}
                  >
                    <Icon source={panelLeftOpen} size={18} />
                  </Button>
                }
              >
                <View class="w-8 h-8 flex-none rounded-lg bg-accent flex items-center justify-center">
                  <Icon source={download} size={17} class="text-on-accent" />
                </View>
                <Text class="min-w-0 flex-1 truncate text-sm font-semibold text-primary">
                  Motrix
                </Text>
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
              </Show>
            </View>
            <TitleBarDragRegion class="min-w-0 justify-center">
              <Text class="text-xs text-muted">Motrix · Wabou</Text>
            </TitleBarDragRegion>
            <Button
              aria-label="Minimize window"
              size="icon"
              variant="ghost"
              onClick={() => window.minimize()}
            >
              <Icon source={minus} size={16} />
            </Button>
            <Button
              aria-label={
                window.maximized() ? "Restore window" : "Maximize window"
              }
              size="icon"
              variant="ghost"
              onClick={() => window.setMaximized(!window.maximized())}
            >
              <Icon source={square} size={14} />
            </Button>
            <Button
              aria-label="Close window"
              size="icon"
              variant="ghost"
              onClick={() => window.close()}
            >
              <Icon source={x} size={17} />
            </Button>
          </TitleBar>
          <View class="relative min-h-0 flex-1 px-2 pb-2 flex gap-3">
            <Show when={draggingFile()}>
              <View
                overlayPlane="floating"
                role="status"
                aria-label="Torrent drop target"
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
            <Show when={dropError()}>
              <View
                overlayPlane="floating"
                role="alert"
                aria-label={dropError()}
                class="absolute top-4 right-4 z-50 max-w-md p-3 flex items-center gap-3 rounded-xl border border-danger-primary bg-surface shadow-lg"
              >
                <Text class="min-w-0 flex-1 text-sm text-danger-primary">
                  {dropError()}
                </Text>
                <Button
                  size="sm"
                  variant="ghost"
                  aria-label="Dismiss file drop error"
                  onClick={() => setDropError("")}
                >
                  Dismiss
                </Button>
              </View>
            </Show>
            <View
              class="flex-none overflow-hidden px-2 py-2 flex flex-col rounded-xl bg-surface-muted"
              style={{ width: px(sidebarWidth.value()) }}
            >
              <Show when={!sidebarOpen()}>
                <View class="mb-4 flex flex-col items-center gap-2">
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
              <PageViewport
                contentClass="max-w-6xl mx-auto px-5 py-4"
                resetKey={location().pathname}
              >
                <AppActionsProvider
                  value={{ openAddTask: () => setAdding(true) }}
                >
                  {props.children}
                </AppActionsProvider>
              </PageViewport>
            </View>
          </View>
          <Modal
            aria-label="Add download task"
            open={adding()}
            onOpenChange={setAddTaskOpen}
            contentClass="w-[560px] max-w-full max-h-11/12 overflow-hidden p-6 flex flex-col gap-4 rounded-xl border border-subtle bg-surface shadow-xl"
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
                <DialogScrollBody contentClass="pr-2 flex flex-col gap-4">
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
                            <Text
                              role="status"
                              aria-label={torrentSummary()}
                              class="truncate text-xs text-muted"
                            >
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
                              if (paths?.[0]) chooseTorrent(paths[0]);
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
                                        ? preview.files.map(
                                            (file) => file.index,
                                          )
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
                                  getItemKey={(file) => file.index}
                                  itemHeight={32}
                                  viewportHeight={176}
                                  accessibilityLabel="Torrent files"
                                >
                                  {(file) => (
                                    <View class="h-8 px-2 min-w-0 flex items-center gap-2">
                                      <Checkbox
                                        class="min-w-0 flex-1"
                                        label={file().path}
                                        checked={selectedTorrentFiles().includes(
                                          file().index,
                                        )}
                                        onCheckedChange={(checked) =>
                                          setSelectedTorrentFiles((indices) =>
                                            checked
                                              ? [...indices, file().index].sort(
                                                  (left, right) => left - right,
                                                )
                                              : indices.filter(
                                                  (index) =>
                                                    index !== file().index,
                                                ),
                                          )
                                        }
                                      />
                                      <Text class="flex-none text-xs text-muted">
                                        {formatBytes(file().length)}
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
                    <View class="flex items-center justify-between gap-3">
                      <Text class="text-sm text-muted">
                        Enter one HTTP, HTTPS or magnet link per line.
                      </Text>
                      <Button size="sm" variant="outline" onClick={pasteLinks}>
                        Paste
                      </Button>
                    </View>
                    <TextArea
                      class="h-28"
                      aria-label="Download URLs"
                      value={url()}
                      placeholder="https://example.com/file.iso"
                      onInput={(event) => setUrl(event.currentTarget.value)}
                    />
                    <Show when={addTask.fieldError("url")}>
                      {(error) => (
                        <Text
                          role="alert"
                          aria-label="Download URI validation error"
                          class="text-sm text-danger-primary"
                        >
                          {error()}
                        </Text>
                      )}
                    </Show>
                  </Show>
                  <View class="flex gap-3">
                    <DirectoryPicker
                      class="min-w-0 flex-1"
                      aria-label="Save directory"
                      value={directory()}
                      placeholder="Default download folder"
                      browseLabel="Browse"
                      browseAriaLabel="Browse save directory"
                      dialogOptions={{ title: "Choose a download folder" }}
                      onValueChange={setDirectory}
                      onBrowseError={(error) => setAddError(String(error))}
                    />
                    <Input
                      class="w-28"
                      aria-label="Split count"
                      value={split()}
                      placeholder="16"
                      onInput={(event) => setSplit(event.currentTarget.value)}
                    />
                  </View>
                  <View class="flex flex-col gap-1">
                    <Text class="text-xs text-muted">Queue priority</Text>
                    <ToggleGroup
                      type="single"
                      aria-label="New task priority"
                      value={priority()}
                      class="w-full gap-1 bg-transparent p-0"
                      onValueChange={(value) => {
                        if (value) setPriority(value as TaskPriority);
                      }}
                    >
                      <ToggleGroupItem value="low" class="min-w-0 flex-1">
                        Low
                      </ToggleGroupItem>
                      <ToggleGroupItem value="normal" class="min-w-0 flex-1">
                        Normal
                      </ToggleGroupItem>
                      <ToggleGroupItem value="high" class="min-w-0 flex-1">
                        High
                      </ToggleGroupItem>
                      <ToggleGroupItem value="critical" class="min-w-0 flex-1">
                        Critical
                      </ToggleGroupItem>
                    </ToggleGroup>
                  </View>
                  <Show when={source() === "links"}>
                    <Input
                      aria-label="Output filename"
                      value={filename()}
                      placeholder="File name (automatic)"
                      onInput={(event) =>
                        setFilename(event.currentTarget.value)
                      }
                    />
                    <Show when={addTask.fieldError("filename")}>
                      <Text role="alert" class="text-sm text-danger-primary">
                        {addTask.fieldError("filename")}
                      </Text>
                    </Show>
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
                          <Text class="text-xs text-muted">
                            Put one Name: value header on each line.
                          </Text>
                        </View>
                      </CollapsibleContent>
                    </Collapsible>
                  </Show>
                  <Show
                    when={
                      addTask.fieldError("split") ??
                      addTask.fieldError("headers")
                    }
                  >
                    {(error) => (
                      <Text
                        role="alert"
                        aria-label="Add task validation error"
                        class="text-sm text-danger-primary"
                      >
                        {error()}
                      </Text>
                    )}
                  </Show>
                  <Show when={addError()}>
                    <Text
                      role="alert"
                      aria-label="Add task error"
                      class="text-sm text-danger-primary"
                    >
                      {addError()}
                    </Text>
                  </Show>
                  <Show when={addNotice()}>
                    <Text role="status" class="text-sm text-muted">
                      {addNotice()}
                    </Text>
                  </Show>
                </DialogScrollBody>
                <View class="flex justify-end gap-2">
                  <Button variant="ghost" onClick={close}>
                    Cancel
                  </Button>
                  <Button
                    disabled={
                      !addTask.valid() ||
                      (source() === "links"
                        ? !url().trim()
                        : inspectingTorrent() ||
                          !torrentPreview() ||
                          selectedTorrentFiles().length === 0)
                    }
                    onClick={async () => {
                      if (!addTask.valid()) return;
                      setAddError("");
                      try {
                        const options = {
                          dir: directory().trim() || undefined,
                          split: Number.parseInt(split(), 10) || undefined,
                          priority: priority(),
                        };
                        if (source() === "torrent")
                          await downloads.addTorrent({
                            path: torrentPath(),
                            selectedFiles: selectedTorrentFiles(),
                            ...options,
                          });
                        else {
                          const uris = downloadUris(url());
                          await downloads.addUris({
                            uris,
                            out: filename().trim() || undefined,
                            headers: headers()
                              .split(/\r?\n/)
                              .map((value) => value.trim())
                              .filter(Boolean),
                            ...options,
                          });
                        }
                        close();
                        if (downloads.config().newTaskShowDownloading)
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
        </WindowFrame>
      </ComponentsProvider>
    </ColorThemeProvider>
  );
}
