import { describe, expect, test } from "bun:test";
import type { Aria2Task } from "./aria2";
import { projectTasks } from "./task-list";

function task(
  gid: string,
  name: string,
  status: string,
  totalLength: number,
  completedLength: number,
  downloadSpeed: number,
): Aria2Task {
  return {
    gid,
    name,
    status,
    totalLength,
    completedLength,
    downloadSpeed,
    uploadSpeed: 0,
    uploadedLength: 0,
    dir: "/downloads",
    connections: 1,
    bittorrent: false,
    fileCount: 1,
  };
}

const tasks = [
  task("1", "Zeta", "active", 100, 50, 10),
  task("2", "Alpha", "paused", 1_000, 100, 5),
  task("3", "Beta", "complete", 500, 500, 0),
];

describe("Motrix task projection", () => {
  test("preserves aria2 queue order by default", () => {
    expect(
      projectTasks(tasks, "all", "", "queue").map((task) => task.gid),
    ).toEqual(["1", "2", "3"]);
  });

  test("combines status filtering, search and deterministic sorting", () => {
    expect(
      projectTasks(tasks, "waiting", "alp", "name").map((task) => task.gid),
    ).toEqual(["2"]);
    expect(
      projectTasks(tasks, "all", "", "size").map((task) => task.gid),
    ).toEqual(["2", "3", "1"]);
    expect(
      projectTasks(tasks, "all", "", "progress").map((task) => task.gid),
    ).toEqual(["3", "1", "2"]);
  });
});
