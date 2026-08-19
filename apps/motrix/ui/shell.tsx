import {
  application,
  Button,
  ColorThemeProvider,
  ComponentsProvider,
  createShortcuts,
  dialog,
  Icon,
  Input,
  Modal,
  ScrollArea,
  Text,
  TextArea,
  useFileDrop,
  useLocation,
  useNavigate,
  View,
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
import { useAria2 } from "./aria2";

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
  const initialConfig = untrack(aria2.config);
  const [adding, setAdding] = createSignal(false);
  const [sidebarOpen, setSidebarOpen] = createSignal(true);
  const [source, setSource] = createSignal<"links" | "torrent">("links");
  const [url, setUrl] = createSignal("");
  const [torrentPath, setTorrentPath] = createSignal("");
  const [addError, setAddError] = createSignal("");
  const [draggingFile, setDraggingFile] = createSignal(false);
  const [confirmingQuit, setConfirmingQuit] = createSignal(false);
  const [directory, setDirectory] = createSignal(initialConfig.downloadDir);
  const [filename, setFilename] = createSignal("");
  const [split, setSplit] = createSignal(String(initialConfig.split));
  const setAddTaskOpen = (open: boolean) => {
    setAdding(open);
    if (open) return;
    const config = aria2.config();
    setSource("links");
    setUrl("");
    setTorrentPath("");
    setFilename("");
    setAddError("");
    setDirectory(config.downloadDir);
    setSplit(String(config.split));
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
    setTorrentPath(torrent);
    setSource("torrent");
    setAdding(true);
  });
  const navButton = (path: string, label: string, icon: string) => (
    <Button
      variant="ghost"
      selected={location().pathname === path}
      class={`w-full h-11 justify-start text-base font-medium text-primary ${location().pathname === path ? "bg-selected" : ""}`}
      onClick={() => navigate({ to: path })}
    >
      <Icon source={icon} size={17} />
      {label}
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
    "Primary+B": () => setSidebarOpen((open) => !open),
    "Primary+Q": requestQuit,
  });
  return (
    <ColorThemeProvider
      theme={aria2.config().theme}
      transition={{ duration: 0.18, easing: "ease-out" }}
    >
      <ComponentsProvider theme={aria2.config().theme}>
        <View
          {...shortcuts.bindings}
          class="w-full h-full p-3 flex gap-4 bg-canvas text-primary"
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
          <Show
            when={sidebarOpen()}
            fallback={
              <Button
                aria-label="Show sidebar"
                size="icon"
                variant="ghost"
                class="flex-none mt-5"
                onClick={() => setSidebarOpen(true)}
              >
                <Icon source={panelLeftOpen} size={18} />
              </Button>
            }
          >
            <View class="w-60 flex-none px-3 pt-5 pb-2 flex flex-col rounded-2xl bg-surface-muted">
              <View class="px-1 mb-7 flex items-center gap-2">
                <View class="w-9 h-9 rounded-lg bg-accent flex items-center justify-center">
                  <Icon source={download} size={19} class="text-on-accent" />
                </View>
                <View class="min-w-0 flex-1 flex flex-col">
                  <Text class="font-semibold text-primary">Motrix</Text>
                  <Text class="text-xs text-muted">Powered by Wabou</Text>
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
          </Show>
          <View class="min-w-0 flex-1 flex flex-col overflow-hidden">
            <Show
              when={
                location().pathname !== "/" &&
                location().pathname !== "/downloads"
              }
              fallback={
                <View class="min-h-0 flex-1 overflow-hidden">
                  <View class="w-full min-w-0 h-full max-w-6xl mx-auto px-7 py-5">
                    {props.children}
                  </View>
                </View>
              }
            >
              <ScrollArea class="flex-1">
                <View class="w-full min-w-0 max-w-6xl mx-auto px-7 py-5">
                  {props.children}
                </View>
              </ScrollArea>
            </Show>
          </View>
          <Modal
            aria-label="Add download task"
            open={adding()}
            onOpenChange={setAddTaskOpen}
            contentClass="w-96 max-w-full p-6 flex flex-col gap-4 rounded-xl border border-subtle bg-surface shadow-xl"
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
                    <View class="h-28 p-4 flex flex-col items-center justify-center gap-3 rounded-lg border border-strong bg-control">
                      <Text class="text-sm text-muted">
                        {torrentPath() || "Choose a .torrent file"}
                      </Text>
                      <Button
                        variant="outline"
                        onClick={async () => {
                          const paths = await dialog.open({
                            title: "Choose torrent",
                            filters: [
                              { name: "BitTorrent", extensions: ["torrent"] },
                            ],
                          });
                          if (paths?.[0]) setTorrentPath(paths[0]);
                        }}
                      >
                        Browse…
                      </Button>
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
                <Input
                  aria-label="Output filename"
                  value={filename()}
                  placeholder="File name (automatic)"
                  onInput={(event) => setFilename(event.currentTarget.value)}
                />
                <Show when={addError()}>
                  <Text class="text-sm text-danger-primary">{addError()}</Text>
                </Show>
                <View class="flex justify-end gap-2">
                  <Button variant="ghost" onClick={close}>
                    Cancel
                  </Button>
                  <Button
                    disabled={
                      source() === "links" ? !url().trim() : !torrentPath()
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
                            ...options,
                          });
                        }
                        await aria2.refresh();
                        setUrl("");
                        setTorrentPath("");
                        setFilename("");
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
        </View>
      </ComponentsProvider>
    </ColorThemeProvider>
  );
}
