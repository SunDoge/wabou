import {
  application,
  Button,
  ColorThemeProvider,
  ComponentsProvider,
  createEventEffect,
  createNotifications,
  createShortcuts,
  createTransition,
  Icon,
  Modal,
  NotificationRegion,
  PageViewport,
  Text,
  useFileDrop,
  useLocation,
  useNavigate,
  useWindow,
  View,
  WindowFrame,
} from "@wabou/ui";
import download from "lucide-static/icons/download.svg?raw";
import { createEffect, createSignal, type JSX, Show } from "solid-js";
import { match } from "ts-pattern";
import { AddTaskDialog } from "./add-task-dialog";
import { AppActionsProvider } from "./app-actions";
import { useDownloads } from "./downloads";
import { AppSidebar, AppTitleBar } from "./shell-chrome";

export function AppShell(props: { children?: JSX.Element }) {
  const navigate = useNavigate();
  const location = useLocation();
  const downloads = useDownloads();
  const window = useWindow();
  const [adding, setAdding] = createSignal(false);
  const [addSource, setAddSource] = createSignal<"links" | "torrent">("links");
  const [droppedTorrent, setDroppedTorrent] = createSignal<string>();
  const [sidebarOpen, setSidebarOpen] = createSignal(true);
  const sidebarWidth = createTransition(() => (sidebarOpen() ? 232 : 52), {
    duration: 0.22,
    ease: "easeOut",
  });
  const [draggingFile, setDraggingFile] = createSignal(false);
  const [dropError, setDropError] = createSignal("");
  const [confirmingQuit, setConfirmingQuit] = createSignal(false);
  const toasts = createNotifications({ defaultDuration: 6_000, limit: 4 });

  const openAddTask = (
    source: "links" | "torrent" = "links",
    torrentPath?: string,
  ) => {
    setAddSource(source);
    setDroppedTorrent(torrentPath);
    setAdding(true);
  };

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
    openAddTask("torrent", torrent);
  });

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
    "Primary+N": () => openAddTask(),
    "Primary+Shift+N": () => openAddTask("torrent"),
    "Primary+O": () => openAddTask("torrent"),
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
          <AppTitleBar
            sidebarOpen={sidebarOpen()}
            sidebarWidth={sidebarWidth.value()}
            onSidebarOpenChange={setSidebarOpen}
            onNewTask={() => openAddTask()}
          />
          <View class="relative min-h-0 flex-1 p-2 flex gap-3">
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
            <AppSidebar
              sidebarOpen={sidebarOpen()}
              sidebarWidth={sidebarWidth.value()}
              onSidebarOpenChange={setSidebarOpen}
              onNewTask={() => openAddTask()}
            />
            <View class="min-w-0 flex-1 flex flex-col overflow-hidden">
              <PageViewport
                contentClass="max-w-6xl mx-auto px-5 py-4"
                resetKey={location().pathname}
              >
                <AppActionsProvider
                  value={{ openAddTask: () => openAddTask() }}
                >
                  {props.children}
                </AppActionsProvider>
              </PageViewport>
            </View>
          </View>
          <AddTaskDialog
            open={adding()}
            initialSource={addSource()}
            initialTorrentPath={droppedTorrent()}
            onOpenChange={setAdding}
            onCreated={() => {
              if (downloads.config().newTaskShowDownloading)
                return navigate({ to: "/downloads" });
            }}
          />
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
