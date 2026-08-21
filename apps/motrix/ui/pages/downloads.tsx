import {
  AdaptiveSplitPane,
  AdaptiveSplitPaneDetail,
  AdaptiveSplitPaneMain,
  Badge,
  Button,
  Card,
  CardContent,
  Checkbox,
  clipboard,
  createContainerMatch,
  createKeyedAsyncAction,
  createKeyedSelection,
  createLatestAsyncResource,
  createWindowMatch,
  Empty,
  EmptyContent,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
  Icon,
  Input,
  isDirectEvent,
  Modal,
  PageHeader,
  Progress,
  ScrollArea,
  Select,
  showNativeMenu,
  Tabs,
  TabsList,
  TabsTrigger,
  Text,
  ToggleGroup,
  ToggleGroupItem,
  View,
  VirtualList,
} from "@wabou/ui";
import sortIcon from "lucide-static/icons/arrow-down-wide-narrow.svg?raw";
import inspect from "lucide-static/icons/panel-right-open.svg?raw";
import pause from "lucide-static/icons/pause.svg?raw";
import play from "lucide-static/icons/play.svg?raw";
import plus from "lucide-static/icons/plus.svg?raw";
import retry from "lucide-static/icons/rotate-ccw.svg?raw";
import stop from "lucide-static/icons/square.svg?raw";
import trash from "lucide-static/icons/trash-2.svg?raw";
import { createEffect, createMemo, createSignal, For, Show } from "solid-js";
import { useAppActions } from "../app-actions";
import { LiveChart } from "../components/live-chart";
import {
  type DownloadTask,
  type DownloadTaskDetails,
  type TaskPriority,
  useDownloads,
} from "../downloads";
import { formatBytes, formatDateTime } from "../lib/format";
import {
  primaryTaskAction,
  primaryTaskActionLabel,
  projectTasks,
  restartTaskAction,
  type TaskFilter,
  type TaskSort,
  taskPathActions,
  taskStatusPresentation,
} from "../task-list";

const sortLabels: Record<TaskSort, string> = {
  newest: "Newest first",
  oldest: "Oldest first",
  priority: "Priority",
  name: "Name",
  size: "Largest size",
  progress: "Most progress",
  speed: "Fastest speed",
};

const priorityOptions: readonly { value: TaskPriority; label: string }[] = [
  { value: "low", label: "Low" },
  { value: "normal", label: "Normal" },
  { value: "high", label: "High" },
  { value: "critical", label: "Critical" },
];

