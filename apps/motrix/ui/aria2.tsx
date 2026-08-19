import { type Host, hostMessages, notification, useHost } from "@wabou/ui";
import {
  createComponent,
  createContext,
  createSignal,
  onCleanup,
  type ParentProps,
  useContext,
} from "solid-js";

export interface Aria2Task {
  gid: string;
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
  /** Completed task retained by Motrix after aria2 stopped seeding it. */
  archived: boolean;
  fileCount: number;
}

export interface Aria2Snapshot {
  revision: number;
  connected: boolean;
  endpoint: string;
  version?: string;
  error?: string;
  downloadSpeed: number;
  uploadSpeed: number;
  tasks: Aria2Task[];
  managed: boolean;
  engineRunning: boolean;
  activity: number[];
  downloadedToday: number;
  nat: NatStatus;
}

export interface NatStatus {
  enabled: boolean;
  state: "disabled" | "starting" | "mapping" | "mapped" | "error";
  tcpExternalAddress?: string;
  udpExternalAddress?: string;
  dhtExternalAddress?: string;
}

export interface Aria2SnapshotPatch extends Omit<Aria2Snapshot, "tasks"> {
  baseRevision: number;
  upsertedTasks: Aria2Task[];
  removedGids: string[];
  taskOrder: string[];
}

export function applySnapshotPatch(
  current: Aria2Snapshot,
  patch: Aria2SnapshotPatch,
): Aria2Snapshot | undefined {
  if (current.revision !== patch.baseRevision) return undefined;
  const removed = new Set(patch.removedGids);
  const tasks = new Map(
    current.tasks
      .filter((task) => !removed.has(task.gid))
      .map((task) => [task.gid, task]),
  );
  for (const task of patch.upsertedTasks) tasks.set(task.gid, task);
  const ordered = patch.taskOrder.flatMap((gid) => {
    const task = tasks.get(gid);
    return task ? [task] : [];
  });
  if (ordered.length !== patch.taskOrder.length) return undefined;
  return {
    revision: patch.revision,
    connected: patch.connected,
    endpoint: patch.endpoint,
    version: patch.version,
    error: patch.error,
    downloadSpeed: patch.downloadSpeed,
    uploadSpeed: patch.uploadSpeed,
    managed: patch.managed,
    engineRunning: patch.engineRunning,
    activity: patch.activity,
    downloadedToday: patch.downloadedToday,
    nat: patch.nat,
    tasks: ordered,
  };
}

