import {
  bindCapability,
  bindJsonCapability,
  type Host,
  hostMessages,
  type NativeCapability,
  useHost,
} from "@wabou/ui";

export interface PiStatus {
  running: boolean;
  cwd?: string;
  runtime: "bun";
  error?: string;
}

export interface PiSession {
  agentId: string;
  sessionId: string;
  sessionFile: string;
  name?: string;
  cwd: string;
  updatedAt: number;
}

export interface WorkspaceInfo {
  repository: boolean;
  branch?: string;
  changedFiles: number;
}

export interface WorkspaceFilePreview {
  path: string;
  text: string;
}

export interface WorkspaceFileChange {
  path: string;
  status: "added" | "modified" | "deleted" | "renamed";
  additions: number;
  deletions: number;
  patch: string;
}

export interface WorkspaceChanges {
  files: readonly WorkspaceFileChange[];
}

export interface WorktreeCheckpoint {
  commitId: string;
  gitRef: string;
  skippedPaths: string[];
}

export interface WorktreeRestore {
  safetyCheckpoint: WorktreeCheckpoint;
  changedPaths: string[];
}

export interface PiSkill {
  id: string;
  name: string;
  description: string;
  scope: "project" | "user";
  source: "pi" | "shared";
  path: string;
  content: string;
}

export interface PersistedAgentProfile {
  id: string;
  name: string;
  cwd: string;
  provider: string;
  model: string;
}

export interface PersistedAppSettings {
  locale: "en" | "zh";
  proxy: string;
  noProxy: string;
  provider: string;
  model: string;
  subagentsEnabled: boolean;
}

interface PiCapability extends NativeCapability {
  __wabouCapabilityVersion: number;
  getStatus(request: string): string | PromiseLike<string>;
  start(request: string): string | PromiseLike<string>;
  prompt(request: string): string | PromiseLike<string>;
  steer(request: string): string | PromiseLike<string>;
  followUp(request: string): string | PromiseLike<string>;
  abort(request: string): string | PromiseLike<string>;
  stop(request: string): string | PromiseLike<string>;
  newSession(request: string): string | PromiseLike<string>;
  renameSession(request: string): string | PromiseLike<string>;
  cycleModel(request: string): string | PromiseLike<string>;
  cycleThinking(request: string): string | PromiseLike<string>;
  setModel(request: string): string | PromiseLike<string>;
  getModelOptions(request: string): string | PromiseLike<string>;
  setThinking(request: string): string | PromiseLike<string>;
  setAutoCompaction(request: string): string | PromiseLike<string>;
  setSteeringMode(request: string): string | PromiseLike<string>;
  setFollowUpMode(request: string): string | PromiseLike<string>;
  listSessions(request: string): string | PromiseLike<string>;
  getMessages(request: string): string | PromiseLike<string>;
  getSessionStats(request: string): string | PromiseLike<string>;
  getCommands(request: string): string | PromiseLike<string>;
  getForkMessages(request: string): string | PromiseLike<string>;
  fork(request: string): string | PromiseLike<string>;
  cloneSession(request: string): string | PromiseLike<string>;
  compactSession(request: string): string | PromiseLike<string>;
  exportSession(request: string): string | PromiseLike<string>;
  listWorkspaceFiles(request: {
    cwd: string;
  }): string[] | PromiseLike<string[]>;
  workspaceInfo(request: {
    cwd: string;
  }): WorkspaceInfo | PromiseLike<WorkspaceInfo>;
  readWorkspaceFile(request: {
    cwd: string;
    path: string;
  }): WorkspaceFilePreview | PromiseLike<WorkspaceFilePreview>;
  workspaceChanges(request: {
    cwd: string;
  }): WorkspaceChanges | PromiseLike<WorkspaceChanges>;
  captureCheckpoint(request: {
    cwd: string;
    namespace: string;
    sequence: number;
  }): WorktreeCheckpoint | PromiseLike<WorktreeCheckpoint>;
  restoreCheckpoint(request: {
    cwd: string;
    commitId: string;
    namespace: string;
    sequence: number;
  }): WorktreeRestore | PromiseLike<WorktreeRestore>;
  retainCheckpoint(request: {
    cwd: string;
    commitId: string;
    sessionId: string;
    entryId: string;
  }): WorktreeCheckpoint | PromiseLike<WorktreeCheckpoint>;
  findCheckpoint(request: {
    cwd: string;
    sessionId: string;
    entryId: string;
  }):
    | WorktreeCheckpoint
    | undefined
    | PromiseLike<WorktreeCheckpoint | undefined>;
  listSkills(request: { cwd: string }): PiSkill[] | PromiseLike<PiSkill[]>;
  respondExtensionUi(request: string): string | PromiseLike<string>;
  listAgents(request?: undefined): PersistedAgentProfile[];
  saveAgents(request: string): string | PromiseLike<string>;
  deleteAgent(request: string): string | PromiseLike<string>;
  defaultWorkspace(request: string): string | PromiseLike<string>;
}

interface PiHost extends Host {
  piAgent: PiCapability;
}

