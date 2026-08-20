import { describe, expect, test } from "bun:test";
import {
  appendTaskSpeedHistories,
  applySnapshotPatch,
  type DownloadSnapshot,
  type DownloadSnapshotPatch,
  type DownloadTask,
  snapshotReflectsTaskAction,
  terminalTaskTransitions,
} from "./downloads";

function task(id: string, downloadSpeed: number): DownloadTask {
  return {
    id,
    name: `task-${id}`,
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
    priority: "normal",
    createdAtMs: Number(id) * 1_000,
  };
}

function snapshot(): DownloadSnapshot {
  return {
    revision: 4,
    status: "ready",
    version: "gosh-dl 0.5.0",
    downloadSpeed: 30,
    uploadSpeed: 0,
    tasks: [task("a", 10), task("b", 20), task("removed", 0)],
    activity: Array(364).fill(0),
    downloadedToday: 0,
    downloadedTotal: 0,
    uploadedTotal: 0,
    nat: { enabled: false, state: "disabled" },
  };
}

function patch(): DownloadSnapshotPatch {
  return {
    revision: 5,
    baseRevision: 4,
    status: "ready",
    version: "gosh-dl 0.5.0",
    downloadSpeed: 52,
    uploadSpeed: 0,
    activity: Array(363).fill(0).concat(45),
    downloadedToday: 45,
    downloadedTotal: 145,
    uploadedTotal: 12,
    nat: {
      enabled: true,
      state: "mapped",
      tcpExternalAddress: "203.0.113.1:6881",
    },
    upsertedTasks: [task("b", 42), task("new", 0)],
    removedIds: ["removed"],
    taskOrder: ["b", "a", "new"],
  };
}

describe("downloads snapshot patches", () => {
  test("reconstructs the exact ordered snapshot", () => {
    const next = applySnapshotPatch(snapshot(), patch());

    expect(next?.revision).toBe(5);
    expect(next?.downloadSpeed).toBe(52);
    expect(next?.tasks.map((value) => value.id)).toEqual(["b", "a", "new"]);
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
    expect(
      applySnapshotPatch(snapshot(), {
        ...patch(),
        removedIds: [],
        taskOrder: ["b", "a", "new", "a"],
      }),
    ).toBeUndefined();
  });
});

describe("downloads terminal task events", () => {
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

describe("download action synchronization", () => {
  test("matches host-owned task transitions without requiring an exact intermediate state", () => {
    const active = snapshot();
    expect(snapshotReflectsTaskAction(active, "a", "pause")).toBe(false);
    expect(
      snapshotReflectsTaskAction(
        {
          ...active,
          tasks: active.tasks.map((value) =>
            value.id === "a" ? { ...value, status: "paused" } : value,
          ),
        },
        "a",
        "pause",
      ),
    ).toBe(true);
    expect(
      snapshotReflectsTaskAction(
        { ...active, tasks: active.tasks.filter((value) => value.id !== "a") },
        "a",
        "remove",
      ),
    ).toBe(true);
  });
});

describe("downloads task speed histories", () => {
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
