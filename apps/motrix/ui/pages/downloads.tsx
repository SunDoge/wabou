import {
  Badge,
  Button,
  Card,
  CardContent,
  Checkbox,
  clipboard,
  Icon,
  Input,
  Modal,
  Progress,
  ScrollArea,
  Text,
  View,
} from "@wabou/ui";
import inspect from "lucide-static/icons/panel-right-open.svg?raw";
import pause from "lucide-static/icons/pause.svg?raw";
import play from "lucide-static/icons/play.svg?raw";
import retry from "lucide-static/icons/rotate-ccw.svg?raw";
import trash from "lucide-static/icons/trash-2.svg?raw";
import { createEffect, createMemo, createSignal, For, Show } from "solid-js";
import { type Aria2Task, type Aria2TaskDetails, useAria2 } from "../aria2";
import { formatBytes } from "../lib/format";

export function DownloadsPage() {
  const aria2 = useAria2();
  const [query, setQuery] = createSignal("");
  type Filter = "all" | "active" | "waiting" | "complete" | "stopped";
  const filters: readonly [Filter, string][] = [
    ["all", "All"],
    ["active", "Downloading"],
    ["waiting", "Waiting"],
    ["complete", "Completed"],
    ["stopped", "Stopped"],
  ];
  const [filter, setFilter] = createSignal<Filter>("all");
  const [selected, setSelected] = createSignal<Aria2Task>();
  const [selectedGids, setSelectedGids] = createSignal<ReadonlySet<string>>(
    new Set<string>(),
  );
  const [actionError, setActionError] = createSignal("");
  const [pendingRemoval, setPendingRemoval] = createSignal<Aria2Task[]>([]);
  const [removeFiles, setRemoveFiles] = createSignal(false);
  const [details, setDetails] = createSignal<Aria2TaskDetails>();
  const [detailsError, setDetailsError] = createSignal("");
  const [fileSelectionBusy, setFileSelectionBusy] = createSignal(false);
  const [detailTab, setDetailTab] = createSignal<
    "overview" | "files" | "peers"
  >("overview");
  let detailsRequest = 0;
  createEffect(
    () => selected()?.gid,
    (gid) => {
      const request = ++detailsRequest;
      setDetails(undefined);
      setDetailsError("");
      setDetailTab("overview");
      if (!gid) return;
      void aria2
        .taskDetails(gid)
        .then((value) => {
          if (request === detailsRequest) setDetails(value);
        })
        .catch((error) => {
          if (request === detailsRequest) setDetailsError(String(error));
        });
    },
  );
  const executeAction = async (action: () => Promise<void>) => {
    setActionError("");
    try {
      await action();
    } catch (error) {
      setActionError(String(error));
    }
  };
  const toggleTaskFile = async (index: number, checked: boolean) => {
    const task = selected();
    const current = details();
    if (!task || !current || fileSelectionBusy()) return;
    const indices = current.files
      .filter((file) => (file.index === index ? checked : file.selected))
      .map((file) => file.index);
    if (indices.length === 0) {
      setDetailsError("At least one file must remain selected.");
      return;
    }
    setFileSelectionBusy(true);
    setDetailsError("");
    try {
      setDetails(await aria2.setSelectedFiles(task.gid, indices));
      await aria2.refresh();
    } catch (error) {
      setDetailsError(String(error));
    } finally {
      setFileSelectionBusy(false);
    }
  };
  const toggleSelected = (gid: string, checked: boolean) =>
    setSelectedGids((current) => {
      const next = new Set(current);
      if (checked) next.add(gid);
      else next.delete(gid);
      return next;
    });
  const runBatch = async (action: "pause" | "resume") => {
    await aria2.batchTaskAction([...selectedGids()], action);
    setSelectedGids(new Set<string>());
    await aria2.refresh();
  };
  const requestRemoval = (tasks: Aria2Task[]) => {
    setRemoveFiles(false);
    setPendingRemoval(tasks);
  };
  const matchesFilter = (task: Aria2Task) => {
    if (filter() === "all") return true;
    if (filter() === "waiting")
      return task.status === "waiting" || task.status === "paused";
    if (filter() === "stopped")
      return task.status === "error" || task.status === "removed";
    if (filter() === "active")
      return task.status === "active" || task.status === "seeding";
    return task.status === filter();
  };
  const shown = createMemo(() =>
    aria2
      .snapshot()
      .tasks.filter(
        (task) =>
          matchesFilter(task) &&
          task.name.toLowerCase().includes(query().toLowerCase()),
      ),
  );
  return (
    <View class="h-full min-h-0 flex flex-col gap-5">
      <View class="flex-none flex items-center justify-between">
        <View class="flex items-center gap-3">
          <Text role="heading" class="text-3xl font-bold">
            All Downloads
          </Text>
          <Badge variant="outline">
            {shown().length}/{aria2.snapshot().tasks.length}
          </Badge>
        </View>
        <View class="flex items-center gap-2">
          <Button
            size="sm"
            variant="ghost"
            disabled={!aria2.snapshot().connected}
            onClick={() =>
              executeAction(async () => {
                await aria2.globalTaskAction("pauseAll");
                await aria2.refresh();
              })
            }
          >
            <Icon source={pause} size={14} />
            Pause all
          </Button>
          <Button
            size="sm"
            variant="ghost"
            disabled={!aria2.snapshot().connected}
            onClick={() =>
              executeAction(async () => {
                await aria2.globalTaskAction("resumeAll");
                await aria2.refresh();
              })
            }
          >
            <Icon source={play} size={14} />
            Resume all
          </Button>
          <Button
            size="sm"
            variant="ghost"
            disabled={
              !aria2
                .snapshot()
                .tasks.some(
                  (task) =>
                    task.status === "complete" || task.status === "error",
                )
            }
            onClick={() =>
              executeAction(async () => {
                await aria2.globalTaskAction("clearCompleted");
                await aria2.refresh();
              })
            }
          >
            Clear finished
          </Button>
          <View class="w-56">
            <Input
              aria-label="Search downloads"
              value={query()}
              placeholder="Search downloads"
              onInput={(event) => setQuery(event.currentTarget.value)}
            />
          </View>
        </View>
      </View>
      <View class="flex-none flex items-center justify-between">
        <View class="flex gap-2">
          <For each={filters}>
            {([value, label]) => (
              <Button
                size="sm"
                variant={filter() === value ? "default" : "ghost"}
                onClick={() => {
                  setFilter(value);
                  setSelected(undefined);
                  setSelectedGids(new Set<string>());
                }}
              >
                {label}
              </Button>
            )}
          </For>
        </View>
        <Show
          when={selectedGids().size > 0}
          fallback={
            <Show when={selected()}>
              {(task) => (
                <Text class="text-sm text-muted">
                  Inspecting · {task().name}
                </Text>
              )}
            </Show>
          }
        >
          <View class="flex items-center gap-2">
            <Text class="text-sm text-muted">
              {selectedGids().size} selected
            </Text>
            <Button
              size="sm"
              variant="ghost"
              onClick={() => executeAction(() => runBatch("pause"))}
            >
              <Icon source={pause} size={14} />
              Pause
            </Button>
            <Button
              size="sm"
              variant="ghost"
              onClick={() => executeAction(() => runBatch("resume"))}
            >
              <Icon source={play} size={14} />
              Resume
            </Button>
            <Button
              size="sm"
              variant="destructive"
              onClick={() =>
                requestRemoval(
                  aria2
                    .snapshot()
                    .tasks.filter((task) => selectedGids().has(task.gid)),
                )
              }
            >
              <Icon source={trash} size={14} />
              Remove
            </Button>
          </View>
        </Show>
      </View>
      <Show when={actionError()}>
        <View role="alert" class="px-4 py-3 rounded-lg bg-danger-surface">
          <Text class="text-sm text-danger-primary">{actionError()}</Text>
        </View>
      </Show>
      <View class="min-h-0 flex-1 flex items-stretch gap-4">
        <Card class="min-w-0 h-full flex-1 rounded-xl shadow-lg">
          <CardContent class="h-full min-h-0 p-0 flex flex-col">
            <View class="h-11 flex-none px-4 flex items-center border-b border-subtle bg-surface-muted">
              <View class="w-9 flex-none">
                <Checkbox
                  aria-label="Select all visible downloads"
                  checked={
                    shown().length > 0 &&
                    shown().every((task) => selectedGids().has(task.gid))
                  }
                  indeterminate={
                    shown().some((task) => selectedGids().has(task.gid)) &&
                    !shown().every((task) => selectedGids().has(task.gid))
                  }
                  onCheckedChange={(checked) =>
                    setSelectedGids(
                      checked
                        ? new Set(shown().map((task) => task.gid))
                        : new Set(),
                    )
                  }
                />
              </View>
              <Text class="w-2/5 text-xs text-muted">NAME</Text>
              <Text class="w-1/6 text-xs text-muted">SIZE</Text>
              <Text class="w-1/4 text-xs text-muted">PROGRESS</Text>
              <Text class="flex-1 text-xs text-muted">STATUS</Text>
            </View>
            <ScrollArea class="min-h-0 flex-1">
              <For each={shown()}>
                {(task) => (
                  <View
                    class={`min-h-20 px-4 flex items-center border-b border-subtle ${selected()?.gid === task.gid ? "bg-selected" : ""}`}
                    onClick={() => setSelected(task)}
                  >
                    <View class="w-9 flex-none">
                      <Checkbox
                        aria-label={`Select ${task.name}`}
                        checked={selectedGids().has(task.gid)}
                        onCheckedChange={(checked) =>
                          toggleSelected(task.gid, checked)
                        }
                      />
                    </View>
                    <View class="w-2/5 min-w-0 flex flex-col">
                      <Text class="truncate font-medium">{task.name}</Text>
                      <Text class="text-xs text-muted">
                        {task.bittorrent ? "BitTorrent" : "HTTP"} ·{" "}
                        {task.fileCount}{" "}
                        {task.fileCount === 1 ? "file" : "files"} ·{" "}
                        {task.connections} connections
                      </Text>
                    </View>
                    <Text class="w-1/6 text-sm">
                      {formatBytes(task.totalLength)}
                    </Text>
                    <View class="w-1/4 pr-5 flex flex-col gap-1">
                      <Progress
                        value={
                          task.totalLength
                            ? (task.completedLength / task.totalLength) * 100
                            : 0
                        }
                      />
                      <Text class="text-xs text-muted">
                        {formatBytes(task.completedLength)} ·{" "}
                        {formatBytes(task.downloadSpeed)}/s · {formatEta(task)}
                      </Text>
                    </View>
                    <View class="flex-1">
                      <Badge
                        variant={
                          task.status === "active" ? "default" : "secondary"
                        }
                      >
                        {task.status}
                      </Badge>
                    </View>
                    <View class="flex gap-1">
                      <Button
                        aria-label={`Inspect ${task.name}`}
                        size="icon"
                        variant="ghost"
                        onClick={() => setSelected(task)}
                      >
                        <Icon source={inspect} size={15} />
                      </Button>
                      <Show
                        when={
                          task.status === "error" &&
                          !task.bittorrent &&
                          Boolean(task.uri)
                        }
                      >
                        <Button
                          aria-label={`Retry ${task.name}`}
                          size="icon"
                          variant="ghost"
                          onClick={() =>
                            executeAction(async () => {
                              await aria2.taskAction(task.gid, "retry");
                              await aria2.refresh();
                            })
                          }
                        >
                          <Icon source={retry} size={15} />
                        </Button>
                      </Show>
                      <Button
                        aria-label={`${task.status === "active" ? "Pause" : "Resume"} ${task.name}`}
                        size="icon"
                        variant="ghost"
                        onClick={() =>
                          executeAction(async () => {
                            await aria2.taskAction(
                              task.gid,
                              task.status === "active" ? "pause" : "resume",
                            );
                            await aria2.refresh();
                          })
                        }
                      >
                        <Icon
                          source={task.status === "active" ? pause : play}
                          size={15}
                        />
                      </Button>
                      <Button
                        aria-label={`Remove ${task.name}`}
                        size="icon"
                        variant="ghost"
                        onClick={() => requestRemoval([task])}
                      >
                        <Icon source={trash} size={15} />
                      </Button>
                    </View>
                  </View>
                )}
              </For>
              <Show when={!shown().length}>
                <View class="h-56 flex items-center justify-center">
                  <Text class="text-muted">
                    No downloads match this search.
                  </Text>
                </View>
              </Show>
            </ScrollArea>
          </CardContent>
        </Card>
        <Show when={selected()}>
          {(task) => (
            <Card class="w-80 h-full flex-none rounded-xl shadow-lg">
              <CardContent class="h-full min-h-0 p-5 flex flex-col gap-4">
                <View class="flex flex-col gap-1">
                  <Text class="text-xs font-medium text-muted">
                    TASK DETAILS
                  </Text>
                  <Text class="font-semibold whitespace-normal">
                    {task().name}
                  </Text>
                </View>
                <View class="flex gap-1 rounded-lg bg-surface-muted p-1">
                  <InspectorTab
                    label="Overview"
                    selected={detailTab() === "overview"}
                    onClick={() => setDetailTab("overview")}
                  />
                  <InspectorTab
                    label={`Files ${details()?.files.length ?? task().fileCount}`}
                    selected={detailTab() === "files"}
                    onClick={() => setDetailTab("files")}
                  />
                  <Show when={task().bittorrent}>
                    <InspectorTab
                      label={`Peers ${details()?.peers.length ?? 0}`}
                      selected={detailTab() === "peers"}
                      onClick={() => setDetailTab("peers")}
                    />
                  </Show>
                </View>
                <Show when={detailTab() === "overview"}>
                  <View class="flex flex-col gap-3">
                    <Detail label="GID" value={task().gid} />
                    <Detail label="Status" value={task().status} />
                    <Detail
                      label="Save directory"
                      value={task().dir || "Default"}
                    />
                    <Show when={task().uri}>
                      <Detail label="Source" value={task().uri ?? ""} />
                    </Show>
                    <Detail
                      label="Downloaded"
                      value={`${formatBytes(task().completedLength)} / ${formatBytes(task().totalLength)}`}
                    />
                    <Detail
                      label="Download speed"
                      value={`${formatBytes(task().downloadSpeed)}/s`}
                    />
                    <Detail
                      label="Upload speed"
                      value={`${formatBytes(task().uploadSpeed)}/s`}
                    />
                    <Show when={task().bittorrent}>
                      <Detail
                        label="Seeders"
                        value={String(task().seeders ?? 0)}
                      />
                      <Detail
                        label="Trackers"
                        value={String(details()?.trackers.length ?? 0)}
                      />
                    </Show>
                    <Show when={task().errorMessage}>
                      <Detail label="Error" value={task().errorMessage ?? ""} />
                    </Show>
                  </View>
                </Show>
                <Show when={detailTab() === "files"}>
                  <ScrollArea class="h-60 rounded-lg border border-subtle">
                    <View class="flex flex-col">
                      <For each={details()?.files ?? []}>
                        {(file) => (
                          <View class="px-3 py-2 flex items-center gap-3 border-b border-subtle">
                            <Show
                              when={
                                task().bittorrent &&
                                (details()?.files.length ?? 0) > 1
                              }
                            >
                              <Checkbox
                                aria-label={`Download ${fileName(file.path)}`}
                                checked={file.selected}
                                disabled={fileSelectionBusy()}
                                onCheckedChange={(checked) =>
                                  void toggleTaskFile(file.index, checked)
                                }
                              />
                            </Show>
                            <View class="min-w-0 flex-1 flex flex-col gap-1">
                              <Text class="truncate text-sm font-medium">
                                {fileName(file.path)}
                              </Text>
                              <Text class="text-xs text-muted">
                                {formatBytes(file.completedLength)} /{" "}
                                {formatBytes(file.length)}
                                {!file.selected ? " · skipped" : ""}
                              </Text>
                            </View>
                          </View>
                        )}
                      </For>
                      <Show when={!details() && !detailsError()}>
                        <Text class="p-3 text-sm text-muted">
                          Loading files…
                        </Text>
                      </Show>
                    </View>
                  </ScrollArea>
                </Show>
                <Show when={detailTab() === "peers"}>
                  <ScrollArea class="h-60 rounded-lg border border-subtle">
                    <View class="flex flex-col">
                      <For each={details()?.peers ?? []}>
                        {(peer) => (
                          <View class="px-3 py-2 flex items-center justify-between border-b border-subtle">
                            <View class="min-w-0 flex flex-col">
                              <Text class="truncate text-sm">
                                {peer.ip}:{peer.port}
                              </Text>
                              <Text class="text-xs text-muted">
                                ↓ {formatBytes(peer.downloadSpeed)}/s · ↑{" "}
                                {formatBytes(peer.uploadSpeed)}/s
                              </Text>
                            </View>
                            <Show when={peer.seeder}>
                              <Badge variant="outline">Seeder</Badge>
                            </Show>
                          </View>
                        )}
                      </For>
                      <Show when={details() && !details()?.peers.length}>
                        <Text class="p-3 text-sm text-muted">
                          No connected peers.
                        </Text>
                      </Show>
                      <Show when={!details() && !detailsError()}>
                        <Text class="p-3 text-sm text-muted">
                          Loading peers…
                        </Text>
                      </Show>
                    </View>
                  </ScrollArea>
                </Show>
                <Show when={detailsError()}>
                  <Text role="alert" class="text-sm text-danger-primary">
                    {detailsError()}
                  </Text>
                </Show>
                <View class="flex gap-2">
                  <Button
                    variant="outline"
                    disabled={!task().filePath && !task().dir}
                    onClick={() => {
                      const path = task().filePath || task().dir;
                      if (path) executeAction(() => aria2.openTaskFolder(path));
                    }}
                  >
                    Open folder
                  </Button>
                  <Button
                    variant="outline"
                    disabled={!task().uri}
                    onClick={() => {
                      const uri = task().uri;
                      if (uri)
                        executeAction(async () => {
                          await clipboard.writeText(uri);
                        });
                    }}
                  >
                    Copy source
                  </Button>
                </View>
                <Button
                  variant="outline"
                  onClick={() => setSelected(undefined)}
                >
                  Close inspector
                </Button>
              </CardContent>
            </Card>
          )}
        </Show>
      </View>
      <View class="h-10 flex-none px-4 flex items-center justify-between rounded-lg border border-subtle bg-surface">
        <View class="flex gap-5">
          <Text class="text-xs text-muted">
            {
              aria2.snapshot().tasks.filter((task) => task.status === "active")
                .length
            }{" "}
            active
          </Text>
          <Text class="text-xs text-muted">
            {
              aria2
                .snapshot()
                .tasks.filter(
                  (task) =>
                    task.status === "waiting" || task.status === "paused",
                ).length
            }{" "}
            waiting
          </Text>
          <Text class="text-xs text-muted">
            {
              aria2
                .snapshot()
                .tasks.filter((task) => task.status === "complete").length
            }{" "}
            completed
          </Text>
        </View>
        <View class="flex gap-5">
          <Text class="text-xs text-muted">
            ↓ {formatBytes(aria2.snapshot().downloadSpeed)}/s
          </Text>
          <Text class="text-xs text-muted">
            ↑ {formatBytes(aria2.snapshot().uploadSpeed)}/s
          </Text>
        </View>
      </View>
      <Modal
        aria-label="Remove download tasks"
        open={pendingRemoval().length > 0}
        onOpenChange={(open) => {
          if (!open) {
            setPendingRemoval([]);
            setRemoveFiles(false);
          }
        }}
        contentClass="w-96 max-w-full p-6 flex flex-col gap-4 rounded-xl border border-subtle bg-surface shadow-xl"
      >
        {({ close }) => (
          <>
            <Text class="text-xl font-semibold">Remove download?</Text>
            <Text class="whitespace-normal text-sm text-muted">
              {pendingRemoval().length === 1
                ? pendingRemoval()[0]?.name
                : `${pendingRemoval().length} selected downloads`}{" "}
              will be removed from aria2.
            </Text>
            <Checkbox
              checked={removeFiles()}
              onCheckedChange={setRemoveFiles}
              label="Also move downloaded files to Trash"
            />
            <View class="flex justify-end gap-2">
              <Button variant="ghost" onClick={close}>
                Cancel
              </Button>
              <Button
                variant="destructive"
                onClick={() => {
                  const gids = pendingRemoval().map((task) => task.gid);
                  const shouldRemoveFiles = removeFiles();
                  close();
                  executeAction(async () => {
                    await aria2.batchTaskAction(gids, "remove", {
                      removeFiles: shouldRemoveFiles,
                    });
                    setSelectedGids(new Set<string>());
                    setSelected(undefined);
                    await aria2.refresh();
                  });
                }}
              >
                Remove
              </Button>
            </View>
          </>
        )}
      </Modal>
    </View>
  );
}

function formatEta(task: Aria2Task): string {
  if (task.status === "complete") return "Done";
  if (task.status === "seeding") return "Seeding";
  const remaining = Math.max(0, task.totalLength - task.completedLength);
  if (!remaining || !task.downloadSpeed) return "—";
  const seconds = Math.ceil(remaining / task.downloadSpeed);
  if (seconds < 60) return `${seconds}s left`;
  if (seconds < 3600) return `${Math.ceil(seconds / 60)}m left`;
  return `${Math.ceil(seconds / 3600)}h left`;
}

function fileName(path: string): string {
  const segments = path.replaceAll("\\", "/").split("/");
  return segments.at(-1) || path;
}

function InspectorTab(props: {
  label: string;
  selected: boolean;
  onClick(): void;
}) {
  return (
    <Button
      class="flex-1"
      size="sm"
      variant={props.selected ? "secondary" : "ghost"}
      selected={props.selected}
      onClick={props.onClick}
    >
      {props.label}
    </Button>
  );
}

function Detail(props: { label: string; value: string }) {
  return (
    <View class="flex flex-col gap-1">
      <Text class="text-xs text-muted">{props.label}</Text>
      <Text class="text-sm whitespace-normal">{props.value}</Text>
    </View>
  );
}
