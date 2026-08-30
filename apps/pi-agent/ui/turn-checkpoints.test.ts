import { describe, expect, test } from "bun:test";
import type { WorktreeCheckpoint, WorktreeRestore } from "./api";
import { TurnCheckpointCoordinator } from "./turn-checkpoints";

const checkpoint = (commitId: string): WorktreeCheckpoint => ({
  commitId,
  gitRef: `refs/checkpoints/${commitId}`,
  skippedPaths: [],
});

describe("TurnCheckpointCoordinator", () => {
  test("attaches an optimistic message checkpoint to its durable Pi entry", async () => {
    const retained: unknown[][] = [];
    const coordinator = new TurnCheckpointCoordinator({
      captureCheckpoint: async () => checkpoint("before-turn"),
      retainCheckpoint: async (...args: unknown[]) => {
        retained.push(args);
        return checkpoint("before-turn");
      },
      findCheckpoint: async () => undefined,
      restoreCheckpoint: async () => undefined as never,
    });

    await coordinator.capture("user-1", "/repo", "agent-1");
    coordinator.synchronize("/repo", "session-1", [
      { id: "user-1", kind: "user", text: "change it", entryId: "entry-1" },
    ]);
    await Promise.resolve();

    expect(retained).toEqual([
      ["/repo", "before-turn", "session-1", "entry-1"],
    ]);
  });

  test("restores the target and can roll back through its safety snapshot", async () => {
    const restores: unknown[][] = [];
    const target = checkpoint("target");
    const safety = checkpoint("safety");
    const result: WorktreeRestore = {
      safetyCheckpoint: safety,
      changedPaths: ["src/app.ts"],
    };
    const coordinator = new TurnCheckpointCoordinator({
      captureCheckpoint: async () => target,
      retainCheckpoint: async () => target,
      findCheckpoint: async () => target,
      restoreCheckpoint: async (...args: unknown[]) => {
        restores.push(args);
        return restores.length === 1 ? result : { ...result, changedPaths: [] };
      },
    });

    const rewind = await coordinator.prepareRewind(
      "/repo",
      "session-1",
      "entry-1",
    );
    expect(rewind).toEqual({ target, restore: result });
    if (!rewind) throw new Error("expected a prepared rewind");
    await coordinator.rollback("/repo", rewind);
    expect(restores[0]?.slice(0, 2)).toEqual(["/repo", "target"]);
    expect(restores[1]?.slice(0, 2)).toEqual(["/repo", "safety"]);
  });
});
