import {
  bindJsonCapability,
  createLatestAsyncResource,
  createRevisionedHostResource,
  type Host,
  hostMessages,
  type JsonCapabilityMethodName,
  type NativeCapability,
  notification,
  reconcileKeyedList,
  useHost,
} from "@wabou/ui";
import {
  createComponent,
  createContext,
  createMemo,
  createSignal,
  onCleanup,
  type ParentProps,
  useContext,
} from "solid-js";
import type {
  NativeDownloadsApi,
  TorrentPreview,
} from "./generated/native-downloads";

export type {
  TorrentFilePreview,
  TorrentPreview,
} from "./generated/native-downloads";

export interface DownloadTask {
  id: string;
  name: string;
  status: string;
  totalLength: number;
  completedLength: number;
  downloadSpeed: number;
  uploadSpeed: number;
  uploadedLength: number;
  dir: string;
  filePath?: string;
  uri?: string;
  connections: number;
  seeders?: number;
  errorMessage?: string;
  bittorrent: boolean;
  /** The host can reconstruct this task from its URI or BitTorrent info hash. */
  retryable: boolean;
  /** Completed task retained by Motrix after downloads stopped seeding it. */
  archived: boolean;
  fileCount: number;
  priority: TaskPriority;
  /** Engine-persisted creation time, as Unix milliseconds. */
  createdAtMs: number;
}

export type TaskPriority = "low" | "normal" | "high" | "critical";

export interface DownloadSnapshot {
  revision: number;
  status: "starting" | "ready" | "failed";
  version?: string;
  error?: string;
  downloadSpeed: number;
  uploadSpeed: number;
  tasks: DownloadTask[];
  activity: number[];
  downloadedToday: number;
  downloadedTotal: number;
  uploadedTotal: number;
  nat: NatStatus;
}

export interface NatStatus {
  enabled: boolean;
  state: "disabled" | "starting" | "mapping" | "mapped" | "error";
  tcpExternalAddress?: string;
  udpExternalAddress?: string;
}

export interface DownloadSnapshotPatch extends Omit<DownloadSnapshot, "tasks"> {
  baseRevision: number;
  upsertedTasks: DownloadTask[];
  removedIds: string[];
  taskOrder: string[];
}

export function applySnapshotPatch(
  current: DownloadSnapshot,
  patch: DownloadSnapshotPatch,
): DownloadSnapshot | undefined {
  if (current.revision !== patch.baseRevision) return undefined;
  const tasks = reconcileKeyedList(
    current.tasks,
    {
      upserted: patch.upsertedTasks,
      removed: patch.removedIds,
      order: patch.taskOrder,
    },
    (task) => task.id,
  );
  if (!tasks) return undefined;
  const { baseRevision, upsertedTasks, removedIds, taskOrder, ...snapshot } =
    patch;
  void baseRevision;
  void upsertedTasks;
  void removedIds;
  void taskOrder;
  return {
    ...snapshot,
    tasks,
  };
}

export function terminalTaskTransitions(
  previousStatuses: Map<string, string>,
  tasks: readonly DownloadTask[],
  suppressUnknown: boolean,
): Array<DownloadTask & { status: "complete" | "error" }> {
  const currentIds = new Set(tasks.map((task) => task.id));
  const terminal: Array<DownloadTask & { status: "complete" | "error" }> = [];
  for (const task of tasks) {
    const previous = previousStatuses.get(task.id);
    previousStatuses.set(task.id, task.status);
    if (
      previous !== task.status &&
      !(previous === undefined && suppressUnknown) &&
      (task.status === "complete" || task.status === "error")
    )
      terminal.push(task as DownloadTask & { status: "complete" | "error" });
  }
  for (const id of previousStatuses.keys()) {
    if (!currentIds.has(id)) previousStatuses.delete(id);
  }
  return terminal;
}

export interface TaskSpeedHistory {
  download: readonly number[];
  upload: readonly number[];
}

export type TaskSpeedHistories = Readonly<Record<string, TaskSpeedHistory>>;

export function appendTaskSpeedHistories(
  current: TaskSpeedHistories,
  tasks: readonly DownloadTask[],
  limit = 120,
): TaskSpeedHistories {
  const boundedLimit = Math.max(1, Math.floor(limit));
  return Object.fromEntries(
    tasks.map((task) => {
      const previous = current[task.id] ?? { download: [], upload: [] };
      return [
        task.id,
        {
          download: [...previous.download, task.downloadSpeed].slice(
            -boundedLimit,
          ),
          upload: [...previous.upload, task.uploadSpeed].slice(-boundedLimit),
        },
      ];
    }),
  );
}

export interface DownloadTaskDetails {
  files: DownloadTaskFile[];
}

