import { describe, expect, test } from "vitest";
import {
  agentProfile,
  createAgentWorkspace,
  resolveActiveAgentId,
  restoreAgentWorkspace,
} from "./workspace";

describe("Pi agent workspace", () => {
  test("never resolves an identity that is absent from the workspace list", () => {
    const agents = [{ id: "agent-2" }, { id: "agent-3" }];
    expect(resolveActiveAgentId(agents, "agent-1", "agent-1")).toBe(
      "agent-2",
    );
    expect(resolveActiveAgentId(agents, "agent-3", "agent-1")).toBe(
      "agent-3",
    );
    expect(resolveActiveAgentId([], "agent-1", "agent-1")).toBeUndefined();
  });

  test("creates isolated project identities and mutable configuration values", () => {
    const first = createAgentWorkspace(1);
    const second = createAgentWorkspace(2);
    expect(first.id).toBe("agent-1");
    expect(second.id).toBe("agent-2");
    expect(first.state).not.toBe(second.state);
    expect(first.name).toBe("Project 1");
  });

  test("persists configuration without serializing transient conversation state", () => {
    const original = createAgentWorkspace(3);
    original.cwd = "/workspace/pi";
    original.state = {
      ...original.state,
      connection: "running",
      items: [{ kind: "user", id: "message-1", text: "hello" }],
    };
    const profile = agentProfile(original);
    expect(profile).not.toHaveProperty("state");
    expect(profile.cwd).toBe("/workspace/pi");

    const restored = restoreAgentWorkspace(profile);
    expect(restored.id).toBe("agent-3");
    expect(restored.state.connection).toBe("stopped");
    expect(restored.state.items).toEqual([]);
  });
});
