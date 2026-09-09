import type { Kv, KvEntry, KvValue } from "@wabou/ui";
import type { BackupProfile } from "./api";
import {
  backupScheduleFromValue,
  backupScheduleValue,
} from "./backup-schedule";

const SCHEMA_VERSION = 3;

export interface StoredProfiles {
  profiles: BackupProfile[];
  activeProfileId?: string;
  recoveryNotice?: string;
}

export interface ProfileStore {
  load(): Promise<StoredProfiles>;
  save(profile: BackupProfile, options?: { activate?: boolean }): Promise<void>;
  setActive(profileId: string | undefined): Promise<void>;
  remove(profileId: string): Promise<void>;
}

function isRecord(
  value: KvValue,
): value is { readonly [key: string]: KvValue } {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function profileFromValue(value: KvValue, expectedId: string): BackupProfile {
  if (!isRecord(value)) throw new TypeError("invalid persisted backup profile");
  const sources = value.sources;
  if (
    typeof value.id !== "string" ||
    value.id !== expectedId ||
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
    repositoryKind:
      value.repositoryKind === "rusticConfig" ? "rusticConfig" : "local",
    sources: sources as string[],
    ...(schedule ? { schedule } : {}),
  };
}

async function quarantineProfile(
  kv: Kv,
  entry: KvEntry<KvValue>,
  profileId: string,
): Promise<void> {
  const recoveryKey = [
    "recovery",
    "profiles",
    profileId,
    entry.versionstamp ?? "unversioned",
  ] as const;
  const result = await kv
    .atomic()
    .check(entry)
    .set(recoveryKey, {
      value: entry.value,
      quarantinedAt: new Date().toISOString(),
    })
    .delete(entry.key)
    .commit();
  if (!result.committed) {
    throw new Error(
      `damaged backup profile ${profileId} changed while Timestow was recovering it`,
    );
  }
}

function profileValue(profile: BackupProfile): KvValue {
  return {
    id: profile.id,
    name: profile.name,
    repositoryPath: profile.repositoryPath,
    repositoryKind: profile.repositoryKind ?? "local",
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
      } else if (
        schema.value !== 1 &&
        schema.value !== 2 &&
        schema.value !== SCHEMA_VERSION
      ) {
        throw new Error(
          `unsupported backup profile schema ${String(schema.value)}`,
        );
      }

      const profiles: BackupProfile[] = [];
      const quarantinedProfileIds: string[] = [];
      for await (const entry of kv.list({ prefix: ["profiles"] })) {
        const keyId = entry.key.length === 2 ? entry.key[1] : undefined;
        const profileId =
          typeof keyId === "string"
            ? keyId
            : `invalid-profile-${quarantinedProfileIds.length + 1}`;
        try {
          profiles.push(profileFromValue(entry.value, profileId));
        } catch {
          await quarantineProfile(kv, entry, profileId);
          quarantinedProfileIds.push(profileId);
        }
      }
      profiles.sort((left, right) => left.name.localeCompare(right.name));

      const active = await kv.get(["state", "activeProfileId"]);
      const activeProfileId =
        typeof active?.value === "string" &&
        profiles.some((profile) => profile.id === active.value)
          ? active.value
          : undefined;
      if (active !== null && activeProfileId === undefined) {
        await kv.delete(["state", "activeProfileId"]);
      }
      if (schema?.value === 1 || schema?.value === 2) {
        await kv.set(["meta", "schemaVersion"], SCHEMA_VERSION);
      }
      return {
        profiles,
        activeProfileId,
        ...(quarantinedProfileIds.length > 0
          ? {
              recoveryNotice: `Timestow isolated damaged metadata for ${quarantinedProfileIds.join(", ")} and kept the original data in recovery storage. Other backups are still available.`,
            }
          : {}),
      };
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

    async remove(profileId) {
      const profileKey = ["profiles", profileId] as const;
      const activeKey = ["state", "activeProfileId"] as const;
      const [profile, active] = await Promise.all([
        kv.get(profileKey),
        kv.get(activeKey),
      ]);
      const operation = kv
        .atomic()
        .check(profile ?? { key: profileKey, versionstamp: null })
        .check(active ?? { key: activeKey, versionstamp: null })
        .delete(profileKey);
      if (active?.value === profileId) operation.delete(activeKey);
      const result = await operation.commit();
      if (!result.committed) {
        throw new Error("backup profile changed while it was being removed");
      }
    },
  };
}
