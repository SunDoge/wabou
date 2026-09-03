import { describe, expect, test } from "bun:test";
import { createRoot } from "solid-js";
import { defaultHost } from "../renderer/host";
import { createKvSignal, type Kv, type KvEntry, openKv } from "./kv";

function installKv(
  overrides: Partial<{
    get(request: unknown): unknown;
    set(request: unknown): unknown;
    delete(request: unknown): unknown;
    list(request: unknown): unknown;
    atomic(request: unknown): unknown;
  }>,
) {
  const host = defaultHost as unknown as Record<string, unknown>;
  const previous = host.kv;
  host.kv = {
    __wabouCapabilityVersion: 2,
    get: async () => null,
    set: async () => ({ versionstamp: "1" }),
    delete: async () => ({ versionstamp: "1" }),
    list: async () => [],
    atomic: async () => ({ committed: true, versionstamp: "1" }),
    ...overrides,
  };
  return () => {
    if (previous === undefined) delete host.kv;
    else host.kv = previous;
  };
}

describe("SQLite KV facade", () => {
  test("prepends namespaces as key parts without string joining", async () => {
    let request: unknown;
    const restore = installKv({
      set: async (next) => {
        request = next;
        return { versionstamp: "42" };
      },
    });
    try {
      const kv = openKv(["projects", 7]);
      expect(await kv.set(["settings", "theme"], "dark")).toBe("42");
      expect(request).toEqual({
        key: [
          { type: "string", value: "projects" },
          { type: "i64", value: "7" },
          { type: "string", value: "settings" },
          { type: "string", value: "theme" },
        ],
        value: "dark",
      });
    } finally {
      restore();
    }
  });

  test("decodes entries relative to their opened namespace", async () => {
    const restore = installKv({
      get: async () => ({
        key: [
          { type: "string", value: "projects" },
          { type: "string", value: "one" },
          { type: "bytes", value: [1, 2, 3] },
        ],
        value: { ready: true },
        versionstamp: "9",
        expiresAt: null,
      }),
    });
    try {
      const entry = await openKv(["projects", "one"]).get([
        new Uint8Array([1, 2, 3]),
      ]);
      expect(entry?.key).toEqual([new Uint8Array([1, 2, 3])]);
      expect(entry?.value).toEqual({ ready: true });
      expect(entry?.versionstamp).toBe("9");
      expect(entry).not.toHaveProperty("expiresAt");
    } finally {
      restore();
    }
  });

  test("builds one optimistic atomic request", async () => {
    let request: unknown;
    const restore = installKv({
      atomic: async (next) => {
        request = next;
        return { committed: false, versionstamp: null };
      },
    });
    try {
      const result = await openKv(["drafts"])
        .atomic()
        .check({ key: ["one"], versionstamp: "3" })
        .set(["one"], { text: "next" }, { expireIn: 5000 })
        .mergePatch(["one"], { text: null, ready: true })
        .delete(["old"])
        .commit();
      expect(result).toEqual({ committed: false });
      expect(request).toEqual({
        checks: [
          {
            key: [
              { type: "string", value: "drafts" },
              { type: "string", value: "one" },
            ],
            versionstamp: "3",
          },
        ],
        mutations: [
          {
            type: "set",
            key: [
              { type: "string", value: "drafts" },
              { type: "string", value: "one" },
            ],
            value: { text: "next" },
            expireIn: 5000,
          },
          {
            type: "mergePatch",
            key: [
              { type: "string", value: "drafts" },
              { type: "string", value: "one" },
            ],
            patch: { text: null, ready: true },
          },
          {
            type: "delete",
            key: [
              { type: "string", value: "drafts" },
              { type: "string", value: "old" },
            ],
          },
        ],
      });
    } finally {
      restore();
    }
  });

  test("sends merge patches as one unconditional atomic mutation", async () => {
    let request: unknown;
    const restore = installKv({
      atomic: async (next) => {
        request = next;
        return { committed: true, versionstamp: "8" };
      },
    });
    try {
      const version = await openKv(["profiles"]).mergePatch(["one"], {
        settings: { compression: 7 },
        obsolete: null,
      });
      expect(version).toBe("8");
      expect(request).toEqual({
        checks: [],
        mutations: [
          {
            type: "mergePatch",
            key: [
              { type: "string", value: "profiles" },
              { type: "string", value: "one" },
            ],
            patch: {
              settings: { compression: 7 },
              obsolete: null,
            },
          },
        ],
      });
    } finally {
      restore();
    }
  });

  test("rejects unstable numeric keys before crossing the host boundary", async () => {
    const restore = installKv({});
    try {
      await expect(openKv().get([Number.MAX_SAFE_INTEGER + 1])).rejects.toThrow(
        "safe integers",
      );
    } finally {
      restore();
    }
  });

  test("keeps a local signal edit made while durable state is loading", async () => {
    type SettingsEntry = KvEntry<{ theme: string }> | null;
    let resolveLoad!: (value: SettingsEntry) => void;
    const writes: unknown[] = [];
    const load = new Promise<SettingsEntry>((resolve) => {
      resolveLoad = resolve;
    });
    const kv: Kv = {
      get: (() => load) as Kv["get"],
      set: async (_key, value) => {
        writes.push(value);
        return "2";
      },
      mergePatch: async () => {
        throw new Error("not used");
      },
      delete: async () => "3",
      list: async function* () {},
      atomic: () => {
        throw new Error("not used");
      },
    };
    let dispose!: () => void;
    let state!: ReturnType<typeof createKvSignal<{ theme: string }>>;
    createRoot((rootDispose) => {
      dispose = rootDispose;
      state = createKvSignal({
        kv,
        key: ["settings"],
        initial: { theme: "light" },
        saveDelayMs: 60_000,
      });
    });

    state.set({ theme: "local" });
    resolveLoad({
      key: ["settings"],
      value: { theme: "durable" },
      versionstamp: "1",
    });
    await load;
    await Promise.resolve();
    expect(state.value()).toEqual({ theme: "local" });
    expect(state.ready()).toBe(true);
    await state.flush();
    expect(writes).toEqual([{ theme: "local" }]);
    dispose();
  });
});
