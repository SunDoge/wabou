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
