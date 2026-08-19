import { describe, expect, test } from "bun:test";
import {
  applySnapshotPatch,
  type Aria2Snapshot,
  type Aria2SnapshotPatch,
  type Aria2Task,
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
    activity: Array(84).fill(0),
    downloadedToday: 0,
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
    activity: Array(83).fill(0).concat(45),
    downloadedToday: 45,
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
