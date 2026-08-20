import { describe, expect, test } from "bun:test";
import type { DownloadTask } from "./downloads";
import {
  primaryTaskAction,
  projectTasks,
  restartTaskAction,
} from "./task-list";

function task(
  id: string,
  name: string,
  status: string,
  totalLength: number,
  completedLength: number,
  downloadSpeed: number,
): DownloadTask {
  return {
    id,
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
    retryable: true,
    archived: false,
    fileCount: 1,
  };
}

const tasks = [
  task("1", "Zeta", "active", 100, 50, 10),
  task("2", "Alpha", "paused", 1_000, 100, 5),
  task("3", "Beta", "complete", 500, 500, 0),
];

describe("Motrix task projection", () => {
  test("preserves downloads queue order by default", () => {
    expect(
      projectTasks(tasks, "all", "", "queue").map((task) => task.id),
    ).toEqual(["1", "2", "3"]);
  });

  test("combines status filtering, search and deterministic sorting", () => {
    expect(
      projectTasks(tasks, "waiting", "alp", "name").map((task) => task.id),
    ).toEqual(["2"]);
    expect(
      projectTasks(tasks, "all", "", "size").map((task) => task.id),
    ).toEqual(["2", "3", "1"]);
    expect(
      projectTasks(tasks, "all", "", "progress").map((task) => task.id),
    ).toEqual(["3", "1", "2"]);
  });

  test("distinguishes retry from re-seeding a reconstructable torrent", () => {
    const torrent = {
      ...task("4", "Linux.iso", "error", 100, 10, 0),
      bittorrent: true,
    };
    expect(restartTaskAction(torrent)).toBe("retry");
    expect(restartTaskAction({ ...torrent, retryable: false })).toBeUndefined();
    expect(restartTaskAction({ ...torrent, status: "complete" })).toBe(
      "reseed",
    );
    expect(
      restartTaskAction({ ...torrent, status: "complete", bittorrent: false }),
    ).toBeUndefined();
  });

  test("maps only actionable states to a primary engine command", () => {
    const active = task("4", "Task", "active", 100, 10, 2);
    expect(primaryTaskAction(active)).toBe("pause");
    expect(primaryTaskAction({ ...active, status: "waiting" })).toBe("pause");
    expect(primaryTaskAction({ ...active, status: "paused" })).toBe("resume");
    expect(primaryTaskAction({ ...active, status: "seeding" })).toBe(
      "stopSeeding",
    );
    expect(
      primaryTaskAction({ ...active, status: "complete" }),
    ).toBeUndefined();
    expect(primaryTaskAction({ ...active, status: "error" })).toBeUndefined();
  });
});
