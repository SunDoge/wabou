import type { AgentItem } from "./agent-state";
import type { usePiApi, WorktreeCheckpoint, WorktreeRestore } from "./api";

type PiApi = Pick<
  ReturnType<typeof usePiApi>,
  | "captureCheckpoint"
  | "retainCheckpoint"
  | "findCheckpoint"
  | "restoreCheckpoint"
>;

interface PendingCheckpoint {
  cwd: string;
  checkpoint: WorktreeCheckpoint;
}

export interface PreparedRewind {
  target: WorktreeCheckpoint;
  restore: WorktreeRestore;
}

/**
 * Bridges transient optimistic message IDs to Pi's durable session entry IDs.
 * The durable association itself lives in Git refs, so restarting the UI does
 * not erase checkpoints that were fully attached to a Pi entry.
 */
export class TurnCheckpointCoordinator {
  readonly #api: PiApi;
  readonly #pending = new Map<string, PendingCheckpoint>();
  readonly #retaining = new Set<string>();
  #sequence = 1;

  constructor(api: PiApi) {
    this.#api = api;
  }

  async capture(
    messageId: string,
    cwd: string,
    provisionalNamespace: string,
  ): Promise<void> {
    const checkpoint = await this.#api.captureCheckpoint(
      cwd,
      provisionalNamespace,
      this.#sequence++,
    );
    this.#pending.set(messageId, { cwd, checkpoint });
  }

  synchronize(
    cwd: string,
    sessionId: string | undefined,
    items: readonly AgentItem[],
  ): void {
    if (!sessionId) return;
    for (const item of items) {
      if (item.kind !== "user" || !item.entryId) continue;
      const pending = this.#pending.get(item.id);
      if (!pending || pending.cwd !== cwd) continue;
      const key = `${sessionId}\0${item.entryId}`;
      if (this.#retaining.has(key)) continue;
      this.#retaining.add(key);
      void this.#api
        .retainCheckpoint(
          cwd,
          pending.checkpoint.commitId,
          sessionId,
          item.entryId,
        )
        .then(() => this.#pending.delete(item.id))
        .catch((error) =>
          console.warn(
            `[pi-agent] could not retain turn checkpoint: ${String(error)}`,
          ),
        )
        .finally(() => this.#retaining.delete(key));
    }
  }

  discard(messageId: string): void {
    this.#pending.delete(messageId);
  }

  async prepareRewind(
    cwd: string,
    sessionId: string,
    entryId: string,
  ): Promise<PreparedRewind | undefined> {
    const target = await this.#api.findCheckpoint(cwd, sessionId, entryId);
    if (!target) return undefined;
    const restore = await this.#api.restoreCheckpoint(
      cwd,
      target.commitId,
      `${sessionId}-rewind`,
      this.#sequence++,
    );
    return { target, restore };
  }

  async rollback(cwd: string, rewind: PreparedRewind): Promise<void> {
    await this.#api.restoreCheckpoint(
      cwd,
      rewind.restore.safetyCheckpoint.commitId,
      "rewind-rollback",
      this.#sequence++,
    );
  }
}
