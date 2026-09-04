import type { Kv, KvValue } from "@wabou/ui";
import type { BackupProfile } from "./api";
import {
  backupScheduleFromValue,
  backupScheduleValue,
} from "./backup-schedule";

const SCHEMA_VERSION = 2;

export interface StoredProfiles {
  profiles: BackupProfile[];
  activeProfileId?: string;
}

export interface ProfileStore {
  load(): Promise<StoredProfiles>;
  save(profile: BackupProfile, options?: { activate?: boolean }): Promise<void>;
  setActive(profileId: string | undefined): Promise<void>;
}

function isRecord(
  value: KvValue,
): value is { readonly [key: string]: KvValue } {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function profileFromValue(value: KvValue): BackupProfile {
  if (!isRecord(value)) throw new TypeError("invalid persisted backup profile");
  const sources = value.sources;
  if (
    typeof value.id !== "string" ||
    typeof value.name !== "string" ||
    typeof value.repositoryPath !== "string" ||
    !Array.isArray(sources) ||
    !sources.every((source: KvValue) => typeof source === "string")
  ) {
    throw new TypeError("invalid persisted backup profile");
  }
  const schedule = backupScheduleFromValue(value.schedule);
  return {
    id: value.id,
    name: value.name,
    repositoryPath: value.repositoryPath,
    sources: sources as string[],
    ...(schedule ? { schedule } : {}),
  };
}

function profileValue(profile: BackupProfile): KvValue {
  return {
    id: profile.id,
    name: profile.name,
    repositoryPath: profile.repositoryPath,
    sources: [...profile.sources],
    ...(profile.schedule
      ? { schedule: backupScheduleValue(profile.schedule) }
      : {}),
  };
}

/** Durable profile metadata. Passwords deliberately remain outside SQLite. */
export function createProfileStore(kv: Kv): ProfileStore {
  return {
    async load() {
      const schema = await kv.get(["meta", "schemaVersion"]);
      if (schema === null) {
        await kv.set(["meta", "schemaVersion"], SCHEMA_VERSION);
      } else if (schema.value !== 1 && schema.value !== SCHEMA_VERSION) {
        throw new Error(
          `unsupported backup profile schema ${String(schema.value)}`,
        );
      }

      const profiles: BackupProfile[] = [];
      for await (const entry of kv.list({ prefix: ["profiles"] })) {
        profiles.push(profileFromValue(entry.value));
      }
      profiles.sort((left, right) => left.name.localeCompare(right.name));

      const active = await kv.get(["state", "activeProfileId"]);
      const activeProfileId =
        typeof active?.value === "string" &&
        profiles.some((profile) => profile.id === active.value)
          ? active.value
          : undefined;
      if (schema?.value === 1) {
        await kv.set(["meta", "schemaVersion"], SCHEMA_VERSION);
      }
      return { profiles, activeProfileId };
    },

    async save(profile, options) {
      const operation = kv
        .atomic()
        .set(["profiles", profile.id], profileValue(profile));
      if (options?.activate !== false) {
        operation.set(["state", "activeProfileId"], profile.id);
      }
      const result = await operation.commit();
      if (!result.committed) throw new Error("could not save backup profile");
    },

    async setActive(profileId) {
      if (profileId === undefined) {
        await kv.delete(["state", "activeProfileId"]);
      } else {
        await kv.set(["state", "activeProfileId"], profileId);
      }
    },
  };
}