export interface DownloadTaskFile {
  index: number;
  path: string;
  length: number;
  completedLength: number;
  selected: boolean;
}

export interface MotrixConfig {
  theme: "light" | "dark" | "system";
  downloadDir: string;
  split: number;
  maxConnectionPerServer: number;
  minSplitSize: string;
  maxConcurrentDownloads: number;
  notifyOnComplete: boolean;
  notifyOnError: boolean;
  resumeAllWhenAppLaunched: boolean;
  newTaskShowDownloading: boolean;
  warnBeforeQuit: boolean;
  dhtEnabled: boolean;
  pexEnabled: boolean;
  btMaxPeers: number;
  listenPort: number;
  natEnabled: boolean;
  natProtocol: "auto" | "pcp" | "natPmp" | "upnp";
  seedRatio: number;
  maxOverallDownloadLimit: string;
  maxOverallUploadLimit: string;
  speedProfiles: MotrixSpeedProfile[];
  userAgent: string;
  proxy: MotrixProxyConfig;
}

export interface MotrixSpeedProfile {
  name: string;
  downloadLimit: string;
  uploadLimit: string;
}

export interface MotrixProxyConfig {
  enabled: boolean;
  host: string;
  port: number;
}

export interface TaskEvent {
  id: number;
  taskId: string;
  name: string;
  status: "complete" | "error";
  time: string;
}

interface NativeDownloadsCapability extends NativeCapability {
  __wabouCapabilityVersion: number;
  getSnapshot(): string | PromiseLike<string>;
  addUri(request: string): string | PromiseLike<string>;
  addTorrent(request: string): string | PromiseLike<string>;
  taskAction(request: string): string | PromiseLike<string>;
  setTaskPriority(request: string): string | PromiseLike<string>;
  batchTaskAction(request: string): string | PromiseLike<string>;
  getConfig(): string | PromiseLike<string>;
  setConfig(request: string): string | PromiseLike<string>;
  openTaskFolder(request: string): string | PromiseLike<string>;
  openPath(request: string): string | PromiseLike<string>;
  openConfigFolder(): string | PromiseLike<string>;
  globalTaskAction(request: string): string | PromiseLike<string>;
  getTaskDetails(request: string): string | PromiseLike<string>;
  retryEngine(): string | PromiseLike<string>;
}

interface DownloadsHost extends Host {
  downloads: NativeDownloadsCapability;
  downloadsNative: NativeDownloadsApi & {
    __wabouCapabilityVersion: number;
  };
}

export type DownloadAction =
  | "pause"
  | "resume"
  | "remove"
  | "retry"
  | "stopSeeding";
export interface TaskActionOptions {
  removeFiles?: boolean;
}
interface DownloadsContextValue {
  snapshot(): DownloadSnapshot;
  downloadHistory(): readonly number[];
  uploadHistory(): readonly number[];
  events(): readonly TaskEvent[];
  quitRequests(): number;
  requestedTaskId(): string | undefined;
  requestTaskInspection(id: string): boolean;
  clearTaskInspectionRequest(): void;
  taskHistory(id: string): TaskSpeedHistory;
  clearEvents(): void;
  config(): MotrixConfig;
  configStatus(): "idle" | "pending" | "ready" | "error";
  configError(): unknown | undefined;
  refreshConfig(): Promise<MotrixConfig | undefined>;
  retryEngine(): Promise<void>;
  addUris(request: AddUrisRequest): Promise<string[]>;
  addTorrent(request: {
    path: string;
    dir?: string;
    split?: number;
    selectedFiles?: number[];
    priority?: TaskPriority;
  }): Promise<string>;
  inspectTorrent(path: string): Promise<TorrentPreview>;
  taskDetails(id: string): Promise<DownloadTaskDetails>;
  taskAction(
    id: string,
    action: DownloadAction,
    options?: TaskActionOptions,
  ): Promise<void>;
  setTaskPriority(id: string, priority: TaskPriority): Promise<void>;
  batchTaskAction(
    ids: string[],
    action: DownloadAction,
    options?: TaskActionOptions,
  ): Promise<string[]>;
  saveConfig(config: MotrixConfig): Promise<SetConfigResult>;
  openTaskFolder(path: string): Promise<void>;
  openPath(path: string): Promise<void>;
  openConfigFolder(): Promise<void>;
  globalTaskAction(action: "pauseAll" | "resumeAll"): Promise<void>;
  refresh(): Promise<void>;
}

export interface SetConfigResult {
  config: MotrixConfig;
  restartRequired: boolean;
}

