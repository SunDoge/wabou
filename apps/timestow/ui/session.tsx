import { openKv, subscribeJsonHostMessages } from "@wabou/ui";
import {
  createContext,
  createEffect,
  createMemo,
  createSignal,
  type JSX,
  onCleanup,
  untrack,
  useContext,
} from "solid-js";
import {
  type BackupProfile,
  type RuntimeStatus,
  type SnapshotEntry,
  useRusticApi,
} from "./api";
import {
  OPERATION_PROGRESS_TOPIC,
  type OperationProgressEvent,
  decodeOperationProgressEvent,
} from "./operation-progress";
import {
  advanceBackupSchedule,
  type BackupSchedule,
  type BackupScheduleInterval,
  createBackupSchedule,
  scheduleDueAt,
} from "./backup-schedule";
import { createProfileStore, type ProfileStore } from "./profile-store";
import {
  createSnapshotBrowserCache,
  type SnapshotBrowserCache,
} from "./snapshot-browser-cache";

export interface ConnectProfileInput {
  id?: string;
  name: string;
  repositoryPath: string;
  passwordSlot: string;
  confirmationSlot?: string;
  sources?: string[];
}

interface TimestowSession {
  profiles: () => readonly BackupProfile[];
  activeProfile: () => BackupProfile | undefined;
  pendingUnlock: () => BackupProfile | undefined;
  runtime: () => RuntimeStatus;
  loading: () => boolean;
  error: () => string | undefined;
  lastBackup: () =>
    | { profileId: string; snapshot: SnapshotEntry; scheduled: boolean }
    | undefined;
  isBackingUp(profileId: string): boolean;
  hasActiveOperations(): boolean;
  beginOperation(profileId: string, operationId: string): void;
  endOperation(profileId: string, operationId: string): void;
  backupProgress(profileId: string): OperationProgressEvent | undefined;
  snapshotBrowser: SnapshotBrowserCache;
  setError(error: string | undefined): void;
  refresh(): Promise<void>;
  beginCreate(): void;
  activateProfile(profileId: string): Promise<boolean>;
  forgetProfile(profileId: string): Promise<void>;
  renameProfile(profileId: string, name: string): Promise<void>;
  connectProfile(
    mode: "create" | "open",
    input: ConnectProfileInput,
  ): Promise<BackupProfile>;
  updateSources(profileId: string, sources: string[]): Promise<void>;
  updateSchedule(
    profileId: string,
    enabled: boolean,
    intervalMinutes: BackupScheduleInterval,
  ): Promise<void>;
  runBackup(profileId: string, scheduled?: boolean): Promise<SnapshotEntry>;
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

function assertUniqueProfileName(
  profiles: readonly BackupProfile[],
  name: string,
  excludedId?: string,
): void {
  if (
    profiles.some(
      (profile) =>
        profile.id !== excludedId &&
        profile.name.toLocaleLowerCase() === name.toLocaleLowerCase(),
    )
  ) {
    throw new Error(`a backup named ${name} already exists`);
  }
}

function operationBelongsToProfile(
  operationKey: string,
  profileId: string,
): boolean {
  return operationKey.startsWith(`${profileId}\u0000`);
}

function hasProfileOperation(
  operationKeys: ReadonlySet<string>,
  profileId: string,
): boolean {
  for (const key of operationKeys) {
    if (operationBelongsToProfile(key, profileId)) return true;
  }
  return false;
}

export function TimestowSessionProvider(props: {
  children?: JSX.Element;
  store?: ProfileStore;
}) {
  const api = useRusticApi();
  const store = props.store ?? createProfileStore(openKv(["timestow"]));
  const snapshotBrowser = createSnapshotBrowserCache();
  const [profiles, setProfiles] = createSignal<BackupProfile[]>([]);
  const [activeProfileId, setActiveProfileId] = createSignal<string>();
  const [pendingUnlockId, setPendingUnlockId] = createSignal<string>();
  const [runtime, setRuntime] = createSignal<RuntimeStatus>({
    unlockedProfileIds: [],
  });
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string>();
  const [lastBackup, setLastBackup] = createSignal<{
    profileId: string;
    snapshot: SnapshotEntry;
    scheduled: boolean;
  }>();
  const [runningBackupIds, setRunningBackupIds] = createSignal<Set<string>>(
    new Set(),
  );
  const [backupProgressByProfile, setBackupProgressByProfile] = createSignal<
    Readonly<Record<string, OperationProgressEvent>>
  >({});
  const [activeOperationKeys, setActiveOperationKeys] = createSignal<
    ReadonlySet<string>
  >(new Set());
  const [locallyPendingOperationIds, setLocallyPendingOperationIds] =
    createSignal<ReadonlySet<string>>(new Set());
  const activeProfile = createMemo(() =>
    profiles().find((profile) => profile.id === activeProfileId()),
  );
  const pendingUnlock = createMemo(() =>
    profiles().find((profile) => profile.id === pendingUnlockId()),
  );
  let activeSelectionWrites = Promise.resolve();

  function writeActiveSelection(profileId: string): Promise<void> {
    const write = activeSelectionWrites.then(() => store.setActive(profileId));
    activeSelectionWrites = write.catch(() => undefined);
    return write;
  }
  const profileRemovalOperations = new Map<string, Promise<void>>();
  const profileMutationTails = new Map<string, Promise<void>>();

  function assertProfileAvailable(profileId: string): void {
    if (!profileRemovalOperations.has(profileId)) return;
    const profile = profiles().find((item) => item.id === profileId);
    throw new Error(
      `${profile?.name ?? "This backup"} is being forgotten; wait for that operation to finish`,
    );
  }

  function enqueueProfileMutation<T>(
    profileId: string,
    mutation: () => Promise<T>,
  ): Promise<T> {
    try {
      assertProfileAvailable(profileId);
    } catch (cause) {
      return Promise.reject(cause);
    }
    const previous = profileMutationTails.get(profileId) ?? Promise.resolve();
    const result = previous.then(mutation);
    const tail = result.then(
      () => undefined,
      () => undefined,
    );
    profileMutationTails.set(profileId, tail);
    void tail.then(() => {
      if (profileMutationTails.get(profileId) === tail) {
        profileMutationTails.delete(profileId);
      }
    });
    return result;
  }

  async function refresh(): Promise<void> {
    setLoading(true);
    try {
      const [stored, nextRuntime] = await Promise.all([
        store.load(),
        api.status(),
      ]);
      setProfiles(stored.profiles);
      setRuntime(nextRuntime);
      const selected = stored.activeProfileId;
      setActiveProfileId(selected);
      if (selected && !nextRuntime.unlockedProfileIds.includes(selected)) {
        setPendingUnlockId(selected);
      }
      setError(stored.recoveryNotice);
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
    assertProfileAvailable(profileId);
    const profile = profiles().find((item) => item.id === profileId);
    if (!profile) throw new Error(`backup profile ${profileId} was not found`);
    const profileName = profile.name;

    async function rememberSelection(): Promise<void> {
      try {
        await writeActiveSelection(profileId);
      } catch (cause) {
        if (activeProfileId() !== profileId) return;
        const message = cause instanceof Error ? cause.message : String(cause);
        setError(
          `${profileName} is active, but Timestow could not remember the selection: ${message}`,
        );
      }
    }

    if (!runtime().unlockedProfileIds.includes(profileId)) {
      setActiveProfileId(profileId);
      setPendingUnlockId(profileId);
      void rememberSelection();
      return false;
    }
    setActiveProfileId(profileId);
    setPendingUnlockId(undefined);
    setError(undefined);
    void rememberSelection();
    return true;
  }

  function forgetProfile(profileId: string): Promise<void> {
    const existing = profileRemovalOperations.get(profileId);
    if (existing) return existing;
    const operation = forgetProfileOnce(profileId);
    profileRemovalOperations.set(profileId, operation);
    const clear = () => {
      if (profileRemovalOperations.get(profileId) === operation) {
        profileRemovalOperations.delete(profileId);
      }
    };
    void operation.then(clear, clear);
    return operation;
  }

  async function forgetProfileOnce(profileId: string): Promise<void> {
    const profile = profiles().find((item) => item.id === profileId);
    if (!profile) throw new Error(`backup profile ${profileId} was not found`);
    if (isBackingUp(profileId)) {
      throw new Error(`wait for ${profile.name} to finish backing up`);
    }
    if (
      hasProfileOperation(activeOperationKeys(), profileId) ||
      hasProfileOperation(locallyPendingOperationIds(), profileId)
    ) {
      throw new Error(
        `wait for ${profile.name} to finish its active operation before forgetting it`,
      );
    }
    const pendingMutation = profileMutationTails.get(profileId);
    if (pendingMutation) await pendingMutation;
    const nextRuntime = await api.forgetProfile({ profileId });
    setRuntime(nextRuntime);
    if (activeProfileId() === profileId) setActiveProfileId(undefined);
    if (pendingUnlockId() === profileId) setPendingUnlockId(undefined);
    setBackupProgressByProfile((current) => {
      const remaining = { ...current };
      delete remaining[profileId];
      return remaining;
    });
    try {
      await activeSelectionWrites;
      await store.remove(profileId);
    } catch (cause) {
      const message = cause instanceof Error ? cause.message : String(cause);
      setError(
        `${profile.name} is locked and disconnected, but Timestow could not remove its saved profile: ${message}. It may reappear after restart; choose Forget backup again to retry.`,
      );
      return;
    }
    setProfiles((current) => current.filter((item) => item.id !== profileId));
    setError(undefined);
  }

  async function renameProfile(profileId: string, name: string): Promise<void> {
    const normalized = name.trim();
    if (!normalized) throw new Error("backup name is required");
    assertUniqueProfileName(profiles(), normalized, profileId);
    await enqueueProfileMutation(profileId, async () => {
      const profile = profiles().find((item) => item.id === profileId);
      if (!profile)
        throw new Error(`backup profile ${profileId} was not found`);
      assertUniqueProfileName(profiles(), normalized, profileId);
      await persistProfile({ ...profile, name: normalized });
      setError(undefined);
    });
  }

  async function connectProfile(
    mode: "create" | "open",
    input: ConnectProfileInput,
  ): Promise<BackupProfile> {
    if (input.id) assertProfileAvailable(input.id);
    const existing = input.id
      ? profiles().find((profile) => profile.id === input.id)
      : undefined;
    const profile: BackupProfile = {
      id: input.id ?? crypto.randomUUID(),
      name: input.name.trim(),
      repositoryPath: input.repositoryPath.trim(),
      sources: [...(input.sources ?? [])],
      ...(existing?.schedule ? { schedule: existing.schedule } : {}),
    };
    if (!profile.name) throw new Error("backup name is required");
    assertUniqueProfileName(profiles(), profile.name, profile.id);
    const request = {
      id: profile.id,
      name: profile.name,
      path: profile.repositoryPath,
      passwordSlot: input.passwordSlot,
      sources: profile.sources,
    };
    let nextRuntime: RuntimeStatus;
    if (mode === "create") {
      const confirmationSlot = input.confirmationSlot?.trim();
      if (!confirmationSlot) {
        throw new Error("repository password confirmation is required");
      }
      nextRuntime = await api.createProfile({
        ...request,
        confirmationSlot,
      });
    } else {
      nextRuntime = await api.openProfile(request);
    }
    let persistenceWarning: string | undefined;
    try {
      await store.save(profile);
    } catch (cause) {
      const message = cause instanceof Error ? cause.message : String(cause);
      persistenceWarning = `${profile.name} is connected, but Timestow could not save this backup profile: ${message}. It remains available until the app closes; update its name or folders to retry saving.`;
    }
    setProfiles((current) => upsertProfile(current, profile));
    setRuntime(nextRuntime);
    setActiveProfileId(profile.id);
    setPendingUnlockId(undefined);
    setError(persistenceWarning);
    return profile;
  }

  async function updateSources(
    profileId: string,
    sources: string[],
  ): Promise<void> {
    const nextSources = [...sources];
    await enqueueProfileMutation(profileId, async () => {
      const profile = profiles().find((item) => item.id === profileId);
      if (!profile)
        throw new Error(`backup profile ${profileId} was not found`);
      const nextProfile = { ...profile, sources: nextSources };
      const nextRuntime = await api.setSources({
        profileId,
        sources: nextSources,
      });
      let persistenceWarning: string | undefined;
      try {
        await store.save(nextProfile, { activate: false });
      } catch (cause) {
        const message = cause instanceof Error ? cause.message : String(cause);
        persistenceWarning = `${profile.name} now uses the updated folders, but Timestow could not save them: ${message}. They remain active until the app closes; change the folders again to retry saving.`;
      }
      setProfiles((current) => upsertProfile(current, nextProfile));
      setRuntime(nextRuntime);
      setError(persistenceWarning);
    });
  }

  async function persistProfile(profile: BackupProfile): Promise<void> {
    await store.save(profile, { activate: false });
    setProfiles((current) => upsertProfile(current, profile));
  }

  async function updateSchedule(
    profileId: string,
    enabled: boolean,
    intervalMinutes: BackupScheduleInterval,
  ): Promise<void> {
    await enqueueProfileMutation(profileId, async () => {
      const profile = profiles().find((item) => item.id === profileId);
      if (!profile)
        throw new Error(`backup profile ${profileId} was not found`);
      await persistProfile({
        ...profile,
        schedule: createBackupSchedule(
          enabled,
          intervalMinutes,
          profile.schedule,
        ),
      });
    });
  }

  function isBackingUp(profileId: string): boolean {
    return runningBackupIds().has(profileId);
  }

  function hasActiveOperations(): boolean {
    return (
      runningBackupIds().size > 0 ||
      activeOperationKeys().size > 0 ||
      locallyPendingOperationIds().size > 0
    );
  }

  function beginOperation(profileId: string, operationId: string): void {
    assertProfileAvailable(profileId);
    setLocallyPendingOperationIds((current) => {
      const next = new Set(current);
      next.add(`${profileId}\u0000${operationId}`);
      return next;
    });
  }

  function endOperation(profileId: string, operationId: string): void {
    setLocallyPendingOperationIds((current) => {
      const next = new Set(current);
      next.delete(`${profileId}\u0000${operationId}`);
      return next;
    });
  }

  function backupProgress(
    profileId: string,
  ): OperationProgressEvent | undefined {
    return backupProgressByProfile()[profileId];
  }

  function setBackingUp(profileId: string, running: boolean): void {
    setRunningBackupIds((current) => {
      const next = new Set(current);
      if (running) next.add(profileId);
      else next.delete(profileId);
      return next;
    });
  }

  async function recordScheduleResult(
    profileId: string,
    schedule: BackupSchedule | undefined,
    error?: string,
  ): Promise<void> {
    if (!schedule) return;
    await enqueueProfileMutation(profileId, async () => {
      const current = profiles().find((profile) => profile.id === profileId);
      if (!current?.schedule) return;
      await persistProfile({
        ...current,
        schedule: advanceBackupSchedule(current.schedule, {
          completedAt: Date.now(),
          error,
        }),
      });
    });
  }

  async function runBackup(
    profileId: string,
    scheduled = false,
  ): Promise<SnapshotEntry> {
    assertProfileAvailable(profileId);
    const profile = profiles().find((item) => item.id === profileId);
    if (!profile) throw new Error(`backup profile ${profileId} was not found`);
    if (!runtime().unlockedProfileIds.includes(profileId)) {
      throw new Error(`unlock ${profile.name} before backing it up`);
    }
    if (profile.sources.length === 0) {
      throw new Error("add at least one backup folder first");
    }
    if (isBackingUp(profileId)) {
      throw new Error(`${profile.name} is already being backed up`);
    }
    setBackingUp(profileId, true);
    setBackupProgressByProfile((current) => ({
      ...current,
      [profileId]: {
        profileId,
        operation: "backup",
        operationId: `backup:${profileId}`,
        state: "running",
        unit: "spinner",
        title: "Preparing backup",
        current: 0,
      },
    }));
    let result: { snapshot: SnapshotEntry };
    try {
      result = await api.runBackup({ profileId });
    } catch (cause) {
      const message = cause instanceof Error ? cause.message : String(cause);
      await recordScheduleResult(profileId, profile.schedule, message).catch(
        () => undefined,
      );
      setBackingUp(profileId, false);
      throw cause;
    }
    try {
      await recordScheduleResult(profileId, profile.schedule);
      setError(undefined);
    } catch (cause) {
      const message = cause instanceof Error ? cause.message : String(cause);
      setError(
        `Backup completed, but its schedule status was not saved: ${message}`,
      );
    }
    setLastBackup({ profileId, snapshot: result.snapshot, scheduled });
    setBackingUp(profileId, false);
    return result.snapshot;
  }

  let scheduleTimer: ReturnType<typeof setTimeout> | undefined;
  let scheduleBatchRunning = false;

  const unsubscribeBackupProgress =
    subscribeJsonHostMessages<OperationProgressEvent>(
      OPERATION_PROGRESS_TOPIC,
      (progress) => {
        const operationKey = `${progress.profileId}\u0000${progress.operation}\u0000${progress.operationId}`;
        setActiveOperationKeys((current) => {
          const next = new Set(current);
          if (progress.state === "completed" || progress.state === "failed") {
            next.delete(operationKey);
          } else {
            next.add(operationKey);
          }
          return next;
        });
        if (progress.operation !== "backup") return;
        setBackupProgressByProfile((current) => ({
          ...current,
          [progress.profileId]: progress,
        }));
      },
      {
        decode: decodeOperationProgressEvent,
        onError: (cause) =>
          console.error("[timestow] invalid operation progress", cause),
      },
    );

  function clearScheduleTimer(): void {
    if (scheduleTimer === undefined) return;
    clearTimeout(scheduleTimer);
    scheduleTimer = undefined;
  }

  async function runDueBackups(): Promise<void> {
    if (scheduleBatchRunning) return;
    scheduleBatchRunning = true;
    scheduleTimer = undefined;
    try {
      const now = Date.now();
      const snapshot = untrack(() => ({
        profiles: profiles(),
        unlocked: runtime().unlockedProfileIds,
      }));
      const due = snapshot.profiles.filter((profile) => {
        const timestamp = profile.schedule
          ? scheduleDueAt(profile.schedule)
          : undefined;
        return (
          timestamp !== undefined &&
          timestamp <= now &&
          snapshot.unlocked.includes(profile.id)
        );
      });
      const failures: Array<{ profile: BackupProfile; message: string }> = [];
      for (const profile of due) {
        try {
          await runBackup(profile.id, true);
        } catch (cause) {
          const message =
            cause instanceof Error ? cause.message : String(cause);
          failures.push({ profile, message });
        }
      }
      if (failures.length === 1) {
        const [failure] = failures;
        setError(
          `Automatic backup for ${failure.profile.name} failed: ${failure.message}. Open this backup and try again.`,
        );
      } else if (failures.length > 1) {
        setError(
          `Automatic backups failed: ${failures
            .map(({ profile, message }) => `${profile.name} — ${message}`)
            .join("; ")}. Open each backup and try again.`,
        );
      }
    } finally {
      scheduleBatchRunning = false;
      const latest = untrack(() => ({
        profiles: profiles(),
        unlockedProfileIds: runtime().unlockedProfileIds,
      }));
      scheduleNextBackup(latest.profiles, latest.unlockedProfileIds);
    }
  }

  function scheduleNextBackup(
    scheduledProfiles: readonly BackupProfile[],
    unlockedProfileIds: readonly string[],
  ): void {
    clearScheduleTimer();
    if (scheduleBatchRunning) return;
    const dueTimes = scheduledProfiles.flatMap((profile) => {
      if (!unlockedProfileIds.includes(profile.id) || !profile.schedule) {
        return [];
      }
      const dueAt = scheduleDueAt(profile.schedule);
      return dueAt === undefined ? [] : [dueAt];
    });
    if (dueTimes.length === 0) return;
    const delay = Math.max(0, Math.min(...dueTimes) - Date.now());
    scheduleTimer = setTimeout(() => void runDueBackups(), delay);
  }

  createEffect(
    () => true,
    () => void refresh().catch(() => undefined),
  );

  createEffect(
    () => ({
      profiles: profiles(),
      unlockedProfileIds: runtime().unlockedProfileIds,
    }),
    ({ profiles: scheduledProfiles, unlockedProfileIds }) =>
      scheduleNextBackup(scheduledProfiles, unlockedProfileIds),
  );
  onCleanup(() => {
    clearScheduleTimer();
    unsubscribeBackupProgress();
  });

  return (
    <SessionContext
      value={{
        profiles,
        activeProfile,
        pendingUnlock,
        runtime,
        loading,
        error,
        lastBackup,
        isBackingUp,
        hasActiveOperations,
        beginOperation,
        endOperation,
        backupProgress,
        snapshotBrowser,
        setError,
        refresh,
        beginCreate,
        activateProfile,
        forgetProfile,
        renameProfile,
        connectProfile,
        updateSources,
        updateSchedule,
        runBackup,
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
