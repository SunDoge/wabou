import { type Accessor, createEffect, createSignal } from "solid-js";
import type { usePiApi } from "./api";
import type { DeferredWriterScheduler } from "./deferred-writer";
import { createPersistedValue } from "./persisted-record";
import {
  type AgentWorkspace,
  agentProfile,
  createAgentWorkspace,
  resolveActiveAgentId,
  restoreAgentWorkspace,
} from "./workspace";

type PiApi = ReturnType<typeof usePiApi>;

export function createAgentProfiles(options: {
  api: PiApi;
  routeAgentId: Accessor<string | undefined>;
  scheduler?: DeferredWriterScheduler;
}) {
  const { api } = options;
  const [lastActiveId, setLastActiveId] = createSignal("agent-1");
  const repairedLoads = new WeakSet<object>();
  const profiles = createPersistedValue<readonly AgentWorkspace[]>({
    initial: [createAgentWorkspace(1)],
    load: async () => {
      const stored = await api.listAgents();
      const loaded =
        stored.length > 0
          ? stored.map(restoreAgentWorkspace)
          : [createAgentWorkspace(1)];
      let repaired = stored.length === 0;
      const agents = await Promise.all(
        loaded.map(async (agent) => {
          if (agent.cwd.trim()) return agent;
          repaired = true;
          const cwd = await api.defaultWorkspace(agent.id);
          return {
            ...agent,
            cwd,
          };
        }),
      );
      if (repaired) repairedLoads.add(agents);
      return agents;
    },
    save: (agents) => api.saveAgents(agents.map(agentProfile)),
    onLoadError: (error) =>
      console.error(`[pi-agent] could not load projects: ${String(error)}`),
    onSaveError: (error) =>
      console.error(`[pi-agent] could not save projects: ${String(error)}`),
    scheduler: options.scheduler,
    equals: (left, right) =>
      JSON.stringify(left.map(agentProfile)) ===
      JSON.stringify(right.map(agentProfile)),
  });
  const agents = profiles.value;
  const setAgents = (
    next:
      | readonly AgentWorkspace[]
      | ((current: readonly AgentWorkspace[]) => readonly AgentWorkspace[]),
  ) => {
    if (typeof next === "function") profiles.update(next);
    else profiles.set(next);
  };

  const updateAgent = (
    id: string,
    update: (agent: AgentWorkspace) => AgentWorkspace,
  ) =>
    setAgents((current) =>
      current.map((agent) => (agent.id === id ? update(agent) : agent)),
    );

  const prepareDefaultWorkspace = async (id: string): Promise<void> => {
    try {
      const cwd = await api.defaultWorkspace(id);
      updateAgent(id, (agent) => (agent.cwd ? agent : { ...agent, cwd }));
    } catch (error) {
      console.error(
        `[pi-agent] could not prepare the default workspace: ${String(error)}`,
      );
    }
  };

  createEffect(agents, (current) => {
    if (repairedLoads.delete(current)) {
      profiles.set(current);
    }
    if (!current.some((agent) => agent.id === lastActiveId())) {
      setLastActiveId(current[0]?.id ?? "");
    }
  });
  createEffect(options.routeAgentId, (routeId) => {
    if (routeId && agents().some((agent) => agent.id === routeId)) {
      setLastActiveId(routeId);
    }
  });

  const activeId = () =>
    resolveActiveAgentId(agents(), options.routeAgentId(), lastActiveId()) ??
    "";
  const active = () =>
    agents().find((agent) => agent.id === activeId()) ?? agents()[0];
  const patchActive = (patch: Partial<AgentWorkspace>) => {
    const id = active().id;
    updateAgent(id, (agent) => ({ ...agent, ...patch }));
  };

  return {
    agents,
    setAgents,
    active,
    activeId,
    setLastActiveId,
    updateAgent,
    patchActive,
    prepareDefaultWorkspace,
    loadError: profiles.loadError,
    saveError: profiles.saveError,
    reload: profiles.reload,
    retrySave: profiles.retrySave,
  };
}
