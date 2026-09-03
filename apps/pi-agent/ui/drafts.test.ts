import { describe, expect, test } from "bun:test";
import type { Kv, KvEntry, KvKey, KvListOptions, KvValue } from "@wabou/ui";
import { createRoot, createSignal, flush } from "solid-js";
import { agentDraftKey, createAgentDraftController } from "./drafts";

function memoryKv(initial: ReadonlyMap<string, KvValue> = new Map()) {
  const values = new Map(initial);
  const keyId = (key: KvKey) => JSON.stringify(key);
  let version = 0;
  const kv: Kv = {
    async get<T extends KvValue>(key: KvKey) {
      const value = values.get(keyId(key));
      return value === undefined
        ? null
        : ({ key, value, versionstamp: String(version) } as KvEntry<T>);
    },
    async set(key, value) {
      values.set(keyId(key), value);
      return String(++version);
    },
    async mergePatch() {
      throw new Error("not used");
    },
    async delete(key) {
      values.delete(keyId(key));
      return String(++version);
    },
    async *list<T extends KvValue>(options: KvListOptions = {}) {
      const prefix = JSON.stringify(options.prefix ?? []).slice(0, -1);
      for (const [encoded, value] of values) {
        if (!encoded.startsWith(prefix)) continue;
        yield {
          key: JSON.parse(encoded) as KvKey,
          value: value as T,
          versionstamp: String(version),
        };
      }
    },
    atomic: () => {
      throw new Error("not used");
    },
  };
  return { kv, values, keyId };
}

function nextTask() {
  return new Promise<void>((resolve) => setTimeout(resolve, 0));
}

describe("Pi Agent conversation drafts", () => {
  test("keeps independent drafts for projects and restored sessions", async () => {
    const storage = memoryKv();
    const [agentId, setAgentId] = createSignal("agent-1");
    const [sessionId, setSessionId] = createSignal<string>();
    let dispose!: () => void;
    const drafts = createRoot((rootDispose) => {
      dispose = rootDispose;
      return createAgentDraftController({
        kv: storage.kv,
        activeAgentId: agentId,
        activeSessionId: sessionId,
        saveDelayMs: 60_000,
      });
    });

    drafts.setDraft("new task");
    drafts.setImages(["page.png"]);
    setSessionId("session-a");
    flush();
    drafts.setDraft("follow up");
    drafts.setContextFiles(["src/main.rs"]);
    setAgentId("agent-2");
    setSessionId(undefined);
    flush();
    drafts.setDraft("other project");
    flush();

    expect(drafts.draft()).toBe("other project");
    setAgentId("agent-1");
    flush();
    expect(drafts.draft()).toBe("new task");
    expect(drafts.images()).toEqual(["page.png"]);
    setSessionId("session-a");
    flush();
    expect(drafts.draft()).toBe("follow up");
    expect(drafts.contextFiles()).toEqual(["src/main.rs"]);

    await drafts.flush();
    dispose();
  });

  test("hydrates a durable draft and preserves a newer local edit", async () => {
    const storedKey = ["draft", 1, "agent-1", "new"] as const;
    const storage = memoryKv(
      new Map([
        [
          JSON.stringify(storedKey),
          {
            version: 1,
            text: "durable text",
            images: ["old.png"],
            contextFiles: [],
          },
        ],
      ]),
    );
    const drafts = createRoot(() =>
      createAgentDraftController({
        kv: storage.kv,
        activeAgentId: () => "agent-1",
        activeSessionId: () => undefined,
        saveDelayMs: 60_000,
      }),
    );

    drafts.setDraft("typed before hydration");
    flush();
    await nextTask();
    expect(drafts.draft()).toBe("typed before hydration");
    await drafts.flush();
    expect(storage.values.get(storage.keyId(storedKey))).toMatchObject({
      text: "typed before hydration",
    });
  });

  test("composes consecutive field updates inside one Solid transaction", () => {
    const storage = memoryKv();
    const drafts = createRoot(() =>
      createAgentDraftController({
        kv: storage.kv,
        activeAgentId: () => "agent-1",
        activeSessionId: () => undefined,
        saveDelayMs: 60_000,
      }),
    );

    drafts.restore("agent-1", undefined, {
      text: "submitted prompt",
      images: ["page.png"],
      contextFiles: ["src/main.rs"],
    });
    drafts.setDraft("");
    drafts.setImages([]);
    drafts.setContextFiles([]);

    expect(drafts.draft()).toBe("");
    expect(drafts.images()).toEqual([]);
    expect(drafts.contextFiles()).toEqual([]);
  });

  test("restores a failed submission and removes every deleted project draft", async () => {
    const storage = memoryKv();
    const drafts = createRoot(() =>
      createAgentDraftController({
        kv: storage.kv,
        activeAgentId: () => "agent-1",
        activeSessionId: () => "session-a",
        saveDelayMs: 60_000,
      }),
    );

    drafts.restore("agent-1", "session-a", {
      text: "retry me",
      images: ["failure.png"],
      contextFiles: ["src/lib.rs"],
    });
    drafts.setDraftFor("agent-1", "session-b", "another session");
    drafts.setDraftFor("agent-2", undefined, "keep me");
    await drafts.flush();
    await drafts.removeAgent("agent-1");

    expect(
      [...storage.values.keys()].some((key) => key.includes("agent-1")),
    ).toBe(false);
    expect(
      [...storage.values.keys()].some((key) => key.includes("agent-2")),
    ).toBe(true);
  });
});

test("uses an explicit and collision-safe in-memory scope key", () => {
  expect(agentDraftKey("agent", "session")).toBe("agent\0session");
  expect(agentDraftKey("agent-session")).not.toBe(
    agentDraftKey("agent", "session"),
  );
});