export function terminalTaskTransitions(
  previousStatuses: Map<string, string>,
  tasks: readonly Aria2Task[],
  suppressUnknown: boolean,
): Array<Aria2Task & { status: "complete" | "error" }> {
  const currentGids = new Set(tasks.map((task) => task.gid));
  const terminal: Array<Aria2Task & { status: "complete" | "error" }> = [];
  for (const task of tasks) {
    const previous = previousStatuses.get(task.gid);
    previousStatuses.set(task.gid, task.status);
    if (
      previous !== task.status &&
      !(previous === undefined && suppressUnknown) &&
      (task.status === "complete" || task.status === "error")
    )
      terminal.push(task as Aria2Task & { status: "complete" | "error" });
  }
  for (const gid of previousStatuses.keys()) {
    if (!currentGids.has(gid)) previousStatuses.delete(gid);
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
  tasks: readonly Aria2Task[],
  limit = 120,
): TaskSpeedHistories {
  const boundedLimit = Math.max(1, Math.floor(limit));
  return Object.fromEntries(
    tasks.map((task) => {
      const previous = current[task.gid] ?? { download: [], upload: [] };
      return [
        task.gid,
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

export interface Aria2TaskDetails {
  files: Aria2TaskFile[];
  trackers: string[];
  peers: Aria2TaskPeer[];
  bitfield: string;
  pieceLength: number;
  numPieces: number;
  maxDownloadLimit: string;
  maxUploadLimit: string;
}

export interface Aria2TaskFile {
  index: number;
  path: string;
  length: number;
  completedLength: number;
  selected: boolean;
}

export interface Aria2TaskPeer {
  ip: string;
  port: number;
  downloadSpeed: number;
  uploadSpeed: number;
  seeder: boolean;
}

export interface MotrixConfig {
  theme: "light" | "dark" | "system";
  engineMode: "managed" | "external";
  externalEndpoint: string;
  externalSecret: string;
  downloadDir: string;
  split: number;
  maxConnectionPerServer: number;
  minSplitSize: string;
  fileAllocation: "none" | "prealloc" | "trunc" | "falloc";
  maxConcurrentDownloads: number;
  notifyOnComplete: boolean;
  notifyOnError: boolean;
  resumeAllWhenAppLaunched: boolean;
  newTaskShowDownloading: boolean;
  warnBeforeQuit: boolean;
  btTrackers: string[];
  dhtEnabled: boolean;
  pexEnabled: boolean;
  btMaxPeers: number;
  listenPort: number;
  dhtListenPort: number;
  natEnabled: boolean;
  natProtocol: "auto" | "pcp" | "natPmp" | "upnp";
  seedRatio: number;
  seedTime: number;
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
  bypass: string[];
}

export interface TaskEvent {
  id: number;
  gid: string;
  name: string;
  status: "complete" | "error";
  time: string;
}

interface NativeAria2Capability {
  __wabouCapabilityVersion: number;
  getSnapshot(): string | PromiseLike<string>;
  addUri(request: string): string | PromiseLike<string>;
  addTorrent(request: string): string | PromiseLike<string>;
  inspectTorrent(request: string): string | PromiseLike<string>;
  taskAction(request: string): string | PromiseLike<string>;
  batchTaskAction(request: string): string | PromiseLike<string>;
  engineAction(request: string): string | PromiseLike<string>;
  getConfig(): string | PromiseLike<string>;
  setConfig(request: string): string | PromiseLike<string>;
  openTaskFolder(request: string): string | PromiseLike<string>;
  openConfigFolder(): string | PromiseLike<string>;
  globalTaskAction(request: string): string | PromiseLike<string>;
  getTaskDetails(request: string): string | PromiseLike<string>;
  setSelectedFiles(request: string): string | PromiseLike<string>;
  setTaskLimits(request: string): string | PromiseLike<string>;
  setTaskTrackers(request: string): string | PromiseLike<string>;
  changeTaskPosition(request: string): string | PromiseLike<string>;
}

interface Aria2Host extends Host {
  aria2: NativeAria2Capability;
}

type Aria2Action = "pause" | "resume" | "remove" | "retry" | "stopSeeding";
export interface TaskActionOptions {
  removeFiles?: boolean;
}
interface Aria2ContextValue {
  snapshot(): Aria2Snapshot;
  downloadHistory(): readonly number[];
  uploadHistory(): readonly number[];
  events(): readonly TaskEvent[];
  quitRequests(): number;
  taskHistory(gid: string): TaskSpeedHistory;
  clearEvents(): void;
  config(): MotrixConfig;
  addUris(request: AddUrisRequest): Promise<string[]>;
  addTorrent(request: {
    path: string;
    dir?: string;
    split?: number;
    selectedFiles?: number[];
  }): Promise<string>;
  inspectTorrent(path: string): Promise<TorrentPreview>;
  taskDetails(gid: string): Promise<Aria2TaskDetails>;
  setSelectedFiles(gid: string, indices: number[]): Promise<Aria2TaskDetails>;
  setTaskLimits(
    gid: string,
    maxDownloadLimit: string,
    maxUploadLimit: string,
  ): Promise<Aria2TaskDetails>;
  setTaskTrackers(gid: string, trackers: string[]): Promise<Aria2TaskDetails>;
  changeTaskPosition(
    gid: string,
    position: "top" | "up" | "down" | "bottom",
  ): Promise<number>;
  taskAction(
    gid: string,
    action: Aria2Action,
    options?: TaskActionOptions,
  ): Promise<void>;
  batchTaskAction(
    gids: string[],
    action: Aria2Action,
    options?: TaskActionOptions,
  ): Promise<string[]>;
  engineAction(action: "start" | "stop" | "restart"): Promise<void>;
  saveConfig(config: MotrixConfig): Promise<MotrixConfig>;
  openTaskFolder(path: string): Promise<void>;
  openConfigFolder(): Promise<void>;
  globalTaskAction(
    action: "pauseAll" | "resumeAll" | "clearCompleted",
  ): Promise<void>;
  refresh(): Promise<void>;
}

export interface TorrentFilePreview {
  index: number;
  path: string;
  length: number;
}

export interface TorrentPreview {
  name: string;
  totalLength: number;
  files: TorrentFilePreview[];
}

export interface AddUrisRequest {
  uris: string[];
  dir?: string;
  out?: string;
  split?: number;
  headers?: string[];
  checksum?: string;
  proxy?: string;
}

const disconnected: Aria2Snapshot = {
  revision: 0,
  connected: false,
  endpoint: "ws://127.0.0.1:6800/jsonrpc",
  error: "Connecting to aria2…",
  downloadSpeed: 0,
  uploadSpeed: 0,
  tasks: [],
  managed: false,
  engineRunning: false,
  activity: Array(364).fill(0),
  downloadedToday: 0,
  nat: { enabled: false, state: "disabled" },
};
const defaultConfig: MotrixConfig = {
  theme: "light",
  engineMode: "managed",
  externalEndpoint: "ws://127.0.0.1:6800/jsonrpc",
  externalSecret: "",
  downloadDir: "",
  split: 16,
  maxConnectionPerServer: 16,
  minSplitSize: "20M",
  fileAllocation: "none",
  maxConcurrentDownloads: 5,
  notifyOnComplete: true,
  notifyOnError: true,
  resumeAllWhenAppLaunched: false,
  newTaskShowDownloading: true,
  warnBeforeQuit: true,
  btTrackers: [
    "udp://tracker.opentrackr.org:1337/announce",
    "udp://open.stealth.si:80/announce",
    "udp://tracker.torrent.eu.org:451/announce",
  ],
  dhtEnabled: true,
  pexEnabled: true,
  btMaxPeers: 128,
  listenPort: 6881,
  dhtListenPort: 6881,
  natEnabled: true,
  natProtocol: "auto",
  seedRatio: 1,
  seedTime: 60,
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
    bypass: ["localhost", "127.0.0.1"],
  },
};

const Aria2Context = createContext<Aria2ContextValue>();

function decode<T>(raw: string, operation: string): T {
  const envelope = JSON.parse(raw) as
    | { ok: true; value: T }
    | { ok: false; error?: { message?: string } };
  if (!envelope.ok)
    throw new Error(envelope.error?.message ?? `${operation} failed`);
  return envelope.value;
}

export function Aria2Provider(props: ParentProps) {
  const host = useHost<Aria2Host>();
  const [snapshot, setSnapshot] = createSignal(disconnected);
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
  const previousStatuses = new Map<string, string>();
  let statusesSeeded = false;
  const [config, setConfig] = createSignal(defaultConfig);
  const call = async <T,>(
    method: keyof NativeAria2Capability,
    request?: unknown,
  ) => {
    const capability = host.aria2;
    if (capability?.__wabouCapabilityVersion !== 1)
      throw new Error("The native aria2 capability is unavailable");
    const functionValue = capability[method];
    if (typeof functionValue !== "function")
      throw new Error(`The native aria2.${method} method is unavailable`);
    const raw = await (request === undefined
      ? (functionValue as () => string | PromiseLike<string>).call(capability)
      : (functionValue as (value: string) => string | PromiseLike<string>).call(
          capability,
          JSON.stringify(request),
        ));
    return decode<T>(raw, `aria2.${method}`);
  };
  const refresh = async () => {
    setSnapshot(await call("getSnapshot"));
  };
  const value: Aria2ContextValue = {
    snapshot,
    downloadHistory,
    uploadHistory,
    events,
    quitRequests,
    taskHistory: (gid) => taskHistories()[gid] ?? { download: [], upload: [] },
    clearEvents: () => setEvents([]),
    config,
    refresh,
    addUris: (request) => call("addUri", request),
    addTorrent: (request) => call("addTorrent", request),
    inspectTorrent: (path) => call("inspectTorrent", { path }),
    taskDetails: (gid) => call("getTaskDetails", { gid }),
    setSelectedFiles: (gid, indices) =>
      call("setSelectedFiles", { gid, indices }),
    setTaskLimits: (gid, maxDownloadLimit, maxUploadLimit) =>
      call("setTaskLimits", { gid, maxDownloadLimit, maxUploadLimit }),
    setTaskTrackers: (gid, trackers) =>
      call("setTaskTrackers", { gid, trackers }),
    changeTaskPosition: (gid, position) =>
      call("changeTaskPosition", { gid, position }),
    taskAction: (gid, action, options) =>
      call("taskAction", {
        gid,
        action,
        removeFiles: options?.removeFiles ?? false,
      }),
    batchTaskAction: (gids, action, options) =>
      call("batchTaskAction", {
        gids,
        action,
        removeFiles: options?.removeFiles ?? false,
      }),
    engineAction: (action) => call("engineAction", { action }),
    saveConfig: async (next) => {
      const saved = await call<MotrixConfig>("setConfig", next);
      setConfig(saved);
      return saved;
    },
    openTaskFolder: (path) => call("openTaskFolder", { path }),
    openConfigFolder: () => call("openConfigFolder"),
    globalTaskAction: (action) => call("globalTaskAction", { action }),
  };
  void refresh().catch((error) =>
    setSnapshot((current) => ({ ...current, error: String(error) })),
  );
  void call<MotrixConfig>("getConfig")
    .then(setConfig)
    .catch((error) => console.error("cannot load Motrix config", error));
  const acceptSnapshot = (next: Aria2Snapshot) => {
    setSnapshot(next);
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
    for (const task of terminalTasks) {
      const event: TaskEvent = {
        id: Date.now() + events().length,
        gid: task.gid,
        name: task.name,
        status: task.status,
        time: new Date().toLocaleTimeString(),
      };
      setEvents((items) => [event, ...items].slice(0, 100));
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
  const unsubscribeSnapshot = hostMessages.subscribe(
    "aria2.snapshot",
    (payload) => {
      if (typeof payload !== "string") return;
      try {
        acceptSnapshot(JSON.parse(payload) as Aria2Snapshot);
      } catch (error) {
        console.error("invalid aria2 snapshot", error);
      }
    },
  );
  const unsubscribePatch = hostMessages.subscribe(
    "aria2.snapshot.patch",
    (payload) => {
      if (typeof payload !== "string") return;
      try {
        const patch = JSON.parse(payload) as Aria2SnapshotPatch;
        const current = snapshot();
        const next = applySnapshotPatch(current, patch);
        if (!next) {
          void refresh().catch((error) =>
            setSnapshot((value) => ({ ...value, error: String(error) })),
          );
          return;
        }
        acceptSnapshot(next);
      } catch (error) {
        console.error("invalid aria2 snapshot patch", error);
      }
    },
  );
  const unsubscribeQuit = hostMessages.subscribe("motrix.quitRequested", () =>
    setQuitRequests((value) => value + 1),
  );
  onCleanup(() => {
    unsubscribeSnapshot();
    unsubscribePatch();
    unsubscribeQuit();
  });
  return createComponent(Aria2Context, {
    value,
    get children() {
      return props.children;
    },
  });
}

export function useAria2(): Aria2ContextValue {
  const value = useContext(Aria2Context);
  if (!value) throw new Error("useAria2 must be used inside Aria2Provider");
  return value;
}
