import type { Kv, KvKey, KvValue } from "@wabou/core";
import { type Accessor, createSignal, getOwner, onCleanup } from "solid-js";

const DRAFT_SCHEMA_VERSION = 1 as const;
const NEW_SESSION = "new";

export type AgentDraftValue = {
  readonly version: typeof DRAFT_SCHEMA_VERSION;
  readonly text: string;
  readonly images: readonly string[];
  readonly contextFiles: readonly string[];
};

interface DraftEntry {
  readonly agentId: string;
  readonly key: KvKey;
  generation: number;
  loadStarted: boolean;
  ready: boolean;
  error: unknown;
  pending?: AgentDraftValue;
  timer?: ReturnType<typeof setTimeout>;
  writer?: Promise<void>;
}

export interface AgentDraftController {
  readonly draft: Accessor<string>;
  readonly images: Accessor<readonly string[]>;
  readonly contextFiles: Accessor<readonly string[]>;
  readonly ready: Accessor<boolean>;
  readonly error: Accessor<unknown>;
  setDraft(value: string): void;
  setImages(paths: readonly string[]): void;
  setContextFiles(paths: readonly string[]): void;
  setDraftFor(
    agentId: string,
    sessionId: string | undefined,
    value: string,
  ): void;
  restore(
    agentId: string,
    sessionId: string | undefined,
    value: Omit<AgentDraftValue, "version">,
  ): void;
  removeAgent(agentId: string): Promise<void>;
  flush(): Promise<void>;
}

export function agentDraftKey(agentId: string, sessionId?: string): string {
  return `${agentId}\0${sessionId || NEW_SESSION}`;
}

function emptyDraft(): AgentDraftValue {
  return {
    version: DRAFT_SCHEMA_VERSION,
    text: "",
    images: [],
    contextFiles: [],
  };
}

function decodeDraft(value: KvValue): AgentDraftValue | undefined {
  if (value === null || Array.isArray(value) || typeof value !== "object") {
    return undefined;
  }
  const draft = value as Readonly<Record<string, KvValue>>;
  if (
    draft.version !== DRAFT_SCHEMA_VERSION ||
    typeof draft.text !== "string" ||
    !Array.isArray(draft.images) ||
    !draft.images.every((path) => typeof path === "string") ||
    !Array.isArray(draft.contextFiles) ||
    !draft.contextFiles.every((path) => typeof path === "string")
  ) {
    return undefined;
  }
  return {
    version: DRAFT_SCHEMA_VERSION,
    text: draft.text,
    images: draft.images as readonly string[],
    contextFiles: draft.contextFiles as readonly string[],
  };
}

/**
 * Persist conversation drafts under one SQLite KV entry per project/session.
 *
 * The visible state is owned by the App root so Solid can propagate writes to
 * native controls. Durable reads and writes remain independent per scope: a
 * late read can never overwrite a newer local edit or another conversation.
 */
