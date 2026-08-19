import { describe, expect, test } from "bun:test";
import {
  type Aria2Snapshot,
  type Aria2SnapshotPatch,
  type Aria2Task,
  appendTaskSpeedHistories,
  applySnapshotPatch,
  terminalTaskTransitions,
} from "./aria2";

function task(gid: string, downloadSpeed: number): Aria2Task {
  return {
    gid,
    name: `task-${gid}`,
    status: "active",
    totalLength: 100,
    completedLength: 25,
    downloadSpeed,
    uploadSpeed: 0,
    uploadedLength: 0,
    dir: "/downloads",
    connections: 1,
    bittorrent: false,
    retryable: true,
    archived: false,
    fileCount: 1,
  };
}

function snapshot(): Aria2Snapshot {
  return {
    revision: 4,
    connected: true,
    endpoint: "ws://127.0.0.1:6800/jsonrpc",
    version: "1.37.0",
    downloadSpeed: 30,
    uploadSpeed: 0,
    tasks: [task("a", 10), task("b", 20), task("removed", 0)],
    managed: true,
    engineRunning: true,
    activity: Array(364).fill(0),
    downloadedToday: 0,
    nat: { enabled: false, state: "disabled" },
  };
}

function patch(): Aria2SnapshotPatch {
  return {
    revision: 5,
    baseRevision: 4,
    connected: true,
    endpoint: "ws://127.0.0.1:6800/jsonrpc",
    version: "1.37.0",
    downloadSpeed: 52,
    uploadSpeed: 0,
    managed: true,
    engineRunning: true,
    activity: Array(363).fill(0).concat(45),
    downloadedToday: 45,
    nat: {
      enabled: true,
      state: "mapped",
      tcpExternalAddress: "203.0.113.1:6881",
    },
    upsertedTasks: [task("b", 42), task("new", 0)],
    removedGids: ["removed"],
    taskOrder: ["b", "a", "new"],
  };
}

describe("aria2 snapshot patches", () => {
  test("reconstructs the exact ordered snapshot", () => {
    const next = applySnapshotPatch(snapshot(), patch());

    expect(next?.revision).toBe(5);
    expect(next?.downloadSpeed).toBe(52);
    expect(next?.tasks.map((value) => value.gid)).toEqual(["b", "a", "new"]);
    expect(next?.tasks[0]?.downloadSpeed).toBe(42);
    expect(next?.nat.tcpExternalAddress).toBe("203.0.113.1:6881");
  });

  test("rejects a missed revision or incomplete task order", () => {
    expect(
      applySnapshotPatch(snapshot(), { ...patch(), baseRevision: 3 }),
    ).toBeUndefined();
    expect(
      applySnapshotPatch(snapshot(), {
        ...patch(),
        taskOrder: ["b", "missing"],
      }),
    ).toBeUndefined();
  });
});

describe("aria2 terminal task events", () => {
  test("suppresses history but reports a newly observed immediate failure", () => {
    const statuses = new Map<string, string>();
    const historical = { ...task("old", 0), status: "complete" as const };
    expect(terminalTaskTransitions(statuses, [historical], true)).toEqual([]);

    const failed = { ...task("new", 0), status: "error" as const };
    expect(
      terminalTaskTransitions(statuses, [historical, failed], false),
    ).toEqual([failed]);
    expect(
      terminalTaskTransitions(statuses, [historical, failed], false),
    ).toEqual([]);
  });
});

describe("aria2 task speed histories", () => {
  test("bounds samples and removes tasks no longer present", () => {
    let histories = appendTaskSpeedHistories(
      {},
      [task("a", 10), task("b", 20)],
      2,
    );
    histories = appendTaskSpeedHistories(histories, [task("a", 30)], 2);
    histories = appendTaskSpeedHistories(histories, [task("a", 40)], 2);

    expect(histories.a?.download).toEqual([30, 40]);
    expect(histories.b).toBeUndefined();
  });
});
