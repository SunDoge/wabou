import { type AgentViewState, initialAgentState } from "./agent-state";

export interface AgentWorkspace {
  id: string;
  name: string;
  cwd: string;
  proxy: string;
  noProxy: string;
  provider: string;
  model: string;
  state: AgentViewState;
}

export function createAgentWorkspace(index: number): AgentWorkspace {
  return {
    id: `agent-${index}`,
    name: `Agent ${index}`,
    cwd: "",
    proxy: "",
    noProxy: "127.0.0.1,localhost",
    provider: "",
    model: "",
    state: { ...initialAgentState, items: [] },
  };
}