export function createAgentDraftController(options: {
  kv: Kv;
  activeAgentId: Accessor<string>;
  activeSessionId: Accessor<string | undefined>;
  saveDelayMs?: number;
}): AgentDraftController {
  let currentValues: Readonly<Record<string, AgentDraftValue>> = {};
  const [values, setValues] = createSignal<
    Readonly<Record<string, AgentDraftValue>>
  >(currentValues, { ownedWrite: true });
  const [statusRevision, setStatusRevision] = createSignal(0, {
    ownedWrite: true,
  });
  const entries = new Map<string, DraftEntry>();
  let disposed = false;

  const touchStatus = () => setStatusRevision((revision) => revision + 1);
  const flushEntry = (entry: DraftEntry): Promise<void> => {
    if (entry.timer !== undefined) clearTimeout(entry.timer);
    entry.timer = undefined;
    if (entry.writer) return entry.writer;
    entry.writer = (async () => {
      while (entry.pending !== undefined) {
        const next = entry.pending;
        entry.pending = undefined;
        try {
          await options.kv.set(entry.key, next);
          entry.error = undefined;
        } catch (error) {
          if (entry.pending === undefined) entry.pending = next;
          entry.error = error;
          throw error;
        } finally {
          if (!disposed) touchStatus();
        }
      }
    })().finally(() => {
      entry.writer = undefined;
    });
    return entry.writer;
  };

  const hydrate = async (cacheKey: string, entry: DraftEntry) => {
    const startedAt = entry.generation;
    try {
      const stored = await options.kv.get(entry.key);
      const draft = stored ? decodeDraft(stored.value) : undefined;
      if (!disposed && entry.generation === startedAt && draft) {
        currentValues = { ...currentValues, [cacheKey]: draft };
        setValues(currentValues);
      }
      entry.error = undefined;
    } catch (error) {
      entry.error = error;
    } finally {
      entry.ready = true;
      if (!disposed) touchStatus();
    }
  };

  const entryFor = (agentId: string, sessionId?: string): DraftEntry => {
    const cacheKey = agentDraftKey(agentId, sessionId);
    let entry = entries.get(cacheKey);
    if (!entry) {
      entry = {
        agentId,
        key: ["draft", DRAFT_SCHEMA_VERSION, agentId, sessionId || NEW_SESSION],
        generation: 0,
        loadStarted: false,
        ready: false,
        error: undefined,
      };
      entries.set(cacheKey, entry);
    }
    if (!entry.loadStarted) {
      entry.loadStarted = true;
      void hydrate(cacheKey, entry);
    }
    return entry;
  };

  const valueFor = (agentId: string, sessionId?: string) => {
    entryFor(agentId, sessionId);
    // Subscribe callers to publication while returning the controller's
    // synchronous snapshot. Solid may defer the observable signal value until
    // the surrounding flush completes, but native event code must observe its
    // own writes immediately.
    values();
    return currentValues[agentDraftKey(agentId, sessionId)] ?? emptyDraft();
  };
  const activeValue = () =>
    valueFor(options.activeAgentId(), options.activeSessionId());
  const activeEntry = () =>
    entryFor(options.activeAgentId(), options.activeSessionId());

  const update = (
    agentId: string,
    sessionId: string | undefined,
    change: (current: AgentDraftValue) => AgentDraftValue,
  ) => {
    const cacheKey = agentDraftKey(agentId, sessionId);
    const entry = entryFor(agentId, sessionId);
    // Solid 2 batches writes inside an event transaction, so a signal read
    // immediately after setValues can still expose the previous snapshot.
    // Keep the controller's authoritative value synchronous: consecutive
    // field updates (clear text, images, then context) must compose instead of
    // restoring fields from a stale reactive read.
    const next = change(currentValues[cacheKey] ?? emptyDraft());
    entry.generation += 1;
    entry.pending = next;
    currentValues = { ...currentValues, [cacheKey]: next };
    setValues(currentValues);
    if (entry.timer !== undefined) clearTimeout(entry.timer);
    entry.timer = setTimeout(
      () => void flushEntry(entry).catch(() => {}),
      options.saveDelayMs ?? 150,
    );
  };

  const setDraftFor = (
    agentId: string,
    sessionId: string | undefined,
    text: string,
  ) => update(agentId, sessionId, (current) => ({ ...current, text }));

  const restore = (
    agentId: string,
    sessionId: string | undefined,
    value: Omit<AgentDraftValue, "version">,
  ) =>
    update(agentId, sessionId, () => ({
      version: DRAFT_SCHEMA_VERSION,
      text: value.text,
      images: [...value.images],
      contextFiles: [...value.contextFiles],
    }));

  const flush = async () => {
    await Promise.all([...entries.values()].map(flushEntry));
  };

  const removeAgent = async (agentId: string) => {
    const cached = [...entries.entries()].filter(
      ([, entry]) => entry.agentId === agentId,
    );
    await Promise.all(cached.map(([, entry]) => flushEntry(entry)));
    for (const [key, entry] of cached) {
      if (entry.timer !== undefined) clearTimeout(entry.timer);
      entries.delete(key);
    }
    currentValues = Object.fromEntries(
      Object.entries(currentValues).filter(
        ([key]) => !key.startsWith(`${agentId}\0`),
      ),
    );
    setValues(currentValues);

    const durableKeys: KvKey[] = [];
    for await (const entry of options.kv.list({
      prefix: ["draft", DRAFT_SCHEMA_VERSION, agentId],
    })) {
      durableKeys.push(entry.key);
    }
    await Promise.all(durableKeys.map((key) => options.kv.delete(key)));
  };

  if (getOwner()) {
    onCleanup(() => {
      disposed = true;
      for (const entry of entries.values()) {
        if (entry.timer !== undefined) clearTimeout(entry.timer);
        void flushEntry(entry).catch(() => {});
      }
    });
  }

  return {
    draft: () => activeValue().text,
    images: () => activeValue().images,
    contextFiles: () => activeValue().contextFiles,
    ready: () => {
      statusRevision();
      return activeEntry().ready;
    },
    error: () => {
      statusRevision();
      return activeEntry().error;
    },
    setDraft: (text) =>
      setDraftFor(options.activeAgentId(), options.activeSessionId(), text),
    setImages: (images) =>
      update(options.activeAgentId(), options.activeSessionId(), (current) => ({
        ...current,
        images: [...images],
      })),
    setContextFiles: (contextFiles) =>
      update(options.activeAgentId(), options.activeSessionId(), (current) => ({
        ...current,
        contextFiles: [...contextFiles],
      })),
    setDraftFor,
    restore,
    removeAgent,
    flush,
  };
}