export function snapshotReflectsTaskAction(
  snapshot: DownloadSnapshot,
  id: string,
  action: DownloadAction,
): boolean {
  const task = snapshot.tasks.find((candidate) => candidate.id === id);
  if (action === "remove" || action === "stopSeeding") return !task;
  if (!task) return true;
  if (action === "pause") return !["active", "waiting"].includes(task.status);
  if (action === "resume") return task.status !== "paused";
  return task.status !== "error" && task.status !== "removed";
}

export interface AddUrisRequest {
  uris: string[];
  dir?: string;
  out?: string;
  split?: number;
  headers?: string[];
  priority?: TaskPriority;
}

const disconnected: DownloadSnapshot = {
  revision: 0,
  status: "starting",
  error: "Starting download engine…",
  downloadSpeed: 0,
  uploadSpeed: 0,
  tasks: [],
  activity: Array(364).fill(0),
  downloadedToday: 0,
  downloadedTotal: 0,
  uploadedTotal: 0,
  nat: { enabled: false, state: "disabled" },
};
const defaultConfig: MotrixConfig = {
  theme: "light",
  downloadDir: "",
  split: 16,
  maxConnectionPerServer: 16,
  minSplitSize: "20M",
  maxConcurrentDownloads: 5,
  notifyOnComplete: true,
  notifyOnError: true,
  resumeAllWhenAppLaunched: false,
  newTaskShowDownloading: true,
  warnBeforeQuit: true,
  dhtEnabled: true,
  pexEnabled: true,
  btMaxPeers: 128,
  listenPort: 6881,
  natEnabled: true,
  natProtocol: "auto",
  seedRatio: 1,
  maxOverallDownloadLimit: "0",
  maxOverallUploadLimit: "0",
  speedProfiles: [
    { name: "Unlimited", downloadLimit: "0", uploadLimit: "0" },
    { name: "Balanced", downloadLimit: "10M", uploadLimit: "1M" },
    { name: "Saver", downloadLimit: "1M", uploadLimit: "256K" },
  ],
  userAgent: "Motrix-Wabou/0.1",
  proxy: {
    enabled: false,
    host: "",
    port: 8080,
  },
};

const DownloadsContext = createContext<DownloadsContextValue>();