export function usePiApi() {
  const host = useHost<PiHost>();
  const capability = bindCapability(host.piAgent, {
    name: "piAgent",
    version: 1,
  });
  const call = bindJsonCapability(capability, {
    name: "piAgent",
    version: 1,
  });
  return {
    listAgents: () => capability.listAgents(),
    saveAgents: (agents: readonly PersistedAgentProfile[]) =>
      call<void>("saveAgents", agents),
    getAppSettings: () => call<PersistedAppSettings>("getAppSettings"),
    saveAppSettings: (settings: PersistedAppSettings) =>
      call<void>("saveAppSettings", settings),
    deleteAgent: (agentId: string) => call<void>("deleteAgent", { agentId }),
    defaultWorkspace: (agentId: string) =>
      call<string>("defaultWorkspace", { agentId }),
    getStatus: (agentId: string) => call<PiStatus>("getStatus", { agentId }),
    start: (options: {
      agentId: string;
      cwd: string;
      proxy?: string;
      noProxy?: string;
      provider?: string;
      model?: string;
      sessionId?: string;
      subagentsEnabled?: boolean;
    }) => call<PiStatus>("start", options),
    prompt: (
      agentId: string,
      requestId: string,
      message: string,
      imagePaths: readonly string[] = [],
      contextPaths: readonly string[] = [],
    ) =>
      call<void>("prompt", {
        agentId,
        requestId,
        message,
        imagePaths,
        contextPaths,
      }),
    steer: (
      agentId: string,
      requestId: string,
      message: string,
      imagePaths: readonly string[] = [],
      contextPaths: readonly string[] = [],
    ) =>
      call<void>("steer", {
        agentId,
        requestId,
        message,
        imagePaths,
        contextPaths,
      }),
    followUp: (
      agentId: string,
      requestId: string,
      message: string,
      imagePaths: readonly string[] = [],
      contextPaths: readonly string[] = [],
    ) =>
      call<void>("followUp", {
        agentId,
        requestId,
        message,
        imagePaths,
        contextPaths,
      }),
    abort: (agentId: string) => call<void>("abort", { agentId }),
    stop: (agentId: string) => call<void>("stop", { agentId }),
    newSession: (agentId: string) => call<void>("newSession", { agentId }),
    renameSession: (agentId: string, name: string) =>
      call<void>("renameSession", { agentId, name }),
    listSessions: (agentId: string) =>
      call<PiSession[]>("listSessions", { agentId }),
    getMessages: (agentId: string, requestId: string) =>
      call<void>("getMessages", { agentId, requestId }),
    getSessionStats: (agentId: string) =>
      call<void>("getSessionStats", { agentId }),
    getCommands: (agentId: string) => call<void>("getCommands", { agentId }),
    getForkMessages: (agentId: string) =>
      call<void>("getForkMessages", { agentId }),
    fork: (agentId: string, entryId: string) =>
      call<void>("fork", { agentId, entryId }),
    cloneSession: (agentId: string) => call<void>("cloneSession", { agentId }),
    compactSession: (agentId: string) =>
      call<void>("compactSession", { agentId }),
    exportSession: (agentId: string, outputPath: string) =>
      call<void>("exportSession", { agentId, outputPath }),
    listWorkspaceFiles: async (cwd: string) =>
      capability.listWorkspaceFiles({ cwd }),
    workspaceInfo: async (cwd: string) => capability.workspaceInfo({ cwd }),
    readWorkspaceFile: async (cwd: string, path: string) =>
      capability.readWorkspaceFile({ cwd, path }),
    workspaceChanges: async (cwd: string) =>
      capability.workspaceChanges({ cwd }),
    captureCheckpoint: async (
      cwd: string,
      namespace: string,
      sequence: number,
    ) => capability.captureCheckpoint({ cwd, namespace, sequence }),
    restoreCheckpoint: async (
      cwd: string,
      commitId: string,
      namespace: string,
      sequence: number,
    ) => capability.restoreCheckpoint({ cwd, commitId, namespace, sequence }),
    retainCheckpoint: async (
      cwd: string,
      commitId: string,
      sessionId: string,
      entryId: string,
    ) => capability.retainCheckpoint({ cwd, commitId, sessionId, entryId }),
    findCheckpoint: async (cwd: string, sessionId: string, entryId: string) =>
      capability.findCheckpoint({ cwd, sessionId, entryId }),
    listSkills: async (cwd: string) => capability.listSkills({ cwd }),
    respondExtensionUi: (
      agentId: string,
      response:
        | { id: string; value: string }
        | { id: string; confirmed: boolean }
        | { id: string; cancelled: true },
    ) => call<void>("respondExtensionUi", { agentId, ...response }),
    cycleModel: (agentId: string) => call<void>("cycleModel", { agentId }),
    cycleThinking: (agentId: string) =>
      call<void>("cycleThinking", { agentId }),
    setModel: (agentId: string, provider: string, modelId: string) =>
      call<void>("setModel", { agentId, provider, modelId }),
    getModelOptions: (agentId: string) =>
      call<void>("getModelOptions", { agentId }),
    setThinking: (agentId: string, level: string) =>
      call<void>("setThinking", { agentId, level }),
    setAutoCompaction: (agentId: string, enabled: boolean) =>
      call<void>("setAutoCompaction", { agentId, enabled }),
    setSteeringMode: (agentId: string, mode: "all" | "one-at-a-time") =>
      call<void>("setSteeringMode", { agentId, mode }),
    setFollowUpMode: (agentId: string, mode: "all" | "one-at-a-time") =>
      call<void>("setFollowUpMode", { agentId, mode }),
    subscribe: (
      listener: (events: readonly Record<string, unknown>[]) => void,
    ) => hostMessages.subscribeJson("pi.event", listener),
  };
}
