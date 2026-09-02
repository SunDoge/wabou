import type { Kv, KvEntry, KvKey, KvValue } from "@wabou/ui";
import { expect, test } from "vitest";
import { createProfileStore } from "../../apps/rustic-gui/ui/profile-store";

function memoryKv(): { kv: Kv; values: Map<string, KvValue> } {
  const values = new Map<string, KvValue>();
  const keyOf = (key: KvKey) => JSON.stringify(key);
  let revision = 0;
  const entry = <T extends KvValue>(key: KvKey, value: T): KvEntry<T> => ({
    key,
    value,
    versionstamp: String(revision),
  });
  const kv: Kv = {
    async get<T extends KvValue>(key: KvKey) {
      const value = values.get(keyOf(key));
      return value === undefined ? null : entry(key, value as T);
    },
    async set(key, value) {
      values.set(keyOf(key), value);
      revision += 1;
      return String(revision);
    },
    async delete(key) {
      values.delete(keyOf(key));
      revision += 1;
      return String(revision);
    },
    async *list<T extends KvValue>(options = {}) {
      const prefix = options.prefix ?? [];
      for (const [encoded, value] of values) {
        const key = JSON.parse(encoded) as KvKey;
        if (prefix.every((part, index) => key[index] === part)) {
          yield entry(key, value as T);
        }
      }
    },
    atomic() {
      const mutations: (() => void)[] = [];
      const operation = {
        check: () => operation,
        set: (key: KvKey, value: KvValue) => {
          mutations.push(() => values.set(keyOf(key), value));
          return operation;
        },
        delete: (key: KvKey) => {
          mutations.push(() => values.delete(keyOf(key)));
          return operation;
        },
        commit: async () => {
          for (const mutation of mutations) mutation();
          revision += 1;
          return { committed: true, versionstamp: String(revision) };
        },
      };
      return operation as ReturnType<Kv["atomic"]>;
    },
  };
  return { kv, values };
}

test("backup profiles persist their source-to-repository aggregate without credentials", async () => {
  const { kv, values } = memoryKv();
  const store = createProfileStore(kv);
  await store.save({
    id: "photos",
    name: "Photos",
    repositoryPath: "/backups/photos",
    sources: ["/home/me/Pictures", "/home/me/Scans"],
  });

  const loaded = await store.load();
  expect(loaded).toEqual({
    profiles: [
      {
        id: "photos",
        name: "Photos",
        repositoryPath: "/backups/photos",
        sources: ["/home/me/Pictures", "/home/me/Scans"],
      },
    ],
    activeProfileId: "photos",
  });
  expect(JSON.stringify([...values.values()])).not.toContain("password");
});
