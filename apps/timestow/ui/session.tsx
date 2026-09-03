import { openKv } from "@wabou/ui";
import {
  createContext,
  createEffect,
  createMemo,
  createSignal,
  type JSX,
  useContext,
} from "solid-js";
import { type BackupProfile, type RuntimeStatus, useRusticApi } from "./api";
import { createProfileStore, type ProfileStore } from "./profile-store";

export interface ConnectProfileInput {
  id?: string;
  name: string;
  repositoryPath: string;
  password: string;
  sources?: string[];
}

interface TimestowSession {
  profiles: () => readonly BackupProfile[];
  activeProfile: () => BackupProfile | undefined;
  pendingUnlock: () => BackupProfile | undefined;
  runtime: () => RuntimeStatus;
  loading: () => boolean;
  error: () => string | undefined;
  setError(error: string | undefined): void;
  refresh(): Promise<void>;
  beginCreate(): void;
  activateProfile(profileId: string): Promise<boolean>;
  connectProfile(
    mode: "create" | "open",
    input: ConnectProfileInput,
  ): Promise<BackupProfile>;
  updateSources(profileId: string, sources: string[]): Promise<void>;
}

const SessionContext = createContext<TimestowSession>();

function upsertProfile(
  profiles: readonly BackupProfile[],
  profile: BackupProfile,
): BackupProfile[] {
  const next = profiles.filter((item) => item.id !== profile.id);
  next.push(profile);
  return next.sort((left, right) => left.name.localeCompare(right.name));
}

export function TimestowSessionProvider(props: {
  children?: JSX.Element;
  store?: ProfileStore;
}) {
  const api = useRusticApi();
  const store = props.store ?? createProfileStore(openKv(["timestow"]));
  const [profiles, setProfiles] = createSignal<BackupProfile[]>([]);
  const [activeProfileId, setActiveProfileId] = createSignal<string>();
  const [pendingUnlockId, setPendingUnlockId] = createSignal<string>();
  const [runtime, setRuntime] = createSignal<RuntimeStatus>({
    unlockedProfileIds: [],
  });
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string>();
  const activeProfile = createMemo(() =>
    profiles().find((profile) => profile.id === activeProfileId()),
  );
  const pendingUnlock = createMemo(() =>
    profiles().find((profile) => profile.id === pendingUnlockId()),
  );

  async function refresh(): Promise<void> {
    setLoading(true);
    try {
      const [stored, nextRuntime] = await Promise.all([
        store.load(),
        api.status(),
      ]);
      setProfiles(stored.profiles);
      setRuntime(nextRuntime);
      const selected = nextRuntime.activeProfileId ?? stored.activeProfileId;
      setActiveProfileId(selected);
      if (selected && !nextRuntime.unlockedProfileIds.includes(selected)) {
        setPendingUnlockId(selected);
      }
      setError(undefined);
    } catch (cause) {
      const message = cause instanceof Error ? cause.message : String(cause);
      setError(message);
      throw cause;
    } finally {
      setLoading(false);
    }
  }

  function beginCreate(): void {
    setPendingUnlockId(undefined);
  }

  async function activateProfile(profileId: string): Promise<boolean> {
    const profile = profiles().find((item) => item.id === profileId);
    if (!profile) throw new Error(`backup profile ${profileId} was not found`);
    setActiveProfileId(profileId);
    await store.setActive(profileId);
    if (!runtime().unlockedProfileIds.includes(profileId)) {
      setPendingUnlockId(profileId);
      return false;
    }
    const next = await api.selectProfile({ profileId });
    setRuntime(next);
    setPendingUnlockId(undefined);
    return true;
  }

  async function connectProfile(
    mode: "create" | "open",
    input: ConnectProfileInput,
  ): Promise<BackupProfile> {
    const profile: BackupProfile = {
      id: input.id ?? crypto.randomUUID(),
      name: input.name.trim(),
      repositoryPath: input.repositoryPath.trim(),
      sources: [...(input.sources ?? [])],
    };
    if (!profile.name) throw new Error("backup name is required");
    const request = {
      id: profile.id,
      name: profile.name,
      path: profile.repositoryPath,
      password: input.password,
      sources: profile.sources,
    };
    const nextRuntime = await (mode === "create"
      ? api.createProfile(request)
      : api.openProfile(request));
    await store.save(profile);
    setProfiles((current) => upsertProfile(current, profile));
    setRuntime(nextRuntime);
    setActiveProfileId(profile.id);
    setPendingUnlockId(undefined);
    setError(undefined);
    return profile;
  }

  async function updateSources(
    profileId: string,
    sources: string[],
  ): Promise<void> {
    const profile = profiles().find((item) => item.id === profileId);
    if (!profile) throw new Error(`backup profile ${profileId} was not found`);
    const nextProfile = { ...profile, sources: [...sources] };
    const nextRuntime = await api.setSources({ profileId, sources });
    await store.save(nextProfile);
    setProfiles((current) => upsertProfile(current, nextProfile));
    setRuntime(nextRuntime);
  }

  createEffect(
    () => true,
    () => void refresh().catch(() => undefined),
  );

  return (
    <SessionContext
      value={{
        profiles,
        activeProfile,
        pendingUnlock,
        runtime,
        loading,
        error,
        setError,
        refresh,
        beginCreate,
        activateProfile,
        connectProfile,
        updateSources,
      }}
    >
      {props.children}
    </SessionContext>
  );
}

export function useTimestowSession(): TimestowSession {
  const session = useContext(SessionContext);
  if (!session) throw new Error("TimestowSessionProvider is missing");
  return session;
}
