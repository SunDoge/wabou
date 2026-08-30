import { createEffect, createSignal, onCleanup, type Accessor } from "solid-js";
import type { usePiApi } from "./api";
import { createDeferredWriter } from "./deferred-writer";
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
}) {
  const { api } = options;
  const [agents, setAgents] = createSignal<readonly AgentWorkspace[]>([
    createAgentWorkspace(1),
  ]);
  const [lastActiveId, setLastActiveId] = createSignal("agent-1");
  let hydrated = false;
  let restored: boolean | undefined;
  const writer = createDeferredWriter({
    write: (serialized: string) => api.saveAgents(JSON.parse(serialized)),
    onError: (error) =>
      console.error(`[pi-agent] could not save projects: ${String(error)}`),
    equals: Object.is,
  });

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

  void api
    .listAgents()
    .then(async (profiles) => {
      if (profiles.length > 0) {
        restored = true;
        const next = profiles.map(restoreAgentWorkspace);
        setAgents(next);
        setLastActiveId(next[0].id);
      } else {
        restored = false;
        await prepareDefaultWorkspace("agent-1");
      }
    })
    .catch((error) => {
      console.error(
        `[pi-agent] could not prepare the default workspace: ${String(error)}`,
      );
    })
    .finally(() => {
      hydrated = true;
      const serialized = JSON.stringify(agents().map(agentProfile));
      if (restored === true) writer.prime(serialized);
      else if (restored === false) writer.schedule(serialized);
    });

  createEffect(
    () => JSON.stringify(agents().map(agentProfile)),
    (serialized) => {
      if (hydrated) writer.schedule(serialized);
    },
  );
  createEffect(options.routeAgentId, (routeId) => {
    if (routeId && agents().some((agent) => agent.id === routeId)) {
      setLastActiveId(routeId);
    }
  });

  const activeId = () =>
    resolveActiveAgentId(agents(), options.routeAgentId(), lastActiveId()) ?? "";
  const active = () =>
    agents().find((agent) => agent.id === activeId()) ?? agents()[0];
  const patchActive = (patch: Partial<AgentWorkspace>) => {
    const id = active().id;
    updateAgent(id, (agent) => ({ ...agent, ...patch }));
  };

  onCleanup(() => writer.flush());
  return {
    agents,
    setAgents,
    active,
    activeId,
    setLastActiveId,
    updateAgent,
    patchActive,
    prepareDefaultWorkspace,
  };
}
