import {
  bindJsonCapability,
  type Host,
  hostMessages,
  type NativeJsonCapability,
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

export interface PersistedAgentProfile {
  id: string;
  name: string;
  cwd: string;
  proxy: string;
  noProxy: string;
  provider: string;
  model: string;
}

interface PiCapability extends NativeJsonCapability {
  __wabouCapabilityVersion: number;
  getStatus(request: string): string | PromiseLike<string>;
  start(request: string): string | PromiseLike<string>;
  prompt(request: string): string | PromiseLike<string>;
  abort(request: string): string | PromiseLike<string>;
  stop(request: string): string | PromiseLike<string>;
  newSession(request: string): string | PromiseLike<string>;
  cycleModel(request: string): string | PromiseLike<string>;
  cycleThinking(request: string): string | PromiseLike<string>;
  setModel(request: string): string | PromiseLike<string>;
  listSessions(request: string): string | PromiseLike<string>;
  getMessages(request: string): string | PromiseLike<string>;
  listAgents(request?: string): string | PromiseLike<string>;
  saveAgents(request: string): string | PromiseLike<string>;
  deleteAgent(request: string): string | PromiseLike<string>;
  defaultWorkspace(request: string): string | PromiseLike<string>;
}

interface PiHost extends Host {
  piAgent: PiCapability;
}

export function usePiApi() {
  const host = useHost<PiHost>();
  const call = bindJsonCapability(host.piAgent, {
    name: "piAgent",
    version: 1,
  });
  return {
    listAgents: () => call<PersistedAgentProfile[]>("listAgents"),
    saveAgents: (agents: readonly PersistedAgentProfile[]) =>
      call<void>("saveAgents", agents),
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
    }) => call<PiStatus>("start", options),
    prompt: (agentId: string, message: string) =>
      call<void>("prompt", { agentId, message }),
    abort: (agentId: string) => call<void>("abort", { agentId }),
    stop: (agentId: string) => call<void>("stop", { agentId }),
    newSession: (agentId: string) => call<void>("newSession", { agentId }),
    listSessions: (agentId: string) =>
      call<PiSession[]>("listSessions", { agentId }),
    getMessages: (agentId: string) => call<void>("getMessages", { agentId }),
    cycleModel: (agentId: string) => call<void>("cycleModel", { agentId }),
    cycleThinking: (agentId: string) =>
      call<void>("cycleThinking", { agentId }),
    setModel: (agentId: string, provider: string, modelId: string) =>
      call<void>("setModel", { agentId, provider, modelId }),
    subscribe: (
      listener: (events: readonly Record<string, unknown>[]) => void,
    ) => hostMessages.subscribeJson("pi.event", listener),
  };
}
