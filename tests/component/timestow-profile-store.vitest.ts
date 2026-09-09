import type { Kv, KvEntry, KvKey, KvValue } from "@wabou/ui";
import { expect, test } from "vitest";
import { createProfileStore } from "../../apps/timestow/ui/profile-store";

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
    async mergePatch() {
      throw new Error("not used");
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
        repositoryKind: "local",
        sources: ["/home/me/Pictures", "/home/me/Scans"],
      },
    ],
    activeProfileId: "photos",
  });
  expect(JSON.stringify([...values.values()])).not.toContain("password");
});

test("backup profiles persist automatic schedule state", async () => {
  const { kv, values } = memoryKv();
  const store = createProfileStore(kv);
  await store.save({
    id: "documents",
    name: "Documents",
    repositoryPath: "/backups/documents",
    sources: ["/home/me/Documents"],
    schedule: {
      enabled: true,
      intervalMinutes: 1440,
      nextRunAt: "2026-09-05T08:00:00.000Z",
      lastRunAt: "2026-09-04T08:00:00.000Z",
    },
  });

  expect((await store.load()).profiles[0]?.schedule).toEqual({
    enabled: true,
    intervalMinutes: 1440,
    nextRunAt: "2026-09-05T08:00:00.000Z",
    lastRunAt: "2026-09-04T08:00:00.000Z",
    lastError: undefined,
  });
  expect(JSON.stringify([...values.values()])).not.toContain("password");
});

test("schedule updates do not steal the active profile", async () => {
  const { kv } = memoryKv();
  const store = createProfileStore(kv);
  await store.save({
    id: "photos",
    name: "Photos",
    repositoryPath: "/backups/photos",
    sources: ["/photos"],
  });
  await store.save(
    {
      id: "documents",
      name: "Documents",
      repositoryPath: "/backups/documents",
      sources: ["/documents"],
    },
    { activate: false },
  );

  expect((await store.load()).activeProfileId).toBe("photos");
});

test("forgetting a profile clears its active selection without touching other profiles", async () => {
  const { kv } = memoryKv();
  const store = createProfileStore(kv);
  await store.save({
    id: "photos",
    name: "Photos",
    repositoryPath: "/backups/photos",
    sources: ["/photos"],
  });
  await store.save(
    {
      id: "documents",
      name: "Documents",
      repositoryPath: "/backups/documents",
      sources: ["/documents"],
    },
    { activate: false },
  );

  await store.remove("photos");

  expect(await store.load()).toEqual({
    profiles: [
      {
        id: "documents",
        name: "Documents",
        repositoryPath: "/backups/documents",
        repositoryKind: "local",
        sources: ["/documents"],
      },
    ],
    activeProfileId: undefined,
  });
  await expect(store.remove("missing")).resolves.toBeUndefined();
});

test("rustic configuration profiles persist only their config path and backend kind", async () => {
  const { kv, values } = memoryKv();
  const store = createProfileStore(kv);
  await store.save({
    id: "s3-archive",
    name: "S3 archive",
    repositoryPath: "/home/me/.config/rustic/rustic.toml",
    repositoryKind: "rusticConfig",
    sources: [],
  });

  expect((await store.load()).profiles[0]).toMatchObject({
    repositoryKind: "rusticConfig",
    repositoryPath: "/home/me/.config/rustic/rustic.toml",
  });
  expect(JSON.stringify([...values.values()])).not.toContain(
    "secret_access_key",
  );
  expect(JSON.stringify([...values.values()])).not.toContain("password");
});

test("damaged profile metadata is quarantined without hiding healthy backups", async () => {
  const { kv, values } = memoryKv();
  const store = createProfileStore(kv);
  await store.save({
    id: "photos",
    name: "Photos",
    repositoryPath: "/backups/photos",
    sources: ["/photos"],
  });
  values.set(JSON.stringify(["profiles", "damaged"]), {
    id: "another-id",
    name: "Damaged",
    repositoryPath: "/backups/damaged",
    sources: ["/damaged"],
  });
  values.set(JSON.stringify(["state", "activeProfileId"]), "damaged");

  const loaded = await store.load();

  expect(loaded.profiles.map(({ id }) => id)).toEqual(["photos"]);
  expect(loaded.activeProfileId).toBeUndefined();
  expect(loaded.recoveryNotice).toContain("damaged");
  expect(values.has(JSON.stringify(["profiles", "damaged"]))).toBe(false);
  expect(values.has(JSON.stringify(["state", "activeProfileId"]))).toBe(false);
  const recovery = [...values.entries()].find(([key]) =>
    key.startsWith('["recovery","profiles","damaged",'),
  );
  expect(recovery?.[1]).toMatchObject({
    value: {
      id: "another-id",
      name: "Damaged",
    },
  });
});
