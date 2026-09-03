import { describe, expect, test } from "bun:test";
import { ScopedHandleRegistry } from "./scoped-handle-registry";

describe("scoped handle registry", () => {
  test("does not resolve a repeated message id from another session", () => {
    const registry = new ScopedHandleRegistry<object>();
    const first = {};
    registry.synchronize("agent-1/session-a", ["restored-0"]);
    expect(registry.register("agent-1/session-a", "restored-0", first)).toBe(
      true,
    );
    expect(registry.resolve("agent-1/session-a", "restored-0")).toBe(first);

    registry.synchronize("agent-1/session-b", ["restored-0"]);
    expect(registry.resolve("agent-1/session-b", "restored-0")).toBeUndefined();
    expect(registry.size).toBe(0);
  });

  test("rejects a late registration from the previous session", () => {
    const registry = new ScopedHandleRegistry<object>();
    registry.synchronize("agent-1/session-a", ["message"]);
    registry.synchronize("agent-2/session-b", ["message"]);

    expect(registry.register("agent-1/session-a", "message", {})).toBe(false);
    expect(registry.resolve("agent-2/session-b", "message")).toBeUndefined();
  });

  test("prunes handles when messages leave the current transcript", () => {
    const registry = new ScopedHandleRegistry<object>();
    registry.synchronize("agent-1/session-a", ["one", "two"]);
    registry.register("agent-1/session-a", "one", {});
    registry.register("agent-1/session-a", "two", {});

    registry.synchronize("agent-1/session-a", ["two"]);
    expect(registry.resolve("agent-1/session-a", "one")).toBeUndefined();
    expect(registry.resolve("agent-1/session-a", "two")).toBeDefined();
    expect(registry.size).toBe(1);
  });
});
