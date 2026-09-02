import { describe, expect, test } from "vitest";
import { activeRuntimeRouteKey, createSingleFlight } from "./runtime-start";

describe("Pi runtime startup", () => {
  test("shares an in-flight launch with every identical caller", async () => {
    let launches = 0;
    let finish!: (value: boolean) => void;
    const deferred = new Promise<boolean>((resolve) => {
      finish = resolve;
    });
    const run = createSingleFlight<string, boolean>();
    const first = run("agent-2/session-1", () => {
      launches += 1;
      return deferred;
    });
    const second = run("agent-2/session-1", () => {
      launches += 1;
      return Promise.resolve(false);
    });

    expect(second).toBe(first);
    expect(launches).toBe(1);
    finish(true);
    await expect(first).resolves.toBe(true);
  });

  test("reduces the active project and session route to a stable scalar key", () => {
    const sessions = [{ agentId: "agent-2", sessionId: "session-1" }];
    expect(
      activeRuntimeRouteKey("agent-2", "session-1", ["agent-2"], sessions),
    ).toBe("agent-2\0session-1");
    expect(
      activeRuntimeRouteKey(
        "agent-2",
        "session-1",
        ["agent-2"],
        sessions.map((session) => ({ ...session })),
      ),
    ).toBe("agent-2\0session-1");
  });

  test("launches an active project without waiting for a first prompt", () => {
    expect(activeRuntimeRouteKey("agent-2", undefined, ["agent-2"], [])).toBe(
      "agent-2\0",
    );
  });

  test("waits for a historical session to be present in its project", () => {
    expect(activeRuntimeRouteKey("agent-2", "missing", ["agent-2"], [])).toBe(
      "",
    );
    expect(
      activeRuntimeRouteKey(
        "agent-2",
        "session-1",
        ["agent-1"],
        [{ agentId: "agent-2", sessionId: "session-1" }],
      ),
    ).toBe("");
  });
});
