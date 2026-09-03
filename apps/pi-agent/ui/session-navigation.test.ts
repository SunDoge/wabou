import { describe, expect, test } from "vitest";
import {
  createSessionNavigation,
  SessionNavigation,
} from "./session-navigation";

describe("SessionNavigation", () => {
  test("moves through explicitly visited sessions", () => {
    const history = new SessionNavigation();
    history.visit({ agentId: "one", sessionId: "a" });
    history.visit({ agentId: "one", sessionId: "b" });
    history.visit({ agentId: "two", sessionId: "c" });

    expect(history.canGoBack).toBe(true);
    expect(history.canGoForward).toBe(false);
    expect(history.back()).toEqual({ agentId: "one", sessionId: "b" });
    expect(history.back()).toEqual({ agentId: "one", sessionId: "a" });
    expect(history.canGoBack).toBe(false);
    expect(history.forward()).toEqual({ agentId: "one", sessionId: "b" });
  });

  test("does not duplicate the active session", () => {
    const history = new SessionNavigation();
    expect(history.visit({ agentId: "one", sessionId: "a" })).toBe(true);
    expect(history.visit({ agentId: "one", sessionId: "a" })).toBe(false);
    expect(history.canGoBack).toBe(false);
  });

  test("a new visit replaces the forward branch", () => {
    const history = new SessionNavigation();
    history.visit({ agentId: "one", sessionId: "a" });
    history.visit({ agentId: "one", sessionId: "b" });
    history.visit({ agentId: "one", sessionId: "c" });
    history.back();
    history.visit({ agentId: "one", sessionId: "d" });

    expect(history.canGoForward).toBe(false);
    expect(history.back()).toEqual({ agentId: "one", sessionId: "b" });
  });

  test("removes deleted projects without retaining dead navigation targets", () => {
    const history = new SessionNavigation();
    history.visit({ agentId: "one", sessionId: "a" });
    history.visit({ agentId: "two", sessionId: "b" });
    history.visit({ agentId: "one", sessionId: "c" });

    expect(history.removeAgent("one")).toBe(true);
    expect(history.canGoBack).toBe(false);
    expect(history.canGoForward).toBe(false);
    expect(history.removeAgent("missing")).toBe(false);
  });

  test("the Solid controller owns its reactive invalidation", () => {
    const history = createSessionNavigation();
    expect(history.canGoBack()).toBe(false);
    history.visit({ agentId: "one", sessionId: "a" });
    history.visit({ agentId: "one", sessionId: "b" });
    expect(history.canGoBack()).toBe(true);
    expect(history.back()).toEqual({ agentId: "one", sessionId: "a" });
    expect(history.canGoForward()).toBe(true);
  });
});
