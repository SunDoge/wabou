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
  showNativeMenu,
  Text,
  TextArea,
  View,
} from "@wabou/ui";
import sortIcon from "lucide-static/icons/arrow-down-wide-narrow.svg?raw";
import inspect from "lucide-static/icons/panel-right-open.svg?raw";
import pause from "lucide-static/icons/pause.svg?raw";
import play from "lucide-static/icons/play.svg?raw";
import retry from "lucide-static/icons/rotate-ccw.svg?raw";
import stop from "lucide-static/icons/square.svg?raw";
import trash from "lucide-static/icons/trash-2.svg?raw";
import { createEffect, createMemo, createSignal, For, Show } from "solid-js";
import { type Aria2Task, type Aria2TaskDetails, useAria2 } from "../aria2";
import { LiveChart } from "../components/live-chart";
import { PieceMap } from "../components/piece-map";
import { formatBytes } from "../lib/format";
import {
  projectTasks,
  primaryTaskAction,
  primaryTaskActionLabel,
  restartTaskAction,
  type TaskFilter,
  type TaskSort,
} from "../task-list";

const sortLabels: Record<TaskSort, string> = {
  queue: "Queue order",
  name: "Name",
  size: "Largest size",
  progress: "Most progress",
  speed: "Fastest speed",
};

const isQueuedTask = (task: Aria2Task) =>
  task.status === "waiting" || task.status === "paused";

