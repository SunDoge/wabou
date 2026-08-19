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
  fileCount: number;
}

export interface Aria2Snapshot {
  connected: boolean;
  endpoint: string;
  version?: string;
  error?: string;
  downloadSpeed: number;
  uploadSpeed: number;
  tasks: Aria2Task[];
  managed: boolean;
  engineRunning: boolean;
}

export interface Aria2TaskDetails {
  files: Aria2TaskFile[];
  trackers: string[];
  peers: Aria2TaskPeer[];
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
  theme: "light" | "dark";
  engineMode: "managed" | "external";
  externalEndpoint: string;
  externalSecret: string;
  downloadDir: string;
  split: number;
  maxConcurrentDownloads: number;
  notifyOnComplete: boolean;
  notifyOnError: boolean;
  resumeAllWhenAppLaunched: boolean;
  newTaskShowDownloading: boolean;
  btTrackers: string[];
  maxOverallDownloadLimit: string;
  maxOverallUploadLimit: string;
  userAgent: string;
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
}

interface Aria2Host extends Host {
  aria2: NativeAria2Capability;
}

type Aria2Action = "pause" | "resume" | "remove" | "retry";
export interface TaskActionOptions {
  removeFiles?: boolean;
}
interface Aria2ContextValue {
  snapshot(): Aria2Snapshot;
  downloadHistory(): readonly number[];
  uploadHistory(): readonly number[];
  sessionDownloaded(): number;
  events(): readonly TaskEvent[];
  clearEvents(): void;
  config(): MotrixConfig;
  addUris(request: AddUrisRequest): Promise<string[]>;
  addTorrent(request: {
    path: string;
    dir?: string;
    split?: number;
  }): Promise<string>;
  taskDetails(gid: string): Promise<Aria2TaskDetails>;
  setSelectedFiles(gid: string, indices: number[]): Promise<Aria2TaskDetails>;
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

export interface AddUrisRequest {
  uris: string[];
  dir?: string;
  out?: string;
  split?: number;
}

const disconnected: Aria2Snapshot = {
  connected: false,
  endpoint: "ws://127.0.0.1:6800/jsonrpc",
  error: "Connecting to aria2…",
  downloadSpeed: 0,
  uploadSpeed: 0,
  tasks: [],
  managed: false,
  engineRunning: false,
};
const defaultConfig: MotrixConfig = {
  theme: "light",
  engineMode: "managed",
  externalEndpoint: "ws://127.0.0.1:6800/jsonrpc",
  externalSecret: "",
  downloadDir: "",
  split: 16,
  maxConcurrentDownloads: 5,
  notifyOnComplete: true,
  notifyOnError: true,
  resumeAllWhenAppLaunched: false,
  newTaskShowDownloading: true,
  btTrackers: [
    "udp://tracker.opentrackr.org:1337/announce",
    "udp://open.stealth.si:80/announce",
    "udp://tracker.torrent.eu.org:451/announce",
  ],
  maxOverallDownloadLimit: "0",
  maxOverallUploadLimit: "0",
  userAgent: "Motrix-Wabou/0.1",
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
  const [sessionDownloaded, setSessionDownloaded] = createSignal(0);
  const [events, setEvents] = createSignal<readonly TaskEvent[]>([]);
  const previousStatuses = new Map<string, string>();
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
    sessionDownloaded,
    events,
    clearEvents: () => setEvents([]),
    config,
    refresh,
    addUris: (request) => call("addUri", request),
    addTorrent: (request) => call("addTorrent", request),
    taskDetails: (gid) => call("getTaskDetails", { gid }),
    setSelectedFiles: (gid, indices) =>
      call("setSelectedFiles", { gid, indices }),
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
  const unsubscribe = hostMessages.subscribe("aria2.snapshot", (payload) => {
    if (typeof payload !== "string") return;
    try {
      const next = JSON.parse(payload) as Aria2Snapshot;
      setSnapshot(next);
      setDownloadHistory((values) => [
        ...values.slice(-29),
        next.downloadSpeed,
      ]);
      setUploadHistory((values) => [...values.slice(-29), next.uploadSpeed]);
      setSessionDownloaded((value) => value + next.downloadSpeed);
      for (const task of next.tasks) {
        const previous = previousStatuses.get(task.gid);
        previousStatuses.set(task.gid, task.status);
        if (
          previous === undefined ||
          previous === task.status ||
          (task.status !== "complete" && task.status !== "error")
        )
          continue;
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
    } catch (error) {
      console.error("invalid aria2 snapshot", error);
    }
  });
  onCleanup(unsubscribe);
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
