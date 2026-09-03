import { type AgentViewState, initialAgentState } from "./agent-state";

export interface AgentWorkspace {
  id: string;
  name: string;
  cwd: string;
  provider: string;
  model: string;
  state: AgentViewState;
}

export type AgentProfile = Omit<AgentWorkspace, "state">;

export function resolveActiveAgentId(
  agents: readonly Pick<AgentWorkspace, "id">[],
  routeId: string | undefined,
  lastActiveId: string | undefined,
): string | undefined {
  if (routeId && agents.some((agent) => agent.id === routeId)) return routeId;
  if (lastActiveId && agents.some((agent) => agent.id === lastActiveId)) {
    return lastActiveId;
  }
  return agents[0]?.id;
}

export function agentProfile(agent: AgentWorkspace): AgentProfile {
  const { state: _state, ...profile } = agent;
  return profile;
}

export function restoreAgentWorkspace(profile: AgentProfile): AgentWorkspace {
  return {
    ...profile,
    state: { ...initialAgentState, items: [] },
  };
}

export function createAgentWorkspace(index: number): AgentWorkspace {
  return {
    id: `agent-${index}`,
    name: `Project ${index}`,
    cwd: "",
    provider: "",
    model: "",
    state: { ...initialAgentState, items: [] },
  };
}