export function DownloadsPage() {
  const aria2 = useAria2();
  const [query, setQuery] = createSignal("");
  const filters: readonly [TaskFilter, string][] = [
    ["all", "All"],
    ["active", "Downloading"],
    ["waiting", "Waiting"],
    ["complete", "Completed"],
    ["stopped", "Stopped"],
  ];
  const [filter, setFilter] = createSignal<TaskFilter>("all");
  const [sort, setSort] = createSignal<TaskSort>("queue");
  const [selected, setSelected] = createSignal<Aria2Task>();
  const [selectedGids, setSelectedGids] = createSignal<ReadonlySet<string>>(
    new Set<string>(),
  );
  const selectedTasks = createMemo(() =>
    aria2.snapshot().tasks.filter((task) => selectedGids().has(task.gid)),
  );
  const [actionError, setActionError] = createSignal("");
  const [pendingRemoval, setPendingRemoval] = createSignal<Aria2Task[]>([]);
  const [removeFiles, setRemoveFiles] = createSignal(false);
  const [details, setDetails] = createSignal<Aria2TaskDetails>();
  const [detailsError, setDetailsError] = createSignal("");
  const [fileSelectionBusy, setFileSelectionBusy] = createSignal(false);
  const [taskDownloadLimit, setTaskDownloadLimit] = createSignal("0");
  const [taskUploadLimit, setTaskUploadLimit] = createSignal("0");
  const [taskLimitsBusy, setTaskLimitsBusy] = createSignal(false);
  const [taskTrackers, setTaskTrackers] = createSignal("");
  const [taskTrackersBusy, setTaskTrackersBusy] = createSignal(false);
  const [detailTab, setDetailTab] = createSignal<
    "overview" | "files" | "activity" | "pieces" | "peers" | "trackers"
  >("overview");
  let detailsRequest = 0;
  createEffect(
    () => [selected(), aria2.snapshot().tasks] as const,
    ([current, tasks]) => {
      if (!current) return;
      const latest = tasks.find((task) => task.gid === current.gid);
      if (!latest) setSelected(undefined);
      else if (latest !== current) setSelected(latest);
    },
  );
  createEffect(
    () => selected()?.gid,
    (gid) => {
      const request = ++detailsRequest;
      setDetails(undefined);
      setDetailsError("");
      setDetailTab("overview");
      if (!gid || selected()?.archived) return;
      void aria2
        .taskDetails(gid)
        .then((value) => {
          if (request !== detailsRequest) return;
          setDetails(value);
          setTaskDownloadLimit(value.maxDownloadLimit);
          setTaskUploadLimit(value.maxUploadLimit);
          setTaskTrackers(value.trackers.join("\n"));
        })
        .catch((error) => {
          if (request === detailsRequest) setDetailsError(String(error));
        });
    },
  );
  createEffect(
    () =>
      [
        selected()?.gid,
        selected()?.completedLength,
        selected()?.uploadedLength,
      ] as const,
    ([gid]) => {
      if (!gid || !details()) return;
      const request = detailsRequest;
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
  const saveTaskLimits = async () => {
    const task = selected();
    if (!task || taskLimitsBusy()) return;
    setTaskLimitsBusy(true);
    setDetailsError("");
    try {
      const value = await aria2.setTaskLimits(
        task.gid,
        taskDownloadLimit(),
        taskUploadLimit(),
      );
      setDetails(value);
      setTaskDownloadLimit(value.maxDownloadLimit);
      setTaskUploadLimit(value.maxUploadLimit);
    } catch (error) {
      setDetailsError(String(error));
    } finally {
      setTaskLimitsBusy(false);
    }
  };
  const saveTaskTrackers = async () => {
    const task = selected();
    if (!task || taskTrackersBusy()) return;
    setTaskTrackersBusy(true);
    setDetailsError("");
    try {
      const value = await aria2.setTaskTrackers(
        task.gid,
        taskTrackers().split(/\r?\n/),
      );
      setDetails(value);
      setTaskTrackers(value.trackers.join("\n"));
    } catch (error) {
      setDetailsError(String(error));
    } finally {
      setTaskTrackersBusy(false);
    }
  };
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
  const runBatch = async (action: "pause" | "resume" | "stopSeeding") => {
    const gids = selectedTasks()
      .filter((task) => primaryTaskAction(task) === action)
      .map((task) => task.gid);
    if (!gids.length) return;
    await aria2.batchTaskAction(gids, action);
    setSelectedGids(new Set<string>());
    await aria2.refresh();
  };
  const requestRemoval = (tasks: Aria2Task[]) => {
    setRemoveFiles(false);
    setPendingRemoval(tasks);
  };
  const pauseOrResume = async (task: Aria2Task) => {
    const action = primaryTaskAction(task);
    if (!action) return;
    await aria2.taskAction(task.gid, action);
    await aria2.refresh();
  };
  const retryTask = async (task: Aria2Task) => {
    await aria2.taskAction(task.gid, "retry");
    await aria2.refresh();
  };
  const moveWaitingTask = async (
    task: Aria2Task,
    position: "top" | "up" | "down" | "bottom",
  ) => {
    if (!isQueuedTask(task)) return;
    await aria2.changeTaskPosition(task.gid, position);
    await aria2.refresh();
  };
  const showTaskMenu = async (
    task: Aria2Task,
    position: { x: number; y: number },
  ) => {
    setSelected(task);
    const restartAction = restartTaskAction(task);
    const primaryAction = primaryTaskAction(task);
    const selection = await showNativeMenu({
      position,
      items: [
        {
          kind: "item",
          id: "toggle",
          label: primaryAction
            ? primaryTaskActionLabel(primaryAction)
            : "Pause",
          enabled: Boolean(primaryAction),
        },
        {
          kind: "item",
          id: "retry",
          label: restartAction === "reseed" ? "Re-seed" : "Retry",
          enabled: Boolean(restartAction),
        },
        {
          kind: "item",
          id: "queue-top",
          label: "Move to top",
          enabled: isQueuedTask(task),
        },
        {
          kind: "item",
          id: "queue-up",
          label: "Move up",
          enabled: isQueuedTask(task),
        },
        {
          kind: "item",
          id: "queue-down",
          label: "Move down",
          enabled: isQueuedTask(task),
        },
        {
          kind: "item",
          id: "queue-bottom",
          label: "Move to bottom",
          enabled: isQueuedTask(task),
        },
        { kind: "separator" },
        {
          kind: "item",
          id: "open-folder",
          label: "Open folder",
          enabled: Boolean(task.filePath || task.dir),
        },
        {
          kind: "item",
          id: "copy-source",
          label: "Copy source",
          enabled: Boolean(task.uri),
        },
        { kind: "separator" },
        { kind: "item", id: "remove", label: "Remove" },
      ],
    });
    if (selection === "toggle" && primaryAction) await pauseOrResume(task);
    else if (selection === "retry" && restartAction) await retryTask(task);
    else if (selection === "queue-top") await moveWaitingTask(task, "top");
    else if (selection === "queue-up") await moveWaitingTask(task, "up");
    else if (selection === "queue-down") await moveWaitingTask(task, "down");
    else if (selection === "queue-bottom")
      await moveWaitingTask(task, "bottom");
    else if (selection === "open-folder") {
      const path = task.filePath || task.dir;
      if (path) await aria2.openTaskFolder(path);
    } else if (selection === "copy-source") {
      if (task.uri) await clipboard.writeText(task.uri);
    } else if (selection === "remove") requestRemoval([task]);
  };
  const shown = createMemo(() =>
    projectTasks(aria2.snapshot().tasks, filter(), query(), sort()),
  );
  return (
    <View class="h-full min-h-0 flex flex-col gap-3">
      <View class="flex-none flex items-center justify-between">
        <View class="flex items-center gap-3">
          <Text role="heading" class="text-2xl font-bold">
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
          <Button
            size="sm"
            variant="ghost"
            aria-label="Sort downloads"
            onClick={() =>
              executeAction(async () => {
                const selectedSort = await showNativeMenu({
                  items: (
                    Object.entries(sortLabels) as [TaskSort, string][]
                  ).map(([id, label]) => ({
                    kind: "item" as const,
                    id,
                    label,
                    checked: sort() === id,
                  })),
                });
                if (
                  typeof selectedSort === "string" &&
                  selectedSort in sortLabels
                )
                  setSort(selectedSort as TaskSort);
              })
            }
          >
            <Icon source={sortIcon} size={14} />
            {sortLabels[sort()]}
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
              disabled={
                !selectedTasks().some(
                  (task) => primaryTaskAction(task) === "pause",
                )
              }
              onClick={() => executeAction(() => runBatch("pause"))}
            >
              <Icon source={pause} size={14} />
              Pause
            </Button>
            <Button
              size="sm"
              variant="ghost"
              disabled={
                !selectedTasks().some(
                  (task) => primaryTaskAction(task) === "resume",
                )
              }
              onClick={() => executeAction(() => runBatch("resume"))}
            >
              <Icon source={play} size={14} />
              Resume
            </Button>
            <Show
              when={selectedTasks().some(
                (task) => primaryTaskAction(task) === "stopSeeding",
              )}
            >
              <Button
                size="sm"
                variant="ghost"
                onClick={() => executeAction(() => runBatch("stopSeeding"))}
              >
                <Icon source={stop} size={14} />
                Stop seeding
              </Button>
            </Show>
            <Button
              size="sm"
              variant="destructive"
              onClick={() => requestRemoval(selectedTasks())}
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
      <View class="min-h-0 flex-1 flex items-stretch gap-3">
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
                    class={`min-h-16 px-4 flex items-center border-b border-subtle ${selected()?.gid === task.gid ? "bg-selected" : ""}`}
                    onClick={() => setSelected(task)}
                    onDblClick={() => {
                      if (task.status !== "complete") return;
                      const path = task.filePath || task.dir;
                      if (path) executeAction(() => aria2.openTaskFolder(path));
                    }}
                    onContextMenu={(event) => {
                      event.preventDefault();
                      void executeAction(() =>
                        showTaskMenu(task, {
                          x: event.clientX,
                          y: event.clientY,
                        }),
                      );
                    }}
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
                      <Show when={restartTaskAction(task)} keyed>
                        {(restartAction) => (
                          <Button
                            aria-label={`${restartAction === "reseed" ? "Re-seed" : "Retry"} ${task.name}`}
                            size="icon"
                            variant="ghost"
                            onClick={() => executeAction(() => retryTask(task))}
                          >
                            <Icon source={retry} size={15} />
                          </Button>
                        )}
                      </Show>
                      <Show when={primaryTaskAction(task)} keyed>
                        {(action) => (
                          <Button
                            aria-label={`${primaryTaskActionLabel(action)} ${task.name}`}
                            size="icon"
                            variant="ghost"
                            onClick={() =>
                              executeAction(() => pauseOrResume(task))
                            }
                          >
                            <Icon
                              source={
                                action === "stopSeeding"
                                  ? stop
                                  : action === "pause"
                                    ? pause
                                    : play
                              }
                              size={15}
                            />
                          </Button>
                        )}
                      </Show>
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
            <Card class="w-72 h-full flex-none rounded-xl shadow-lg">
              <CardContent class="h-full min-h-0 p-4 flex flex-col gap-3">
                <View class="flex flex-col gap-1">
                  <Text class="text-xs font-medium text-muted">
                    TASK DETAILS
                  </Text>
                  <Text class="font-semibold whitespace-normal">
                    {task().name}
                  </Text>
                </View>
                <View class="grid grid-cols-2 gap-1 rounded-lg bg-surface-muted p-1">
                  <InspectorTab
                    label="Overview"
                    ariaLabel="Task overview"
                    selected={detailTab() === "overview"}
                    onClick={() => setDetailTab("overview")}
                  />
                  <Show when={!task().archived}>
                    <InspectorTab
                      label={`Files ${details()?.files.length ?? task().fileCount}`}
                      ariaLabel="Task files"
                      selected={detailTab() === "files"}
                      onClick={() => setDetailTab("files")}
                    />
                  </Show>
                  <InspectorTab
                    label="Activity"
                    ariaLabel="Task activity"
                    selected={detailTab() === "activity"}
                    onClick={() => setDetailTab("activity")}
                  />
                  <Show when={task().bittorrent && !task().archived}>
                    <InspectorTab
                      label="Pieces"
                      ariaLabel="Task pieces"
                      selected={detailTab() === "pieces"}
                      onClick={() => setDetailTab("pieces")}
                    />
                    <InspectorTab
                      label={`Peers ${details()?.peers.length ?? 0}`}
                      ariaLabel="Task peers"
                      selected={detailTab() === "peers"}
                      onClick={() => setDetailTab("peers")}
                    />
                    <InspectorTab
                      label="Trackers"
                      ariaLabel="Task trackers"
                      selected={detailTab() === "trackers"}
                      onClick={() => setDetailTab("trackers")}
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
                    <Show
                      when={
                        task().status !== "complete" &&
                        task().status !== "removed" &&
                        task().status !== "error"
                      }
                    >
                      <View class="pt-2 flex flex-col gap-2 border-t border-subtle">
                        <Text class="text-xs font-medium text-muted">
                          TASK SPEED LIMITS
                        </Text>
                        <Input
                          aria-label="Task download limit"
                          value={taskDownloadLimit()}
                          placeholder="0 or 10M"
                          onInput={(event) =>
                            setTaskDownloadLimit(event.currentTarget.value)
                          }
                        />
                        <Input
                          aria-label="Task upload limit"
                          value={taskUploadLimit()}
                          placeholder="0 or 1M"
                          onInput={(event) =>
                            setTaskUploadLimit(event.currentTarget.value)
                          }
                        />
                        <Button
                          size="sm"
                          variant="outline"
                          disabled={taskLimitsBusy()}
                          onClick={() => void saveTaskLimits()}
                        >
                          {taskLimitsBusy() ? "Saving…" : "Save task limits"}
                        </Button>
                      </View>
                    </Show>
                    <Show when={isQueuedTask(task())}>
                      <View class="pt-2 flex flex-col gap-2 border-t border-subtle">
                        <Text class="text-xs font-medium text-muted">
                          QUEUE POSITION
                        </Text>
                        <View class="grid grid-cols-2 gap-2">
                          <Button
                            size="sm"
                            variant="outline"
                            onClick={() =>
                              executeAction(() =>
                                moveWaitingTask(task(), "top"),
                              )
                            }
                          >
                            Move to top
                          </Button>
                          <Button
                            size="sm"
                            variant="outline"
                            onClick={() =>
                              executeAction(() => moveWaitingTask(task(), "up"))
                            }
                          >
                            Move up
                          </Button>
                          <Button
                            size="sm"
                            variant="outline"
                            onClick={() =>
                              executeAction(() =>
                                moveWaitingTask(task(), "down"),
                              )
                            }
                          >
                            Move down
                          </Button>
                          <Button
                            size="sm"
                            variant="outline"
                            onClick={() =>
                              executeAction(() =>
                                moveWaitingTask(task(), "bottom"),
                              )
                            }
                          >
                            Move to bottom
                          </Button>
                        </View>
                      </View>
                    </Show>
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
                <Show when={detailTab() === "activity"}>
                  <View class="flex flex-col gap-3">
                    <View class="grid grid-cols-2 gap-2">
                      <View class="p-3 flex flex-col gap-1 rounded-lg bg-surface-muted">
                        <Text class="text-xs text-muted">Downloaded</Text>
                        <Text class="text-sm font-semibold">
                          {formatBytes(task().completedLength)}
                        </Text>
                        <Text class="text-xs text-muted">
                          {formatBytes(task().downloadSpeed)}/s now
                        </Text>
                      </View>
                      <View class="p-3 flex flex-col gap-1 rounded-lg bg-surface-muted">
                        <Text class="text-xs text-muted">Uploaded</Text>
                        <Text class="text-sm font-semibold">
                          {formatBytes(task().uploadedLength)}
                        </Text>
                        <Text class="text-xs text-muted">
                          {formatBytes(task().uploadSpeed)}/s now
                        </Text>
                      </View>
                    </View>
                    <View class="flex flex-col gap-2">
                      <View class="flex items-center gap-3">
                        <View class="flex items-center gap-1">
                          <View class="w-2 h-2 rounded-full bg-chart-download" />
                          <Text class="text-xs text-muted">Download</Text>
                        </View>
                        <View class="flex items-center gap-1">
                          <View class="w-2 h-2 rounded-full bg-chart-upload" />
                          <Text class="text-xs text-muted">Upload</Text>
                        </View>
                      </View>
                      <View class="relative h-24">
                        <LiveChart
                          width={248}
                          values={aria2.taskHistory(task().gid).download}
                        />
                        <View class="absolute inset-0">
                          <LiveChart
                            width={248}
                            color="upload"
                            values={aria2.taskHistory(task().gid).upload}
                          />
                        </View>
                      </View>
                    </View>
                    <Text class="text-xs text-muted">
                      Session history · up to 120 snapshots
                    </Text>
                  </View>
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
                <Show when={detailTab() === "pieces"}>
                  <Show
                    when={details()}
                    fallback={
                      <Text class="p-3 text-sm text-muted">
                        Loading piece map…
                      </Text>
                    }
                  >
                    {(value) => (
                      <PieceMap
                        bitfield={value().bitfield}
                        pieceCount={value().numPieces}
                        pieceLength={value().pieceLength}
                      />
                    )}
                  </Show>
                </Show>
                <Show when={detailTab() === "trackers"}>
                  <View class="flex flex-col gap-2">
                    <TextArea
                      class="h-52"
                      aria-label="Task tracker URLs"
                      value={taskTrackers()}
                      placeholder="One HTTP, HTTPS, or UDP tracker per line"
                      disabled={!details() || taskTrackersBusy()}
                      onInput={(event) =>
                        setTaskTrackers(event.currentTarget.value)
                      }
                    />
                    <Text class="text-xs text-muted">
                      Empty lines and lines beginning with # are ignored.
                    </Text>
                    <Button
                      size="sm"
                      variant="outline"
                      disabled={!details() || taskTrackersBusy()}
                      onClick={() => void saveTaskTrackers()}
                    >
                      {taskTrackersBusy() ? "Saving…" : "Save task trackers"}
                    </Button>
                  </View>
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
  ariaLabel?: string;
  selected: boolean;
  onClick(): void;
}) {
  return (
    <Button
      aria-label={props.ariaLabel}
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