export function DownloadsProvider(props: ParentProps) {
  const host = useHost<DownloadsHost>();
  const [downloadHistory, setDownloadHistory] = createSignal<readonly number[]>(
    Array(30).fill(0),
  );
  const [uploadHistory, setUploadHistory] = createSignal<readonly number[]>(
    Array(30).fill(0),
  );
  const [events, setEvents] = createSignal<readonly TaskEvent[]>([]);
  const [taskHistories, setTaskHistories] = createSignal<TaskSpeedHistories>(
    {},
  );
  const [quitRequests, setQuitRequests] = createSignal(0);
  const [requestedTaskId, setRequestedTaskId] = createSignal<string>();
  const previousStatuses = new Map<string, string>();
  let statusesSeeded = false;
  let nextEventId = 1;
  const invoke = bindJsonCapability(host.downloads, {
    name: "downloads",
    version: 1,
  });
  const call = <T,>(
    method: JsonCapabilityMethodName<NativeDownloadsCapability>,
    request?: unknown,
  ) => invoke<T>(String(method), request);
  const configResource = createLatestAsyncResource<boolean, MotrixConfig>({
    source: () => true,
    initialValue: defaultConfig,
    load: () => call("getConfig"),
  });
  const config = () => configResource.value() ?? defaultConfig;
  const acceptSnapshot = (next: DownloadSnapshot) => {
    setDownloadHistory((values) => [...values.slice(-29), next.downloadSpeed]);
    setUploadHistory((values) => [...values.slice(-29), next.uploadSpeed]);
    setTaskHistories((current) =>
      appendTaskSpeedHistories(current, next.tasks),
    );
    const terminalTasks = terminalTaskTransitions(
      previousStatuses,
      next.tasks,
      !statusesSeeded,
    );
    statusesSeeded = true;
    const nextEvents = terminalTasks.map<TaskEvent>((task) => ({
      id: nextEventId++,
      taskId: task.id,
      name: task.name,
      status: task.status,
      time: new Date().toLocaleTimeString(),
    }));
    if (nextEvents.length > 0)
      setEvents((items) =>
        [...nextEvents.toReversed(), ...items].slice(0, 100),
      );
    for (const task of terminalTasks) {
      const currentConfig = config();
      if (
        (task.status === "complete" && currentConfig.notifyOnComplete) ||
        (task.status === "error" && currentConfig.notifyOnError)
      )
        void notification.show({
          title:
            task.status === "complete"
              ? "Download complete"
              : "Download failed",
          body: task.name,
        });
    }
  };
  const snapshotResource = createRevisionedHostResource<
    DownloadSnapshot,
    DownloadSnapshotPatch
  >({
    initial: disconnected,
    load: () => call("getSnapshot"),
    snapshotTopic: "downloads.snapshot",
    patchTopic: "downloads.snapshot.patch",
    applyPatch: applySnapshotPatch,
    onValue: acceptSnapshot,
    onError(error, source) {
      console.error(`cannot synchronize downloads ${source}`, error);
    },
  });
  const snapshot = createMemo<DownloadSnapshot>(() => {
    const current = snapshotResource.value();
    const error = snapshotResource.error();
    return error ? { ...current, error: String(error) } : current;
  });
  const refresh = async () => {
    await snapshotResource.refresh();
  };
  const waitForSnapshot = async (
    predicate: (value: DownloadSnapshot) => boolean,
  ) =>
    snapshotResource.waitFor(predicate, {
      timeout: 500,
      refreshOnTimeout: true,
    });
  const value: DownloadsContextValue = {
    snapshot,
    downloadHistory,
    uploadHistory,
    events,
    quitRequests,
    requestedTaskId,
    requestTaskInspection: (id) => {
      if (!snapshot().tasks.some((task) => task.id === id)) return false;
      setRequestedTaskId(id);
      return true;
    },
    clearTaskInspectionRequest: () => setRequestedTaskId(undefined),
    taskHistory: (id) => taskHistories()[id] ?? { download: [], upload: [] },
    clearEvents: () => setEvents([]),
    config,
    configStatus: configResource.status,
    configError: configResource.error,
    refreshConfig: configResource.refresh,
    retryEngine: async () => {
      try {
        await call("retryEngine");
      } finally {
        await configResource.refresh();
        await snapshotResource.refresh();
      }
    },
    refresh,
    addUris: async (request) => {
      const ids = await call<string[]>("addUri", request);
      await waitForSnapshot((next) =>
        ids.every((id) => next.tasks.some((task) => task.id === id)),
      );
      return ids;
    },
    addTorrent: async (request) => {
      const id = await call<string>("addTorrent", request);
      await waitForSnapshot((next) =>
        next.tasks.some((task) => task.id === id),
      );
      return id;
    },
    inspectTorrent: (path) => host.downloadsNative.inspectTorrent({ path }),
    taskDetails: (id) => call("getTaskDetails", { id }),
    taskAction: async (id, action, options) => {
      await call("taskAction", {
        id,
        action,
        removeFiles: options?.removeFiles ?? false,
      });
      await waitForSnapshot((next) =>
        snapshotReflectsTaskAction(next, id, action),
      );
    },
    setTaskPriority: async (id, priority) => {
      await call("setTaskPriority", { id, priority });
      await waitForSnapshot(
        (next) =>
          next.tasks.find((candidate) => candidate.id === id)?.priority ===
          priority,
      );
    },
    batchTaskAction: async (ids, action, options) => {
      const completed = await call<string[]>("batchTaskAction", {
        ids,
        action,
        removeFiles: options?.removeFiles ?? false,
      });
      await waitForSnapshot((next) =>
        completed.every((id) => snapshotReflectsTaskAction(next, id, action)),
      );
      return completed;
    },
    saveConfig: async (next) => {
      const result = await call<SetConfigResult>("setConfig", next);
      configResource.mutate(result.config);
      return result;
    },
    openTaskFolder: (path) => call("openTaskFolder", { path }),
    openPath: (path) => call("openPath", { path }),
    openConfigFolder: () => call("openConfigFolder"),
    globalTaskAction: async (action) => {
      const affected = snapshotResource
        .value()
        .tasks.filter((task) =>
          action === "pauseAll"
            ? ["active", "waiting"].includes(task.status)
            : task.status === "paused",
        )
        .map((task) => task.id);
      await call("globalTaskAction", { action });
      await waitForSnapshot((next) =>
        affected.every((id) => {
          const task = next.tasks.find((candidate) => candidate.id === id);
          return (
            !task ||
            (action === "pauseAll"
              ? !["active", "waiting"].includes(task.status)
              : task.status !== "paused")
          );
        }),
      );
    },
  };
  const unsubscribeQuit = hostMessages.subscribe("motrix.quitRequested", () =>
    setQuitRequests((value) => value + 1),
  );
  onCleanup(() => {
    unsubscribeQuit();
  });
  return createComponent(DownloadsContext, {
    value,
    get children() {
      return props.children;
    },
  });
}

export function useDownloads(): DownloadsContextValue {
  const value = useContext(DownloadsContext);
  if (!value)
    throw new Error("useDownloads must be used inside DownloadsProvider");
  return value;
}