export function DownloadsPage() {
  const downloads = useDownloads();
  const appActions = useAppActions();
  const compactToolbar = createWindowMatch({ maxWidth: 1100 });
  const narrow = createWindowMatch({ maxWidth: 800 });
  // The fixed sidebar consumes roughly 230px. Compact only the table before
  // its content area can no longer hold metadata, progress and actions; the
  // surrounding toolbar has a separate, smaller responsive breakpoint.
  const compactTable = createContainerMatch({ maxWidth: 760 });
  const [query, setQuery] = createSignal("");
  const filters: readonly [TaskFilter, string][] = [
    ["all", "All"],
    ["active", "Downloading"],
    ["waiting", "Waiting"],
    ["complete", "Completed"],
    ["stopped", "Stopped"],
  ];
  const [filter, setFilter] = createSignal<TaskFilter>("all");
  const [sort, setSort] = createSignal<TaskSort>("newest");
  const taskSource = () => downloads.snapshot().tasks;
  const inspectorSelection = createKeyedSelection({
    items: taskSource,
    key: (task) => task.id,
    mode: "single",
  });
  const batchSelection = createKeyedSelection({
    items: taskSource,
    key: (task) => task.id,
    mode: "multiple",
  });
  const removalSelection = createKeyedSelection({
    items: taskSource,
    key: (task) => task.id,
    mode: "multiple",
  });
  const selected = inspectorSelection.item;
  const selectedIds = batchSelection.keys;
  const selectedTasks = batchSelection.items;
  const [actionError, setActionError] = createSignal("");
  const [actionNotice, setActionNotice] = createSignal("");
  const pendingRemoval = removalSelection.items;
  const [removeFiles, setRemoveFiles] = createSignal(false);
  const [detailTab, setDetailTab] = createSignal<
    "overview" | "files" | "activity"
  >("overview");
  createEffect(downloads.requestedTaskId, (id) => {
    if (!id) return;
    inspectorSelection.select(id);
    downloads.clearTaskInspectionRequest();
  });
  const detailsResource = createLatestAsyncResource<
    string,
    DownloadTaskDetails
  >({
    source: () =>
      detailTab() === "files" && !selected()?.archived
        ? selected()?.id
        : undefined,
    load: (id) => downloads.taskDetails(id),
  });
  const details = detailsResource.value;
  const detailsError = () =>
    detailsResource.error() ? String(detailsResource.error()) : "";
  createEffect(
    () => selected()?.id,
    () => {
      setDetailTab("overview");
      setActionNotice("");
    },
  );
  createEffect(
    () =>
      [
        detailTab(),
        selected()?.id,
        selected()?.completedLength,
        selected()?.uploadedLength,
      ] as const,
    ([tab, id]) => {
      if (tab !== "files" || !id || !details()) return;
      void detailsResource.refresh();
    },
  );
  const taskActions = createKeyedAsyncAction(
    (key: string, _operation: () => Promise<void>) => key,
    (_key: string, operation: () => Promise<void>) => operation(),
  );
  const executeAction = async (
    key: string,
    action: () => Promise<void>,
    successMessage?: string,
  ) => {
    setActionError("");
    setActionNotice("");
    const outcome = await taskActions.run(key, action);
    if (!outcome.ok) setActionError(String(outcome.error));
    else if (successMessage) setActionNotice(successMessage);
  };
  const toggleSelected = (id: string, checked: boolean) =>
    checked ? batchSelection.select(id) : batchSelection.deselect(id);
  const runBatch = async (action: "pause" | "resume" | "stopSeeding") => {
    const ids = selectedTasks()
      .filter((task) => primaryTaskAction(task) === action)
      .map((task) => task.id);
    if (!ids.length) return;
    await downloads.batchTaskAction(ids, action);
    batchSelection.clear();
  };
  const requestRemoval = (tasks: readonly DownloadTask[]) => {
    setRemoveFiles(false);
    removalSelection.set(tasks.map((task) => task.id));
  };
  const pauseOrResume = async (task: DownloadTask) => {
    const action = primaryTaskAction(task);
    if (!action) return;
    await downloads.taskAction(task.id, action);
  };
  const retryTask = async (task: DownloadTask) => {
    await downloads.taskAction(task.id, "retry");
  };
  const showTaskMenu = async (
    task: DownloadTask,
    position: { x: number; y: number },
  ) => {
    inspectorSelection.select(task.id);
    const restartAction = restartTaskAction(task);
    const primaryAction = primaryTaskAction(task);
    const pathActions = taskPathActions(task);
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
          id: "open-file",
          label: "Open file",
          enabled: pathActions.openFile,
        },
        {
          kind: "item",
          id: "show-in-folder",
          label: "Show in folder",
          enabled: pathActions.showInFolder,
        },
        {
          kind: "item",
          id: "copy-source",
          label: "Copy source",
          enabled: Boolean(task.uri),
        },
        {
          kind: "submenu",
          label: "Priority",
          items: priorityOptions.map(({ value, label }) => ({
            kind: "item" as const,
            id: `priority-${value}`,
            label,
            checked: task.priority === value,
          })),
        },
        { kind: "separator" },
        { kind: "item", id: "remove", label: "Remove" },
      ],
    });
    if (selection === "toggle" && primaryAction) await pauseOrResume(task);
    else if (selection === "retry" && restartAction) await retryTask(task);
    else if (selection === "open-file") {
      if (task.filePath) {
        await downloads.openPath(task.filePath);
        setActionNotice(`Opened ${task.name}.`);
      }
    } else if (selection === "show-in-folder") {
      const path = task.filePath || task.dir;
      if (path) {
        await downloads.openTaskFolder(path);
        setActionNotice(`Opened the folder containing ${task.name}.`);
      }
    } else if (selection === "copy-source") {
      if (task.uri) {
        await clipboard.writeText(task.uri);
        setActionNotice(`Copied the source for ${task.name}.`);
      }
    } else if (selection.startsWith("priority-")) {
      const priority = selection.slice("priority-".length) as TaskPriority;
      if (priorityOptions.some((option) => option.value === priority)) {
        await downloads.setTaskPriority(task.id, priority);
        setActionNotice(`${task.name} priority set to ${priority}.`);
      }
    } else if (selection === "remove") requestRemoval([task]);
  };
  const shown = createMemo(() =>
    projectTasks(downloads.snapshot().tasks, filter(), query(), sort()),
  );
  const stoppedTasks = createMemo(() =>
    downloads
      .snapshot()
      .tasks.filter(
        (task) => task.status === "error" || task.status === "removed",
      ),
  );
  return (
    <View class="h-full min-h-0 flex flex-col gap-3">
      <PageHeader
        title="All Downloads"
        stacked={compactToolbar()}
        titleAdornment={
          <Badge variant="outline">
            {shown().length}/{downloads.snapshot().tasks.length}
          </Badge>
        }
        actions={
          <View
            class="w-full flex gap-2"
            classList={{ "flex-col items-stretch": narrow() }}
          >
            <View class="flex flex-row items-center gap-1">
              <Button
                size="sm"
                variant="ghost"
                disabled={
                  downloads.snapshot().status !== "ready" ||
                  taskActions.pending("global")
                }
                onClick={() =>
                  executeAction("global", async () => {
                    await downloads.globalTaskAction("pauseAll");
                  })
                }
              >
                <Icon source={pause} size={14} />
                Pause all
              </Button>
              <Button
                size="sm"
                variant="ghost"
                disabled={
                  downloads.snapshot().status !== "ready" ||
                  taskActions.pending("global")
                }
                onClick={() =>
                  executeAction("global", async () => {
                    await downloads.globalTaskAction("resumeAll");
                  })
                }
              >
                <Icon source={play} size={14} />
                Resume all
              </Button>
              <Button
                size="sm"
                variant="ghost"
                aria-label={`Sort downloads: ${sortLabels[sort()]}`}
                disabled={taskActions.pending("sort")}
                onClick={() =>
                  executeAction("sort", async () => {
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
            </View>
            <View
              classList={{
                "w-56": !compactToolbar(),
                "min-w-32 flex-1": compactToolbar(),
              }}
            >
              <Input
                aria-label="Search downloads"
                value={query()}
                placeholder="Search downloads"
                onInput={(event) => setQuery(event.currentTarget.value)}
              />
            </View>
          </View>
        }
      />
      <View class="flex-none flex items-center justify-between gap-3">
        <Show
          when={narrow()}
          fallback={
            <ToggleGroup
              type="single"
              value={filter()}
              aria-label="Download status"
              class="gap-2 bg-transparent p-0"
              onValueChange={(value) => {
                setFilter(value as TaskFilter);
                inspectorSelection.clear();
                batchSelection.clear();
              }}
            >
              <For each={filters}>
                {([value, label]) => (
                  <ToggleGroupItem
                    value={value}
                    variant="accent"
                    class="flex-none"
                  >
                    {label}
                  </ToggleGroupItem>
                )}
              </For>
            </ToggleGroup>
          }
        >
          <Select
            aria-label="Download status"
            class="w-44"
            value={filter()}
            options={filters.map(([value, label]) => ({ value, label }))}
            onValueChange={(value) => {
              setFilter(value as TaskFilter);
              inspectorSelection.clear();
              batchSelection.clear();
            }}
          />
        </Show>
        <Show
          when={selectedIds().size > 0}
          fallback={
            <View class="flex items-center gap-2">
              <Show when={selected()}>
                {(task) => (
                  <Text class="text-sm text-muted">
                    Inspecting · {task().name}
                  </Text>
                )}
              </Show>
              <Show when={stoppedTasks().length > 0}>
                <Button
                  size="sm"
                  variant="ghost"
                  disabled={taskActions.pending("batch")}
                  onClick={() => requestRemoval(stoppedTasks())}
                >
                  <Icon source={trash} size={14} />
                  Clear stopped
                </Button>
              </Show>
            </View>
          }
        >
          <View class="flex items-center gap-2">
            <Text
              role="status"
              aria-label={`${selectedIds().size} downloads selected`}
              class="text-sm text-muted"
            >
              {selectedIds().size} selected
            </Text>
            <Button
              size="sm"
              variant="ghost"
              disabled={
                taskActions.pending("batch") ||
                !selectedTasks().some(
                  (task) => primaryTaskAction(task) === "pause",
                )
              }
              onClick={() => executeAction("batch", () => runBatch("pause"))}
            >
              <Icon source={pause} size={14} />
              Pause
            </Button>
            <Button
              size="sm"
              variant="ghost"
              disabled={
                taskActions.pending("batch") ||
                !selectedTasks().some(
                  (task) => primaryTaskAction(task) === "resume",
                )
              }
              onClick={() => executeAction("batch", () => runBatch("resume"))}
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
                disabled={taskActions.pending("batch")}
                onClick={() =>
                  executeAction("batch", () => runBatch("stopSeeding"))
                }
              >
                <Icon source={stop} size={14} />
                Stop seeding
              </Button>
            </Show>
            <Button
              size="sm"
              variant="destructive"
              disabled={taskActions.pending("batch")}
              onClick={() => requestRemoval(selectedTasks())}
            >
              <Icon source={trash} size={14} />
              Remove
            </Button>
          </View>
        </Show>
      </View>
      <Show when={actionError()}>
        <View
          role="alert"
          aria-label="Download action failed"
          class="px-4 py-3 rounded-lg bg-danger-surface"
        >
          <Text class="text-sm text-danger-primary">{actionError()}</Text>
        </View>
      </Show>
      <Show when={!selected() && !actionError() && actionNotice()}>
        <View
          role="status"
          aria-label="Download action completed"
          class="px-4 py-3 rounded-lg bg-success-surface"
        >
          <Text class="text-sm text-success-primary">{actionNotice()}</Text>
        </View>
      </Show>
      <AdaptiveSplitPane
        compact={compactToolbar()}
        class="min-h-0 flex-1 gap-3"
      >
        <AdaptiveSplitPaneMain class="h-full">
          <Card
            ref={compactTable.ref}
            role="group"
            aria-label="Download list"
            class="min-w-0 h-full rounded-xl shadow-lg"
          >
            <CardContent class="h-full min-h-0 p-0 flex flex-col">
              <View class="h-11 flex-none px-4 flex items-center border-b border-subtle bg-surface-muted">
                <View class="w-9 flex-none">
                  <Checkbox
                    aria-label="Select all visible downloads"
                    checked={
                      shown().length > 0 &&
                      shown().every((task) => selectedIds().has(task.id))
                    }
                    indeterminate={
                      shown().some((task) => selectedIds().has(task.id)) &&
                      !shown().every((task) => selectedIds().has(task.id))
                    }
                    onCheckedChange={(checked) =>
                      checked
                        ? batchSelection.set(shown().map((task) => task.id))
                        : batchSelection.clear()
                    }
                  />
                </View>
                <Text
                  class={
                    compactTable.matches()
                      ? "min-w-0 flex-1 text-xs text-muted"
                      : "w-2/5 text-xs text-muted"
                  }
                >
                  NAME
                </Text>
                <Show when={!compactTable.matches()}>
                  <Text class="w-1/6 text-xs text-muted">SIZE</Text>
                  <Text class="w-1/4 text-xs text-muted">PROGRESS</Text>
                </Show>
                <Text class="w-20 flex-none text-xs text-muted">STATUS</Text>
              </View>
              <Show
                when={shown().length > 0}
                fallback={
                  <Show
                    when={!query().trim()}
                    fallback={
                      <Empty
                        variant="plain"
                        class={narrow() ? "flex-1 p-2 gap-2" : "flex-1"}
                      >
                        <EmptyHeader>
                          <EmptyTitle>No matching downloads</EmptyTitle>
                          <EmptyDescription>
                            Try another name or clear the current search.
                          </EmptyDescription>
                        </EmptyHeader>
                        <EmptyContent>
                          <Button
                            variant="outline"
                            onClick={() => setQuery("")}
                          >
                            Clear search
                          </Button>
                        </EmptyContent>
                      </Empty>
                    }
                  >
                    <Empty
                      variant="plain"
                      class={narrow() ? "flex-1 p-2 gap-2" : "flex-1"}
                    >
                      <Show when={!narrow()}>
                        <EmptyMedia
                          variant="icon"
                          class="rounded-full bg-selected"
                        >
                          <Icon source={plus} size={22} class="text-accent" />
                        </EmptyMedia>
                      </Show>
                      <EmptyHeader>
                        <EmptyTitle>Start your first download</EmptyTitle>
                        <EmptyDescription class="max-w-md">
                          Add one or more links, or inspect a torrent before
                          creating the task.
                        </EmptyDescription>
                      </EmptyHeader>
                      <EmptyContent>
                        <Button
                          aria-label="Add a download"
                          onClick={appActions.openAddTask}
                        >
                          <Icon source={plus} size={15} />
                          Add download
                        </Button>
                      </EmptyContent>
                    </Empty>
                  </Show>
                }
              >
                <VirtualList
                  class="min-h-0 flex-1"
                  items={shown}
                  getItemKey={(task) => task.id}
                  itemHeight={64}
                  overscan={4}
                  accessibilityLabel="Downloads"
                >
                  {(task) => (
                    <View
                      class={`h-16 px-4 flex items-center border-b border-subtle ${selected()?.id === task().id ? "bg-selected" : ""}`}
                      onClick={(event) => {
                        if (isDirectEvent(event))
                          inspectorSelection.select(task().id);
                      }}
                      onDblClick={() => {
                        if (task().status !== "complete") return;
                        const filePath = task().filePath;
                        if (filePath)
                          executeAction(
                            task().id,
                            () => downloads.openPath(filePath),
                            `Opened ${task().name}.`,
                          );
                        else if (task().dir)
                          executeAction(
                            task().id,
                            () => downloads.openTaskFolder(task().dir),
                            `Opened the folder containing ${task().name}.`,
                          );
                      }}
                      onContextMenu={(event) => {
                        event.preventDefault();
                        void executeAction(task().id, () =>
                          showTaskMenu(task(), {
                            x: event.clientX,
                            y: event.clientY,
                          }),
                        );
                      }}
                    >
                      <View class="w-9 flex-none">
                        <Checkbox
                          aria-label={`Select ${task().name}`}
                          checked={selectedIds().has(task().id)}
                          onCheckedChange={(checked) =>
                            toggleSelected(task().id, checked)
                          }
                        />
                      </View>
                      <View
                        role="group"
                        aria-label={`Task identity: ${task().name}`}
                        class={
                          compactTable.matches()
                            ? "min-w-0 flex-1 flex flex-col"
                            : "w-2/5 min-w-0 flex flex-col"
                        }
                      >
                        <View class="min-w-0 flex items-center gap-2">
                          <Text class="min-w-0 flex-1 truncate font-medium">
                            {task().name}
                          </Text>
                          <Show when={task().priority !== "normal"}>
                            <Badge variant="outline" class="flex-none">
                              {priorityOptions.find(
                                (option) => option.value === task().priority,
                              )?.label ?? task().priority}{" "}
                              priority
                            </Badge>
                          </Show>
                        </View>
                        <Text class="text-xs text-muted">
                          {task().bittorrent ? "BitTorrent" : "HTTP"} ·{" "}
                          {task().fileCount}{" "}
                          {task().fileCount === 1 ? "file" : "files"} ·{" "}
                          {task().connections} connections
                        </Text>
                      </View>
                      <Show when={!compactTable.matches()}>
                        <Text class="w-1/6 text-sm">
                          {formatBytes(task().totalLength)}
                        </Text>
                        <View class="w-1/4 pr-5 flex flex-col gap-1">
                          <Progress
                            value={
                              task().totalLength
                                ? (task().completedLength /
                                    task().totalLength) *
                                  100
                                : 0
                            }
                          />
                          <Text class="text-xs text-muted">
                            {formatBytes(task().completedLength)} ·{" "}
                            {formatBytes(task().downloadSpeed)}/s ·{" "}
                            {formatEta(task())}
                          </Text>
                        </View>
                      </Show>
                      <View
                        role="group"
                        aria-label={`Task status: ${task().name}`}
                        class="w-20 flex-none"
                      >
                        <Badge
                          variant={
                            taskStatusPresentation(task().status).variant
                          }
                        >
                          {taskStatusPresentation(task().status).label}
                        </Badge>
                      </View>
                      <View class="flex gap-1">
                        <Button
                          aria-label={`Inspect ${task().name}`}
                          size="icon"
                          variant="ghost"
                          onClick={() => inspectorSelection.select(task().id)}
                        >
                          <Icon source={inspect} size={15} />
                        </Button>
                        <Show when={restartTaskAction(task())} keyed>
                          {(restartAction) => (
                            <Button
                              aria-label={`${restartAction === "reseed" ? "Re-seed" : "Retry"} ${task().name}`}
                              size="icon"
                              variant="ghost"
                              disabled={taskActions.pending(task().id)}
                              onClick={() =>
                                executeAction(task().id, () =>
                                  retryTask(task()),
                                )
                              }
                            >
                              <Icon source={retry} size={15} />
                            </Button>
                          )}
                        </Show>
                        <Show when={primaryTaskAction(task())} keyed>
                          {(action) => (
                            <Button
                              aria-label={`${primaryTaskActionLabel(action)} ${task().name}`}
                              size="icon"
                              variant="ghost"
                              disabled={taskActions.pending(task().id)}
                              onClick={() =>
                                executeAction(task().id, () =>
                                  pauseOrResume(task()),
                                )
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
                          aria-label={`Remove ${task().name}`}
                          size="icon"
                          variant="ghost"
                          disabled={taskActions.pending(task().id)}
                          onClick={() => requestRemoval([task()])}
                        >
                          <Icon source={trash} size={15} />
                        </Button>
                      </View>
                    </View>
                  )}
                </VirtualList>
              </Show>
            </CardContent>
          </Card>
        </AdaptiveSplitPaneMain>
        <Show when={selected()}>
          {(task) => (
            <AdaptiveSplitPaneDetail
              open={true}
              onOpenChange={(open) => {
                if (!open) inspectorSelection.clear();
              }}
              aria-label={`Task details: ${task().name}`}
              class="w-72 h-full"
              modalClass="w-4/5 max-w-full"
            >
              <Card class="w-full h-full rounded-xl shadow-lg">
                <CardContent class="h-full min-h-0 p-4 flex flex-col gap-3">
                  <View class="flex flex-col gap-1">
                    <Text class="text-xs font-medium text-muted">
                      TASK DETAILS
                    </Text>
                    <Text class="font-semibold whitespace-normal">
                      {task().name}
                    </Text>
                  </View>
                  <Tabs
                    class="gap-0"
                    value={detailTab()}
                    onValueChange={(value) =>
                      setDetailTab(value as "overview" | "files" | "activity")
                    }
                  >
                    <TabsList aria-label="Task detail sections" class="w-full">
                      <TabsTrigger
                        value="overview"
                        aria-label="Task overview"
                        class="min-w-0 flex-1"
                      >
                        Overview
                      </TabsTrigger>
                      <Show when={!task().archived}>
                        <TabsTrigger
                          value="files"
                          aria-label="Task files"
                          class="min-w-0 flex-1"
                        >
                          {`Files ${details()?.files.length ?? task().fileCount}`}
                        </TabsTrigger>
                      </Show>
                      <TabsTrigger
                        value="activity"
                        aria-label="Task activity"
                        class="min-w-0 flex-1"
                      >
                        Activity
                      </TabsTrigger>
                    </TabsList>
                  </Tabs>
                  <ScrollArea class="min-h-0 flex-1">
                    <View class="flex flex-col gap-3 pr-2">
                      <Show when={detailTab() === "overview"}>
                        <View class="flex flex-col gap-3">
                          <Detail label="Task ID" value={task().id} />
                          <TaskStatusDetail status={task().status} />
                          <Detail
                            label="Added"
                            value={formatDateTime(task().createdAtMs)}
                          />
                          <View class="flex flex-col gap-1">
                            <Text class="text-xs text-muted">Priority</Text>
                            <Select
                              aria-label="Task priority"
                              class="w-full"
                              value={task().priority}
                              options={[...priorityOptions]}
                              disabled={taskActions.pending(task().id)}
                              onValueChange={(value) =>
                                executeAction(
                                  task().id,
                                  () =>
                                    downloads.setTaskPriority(
                                      task().id,
                                      value as TaskPriority,
                                    ),
                                  `${task().name} priority set to ${value}.`,
                                )
                              }
                            />
                          </View>
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
                          </Show>
                          <Show when={task().errorMessage}>
                            <Detail
                              label="Error"
                              value={task().errorMessage ?? ""}
                            />
                          </Show>
                        </View>
                      </Show>
                      <Show when={detailTab() === "files"}>
                        <View class="flex flex-col rounded-lg border border-subtle">
                          <View
                            role="table"
                            aria-label="Downloaded files"
                            class="flex flex-col"
                          >
                            <For each={details()?.files ?? []}>
                              {(file) => (
                                <View
                                  role="row"
                                  aria-label={fileName(file.path)}
                                  class="px-3 py-2 flex items-center gap-3 border-b border-subtle"
                                >
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
                        </View>
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
                                values={
                                  downloads.taskHistory(task().id).download
                                }
                              />
                              <View class="absolute inset-0">
                                <LiveChart
                                  color="upload"
                                  grid={false}
                                  values={
                                    downloads.taskHistory(task().id).upload
                                  }
                                />
                              </View>
                            </View>
                          </View>
                          <Text class="text-xs text-muted">
                            Session history · up to 120 snapshots
                          </Text>
                        </View>
                      </Show>
                      <Show when={detailsError()}>
                        <Text role="alert" class="text-sm text-danger-primary">
                          {detailsError()}
                        </Text>
                      </Show>
                    </View>
                  </ScrollArea>
                  <Show when={!actionError() && actionNotice()}>
                    <View
                      role="status"
                      aria-label="Download action completed"
                      class="px-3 py-2 rounded-lg bg-success-surface"
                    >
                      <Text class="text-xs text-success-primary">
                        {actionNotice()}
                      </Text>
                    </View>
                  </Show>
                  <View
                    class={
                      compactToolbar()
                        ? "flex flex-row gap-2"
                        : "flex flex-col gap-2"
                    }
                  >
                    <Show when={taskPathActions(task()).openFile}>
                      <Button
                        variant="outline"
                        class={compactToolbar() ? "" : "w-full"}
                        disabled={taskActions.pending(task().id)}
                        onClick={() => {
                          const path = task().filePath;
                          if (path)
                            executeAction(
                              task().id,
                              () => downloads.openPath(path),
                              `Opened ${task().name}.`,
                            );
                        }}
                      >
                        Open file
                      </Button>
                    </Show>
                    <Button
                      variant="outline"
                      class={compactToolbar() ? "" : "w-full"}
                      disabled={
                        taskActions.pending(task().id) ||
                        (!task().filePath && !task().dir)
                      }
                      onClick={() => {
                        const path = task().filePath || task().dir;
                        if (path)
                          executeAction(
                            task().id,
                            () => downloads.openTaskFolder(path),
                            `Opened the folder containing ${task().name}.`,
                          );
                      }}
                    >
                      Show in folder
                    </Button>
                    <Button
                      variant="outline"
                      class={compactToolbar() ? "" : "w-full"}
                      disabled={taskActions.pending("clipboard") || !task().uri}
                      onClick={() => {
                        const uri = task().uri;
                        if (uri)
                          executeAction(
                            "clipboard",
                            async () => {
                              await clipboard.writeText(uri);
                            },
                            `Copied the source for ${task().name}.`,
                          );
                      }}
                    >
                      Copy source
                    </Button>
                  </View>
                  <Button
                    variant="outline"
                    onClick={() => {
                      setActionNotice("");
                      inspectorSelection.clear();
                    }}
                  >
                    Close inspector
                  </Button>
                </CardContent>
              </Card>
            </AdaptiveSplitPaneDetail>
          )}
        </Show>
      </AdaptiveSplitPane>
      <View class="h-10 flex-none px-4 flex items-center justify-between rounded-lg border border-subtle bg-surface">
        <View class="flex gap-5">
          <Text class="text-xs text-muted">
            {
              downloads
                .snapshot()
                .tasks.filter((task) => task.status === "active").length
            }{" "}
            active
          </Text>
          <Text class="text-xs text-muted">
            {
              downloads
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
              downloads
                .snapshot()
                .tasks.filter((task) => task.status === "complete").length
            }{" "}
            completed
          </Text>
        </View>
        <View class="flex gap-5">
          <Text class="text-xs text-muted">
            ↓ {formatBytes(downloads.snapshot().downloadSpeed)}/s
          </Text>
          <Text class="text-xs text-muted">
            ↑ {formatBytes(downloads.snapshot().uploadSpeed)}/s
          </Text>
        </View>
      </View>
      <Modal
        aria-label="Remove download tasks"
        open={pendingRemoval().length > 0}
        onOpenChange={(open) => {
          if (!open) {
            removalSelection.clear();
            setRemoveFiles(false);
          }
        }}
        contentClass="w-96 max-w-full p-6 flex flex-col gap-4 rounded-xl border border-subtle bg-surface shadow-xl"
      >
        {({ close }) => (
          <>
            <Text class="text-xl font-semibold">
              {pendingRemoval().length === 1
                ? "Remove download?"
                : "Remove downloads?"}
            </Text>
            <Text class="whitespace-normal text-sm text-muted">
              {pendingRemoval().length === 1
                ? pendingRemoval()[0]?.name
                : `${pendingRemoval().length} selected downloads`}{" "}
              will be removed from downloads.
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
                disabled={taskActions.pending("batch")}
                onClick={() => {
                  const ids = pendingRemoval().map((task) => task.id);
                  const shouldRemoveFiles = removeFiles();
                  close();
                  executeAction("batch", async () => {
                    await downloads.batchTaskAction(ids, "remove", {
                      removeFiles: shouldRemoveFiles,
                    });
                    batchSelection.clear();
                    inspectorSelection.clear();
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

function formatEta(task: DownloadTask): string {
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

function Detail(props: { label: string; value: string }) {
  return (
    <View role="group" aria-label={props.label} class="flex flex-col gap-1">
      <Text class="text-xs text-muted">{props.label}</Text>
      <Text class="text-sm whitespace-normal">{props.value}</Text>
    </View>
  );
}

function TaskStatusDetail(props: { status: string }) {
  const presentation = () => taskStatusPresentation(props.status);
  return (
    <View
      role="group"
      aria-label={`Task detail status: ${presentation().label}`}
      class="flex flex-col items-start gap-1"
    >
      <Text class="text-xs text-muted">Status</Text>
      <Badge variant={presentation().variant}>{presentation().label}</Badge>
    </View>
  );
}
