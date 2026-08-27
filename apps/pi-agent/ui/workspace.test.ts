import { describe, expect, test } from "vitest";
import { createAgentWorkspace } from "./workspace";

describe("Pi agent workspace", () => {
  test("creates isolated agent identities and mutable configuration values", () => {
    const first = createAgentWorkspace(1);
    const second = createAgentWorkspace(2);
    expect(first.id).toBe("agent-1");
    expect(second.id).toBe("agent-2");
    expect(first.state).not.toBe(second.state);
    expect(first.noProxy).toContain("localhost");
  });
});
