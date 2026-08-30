import { match, P } from "ts-pattern";

export type AgentConnection = "stopped" | "ready" | "running" | "failed";

/** Reconcile a process liveness snapshot without erasing newer RPC activity. */
export function reconcileProcessConnection(
  current: AgentConnection,
  processRunning: boolean,
): AgentConnection {
  if (!processRunning) return "stopped";
  return current === "running" || current === "failed" ? current : "ready";
}
export type AgentQueueMode = "all" | "one-at-a-time";
export type AgentActivity =
  | { kind: "responding" }
  | { kind: "compacting"; reason?: string }
  | {
      kind: "retrying";
      attempt?: number;
      maxAttempts?: number;
      delayMs?: number;
    }
  | { kind: "summarizing" };
export type AgentItem =
  | {
      id: string;
      kind: "user";
      text: string;
      queued?: boolean;
      imageNames?: readonly string[];
      contextPaths?: readonly string[];
      entryId?: string;
    }
  | {
      id: string;
      kind: "assistant";
      text: string;
      /** Model reasoning is kept separate from the user-facing answer. */
      thinkingText?: string;
      streaming?: boolean;
    }
  | {
      id: string;
      kind: "tool";
      name: string;
      state: "running" | "success" | "failed";
      input: string;
      output: string;
      /** Total wall-clock time for the completed Pi turn owning this group. */
      turnDurationMs?: number;
    }
  | {
      id: string;
      kind: "notice";
      text: string;
      tone?: "default" | "error";
      recovery?: "retry_prompt";
    };

export interface AgentSessionStats {
  userMessages: number;
  assistantMessages: number;
  toolCalls: number;
  totalMessages: number;
  tokens: {
    input: number;
    output: number;
    cacheRead: number;
    cacheWrite: number;
    total: number;
  };
  cost: number;
  contextUsage?: {
    tokens: number | null;
    contextWindow: number;
    percent: number | null;
  };
}

export interface AgentCommand {
  name: string;
  description?: string;
  source: "extension" | "prompt" | "skill" | string;
}

export interface AgentModel {
  provider: string;
  id: string;
  name: string;
  reasoning: boolean;
  contextWindow?: number;
}

export type AgentThinkingLevel =
  | "off"
  | "minimal"
  | "low"
  | "medium"
  | "high"
  | "xhigh"
  | "max";

const thinkingLevels = new Set<AgentThinkingLevel>([
  "off",
  "minimal",
  "low",
  "medium",
  "high",
  "xhigh",
  "max",
]);

export interface AgentViewState {
  connection: AgentConnection;
  activity?: AgentActivity;
  queue: { steering: number; followUp: number };
  runtimeLogs: readonly string[];
  items: readonly AgentItem[];
  activeAssistantId?: string;
  turnStartedAtMs?: number;
  turnStartItemIndex?: number;
  error?: string;
  model?: string;
  modelId?: string;
  modelProvider?: string;
  models: readonly AgentModel[];
  thinking?: AgentThinkingLevel;
  autoCompactionEnabled?: boolean;
  steeringMode?: AgentQueueMode;
  followUpMode?: AgentQueueMode;
  availableThinkingLevels: readonly AgentThinkingLevel[];
  sessionId?: string;
  sessionFile?: string;
  sessionName?: string;
  stats?: AgentSessionStats;
  commands: readonly AgentCommand[];
}

export const initialAgentState: AgentViewState = {
  connection: "stopped",
  queue: { steering: 0, followUp: 0 },
  runtimeLogs: [],
  items: [],
  commands: [],
  models: [],
  availableThinkingLevels: [],
};

type JsonRecord = Record<string, unknown>;

const record = (value: unknown): JsonRecord | undefined =>
  value !== null && typeof value === "object" && !Array.isArray(value)
    ? (value as JsonRecord)
    : undefined;

const contentByType = (
  value: unknown,
  accepted: ReadonlySet<string>,
): string => {
  const source = record(value)?.content;
  if (!Array.isArray(source)) return "";
  return source
    .map((part) => record(part))
    .filter((part): part is JsonRecord => Boolean(part))
    .filter((part) => accepted.has(String(part.type)))
    .map((part) => (typeof part.text === "string" ? part.text : ""))
    .join("");
};

const textContent = (value: unknown): string =>
  contentByType(value, new Set(["text"]));

const thinkingContent = (value: unknown): string =>
  contentByType(value, new Set(["thinking"]));

const restoredItems = (value: unknown): readonly AgentItem[] => {
  const messages = record(value)?.messages;
  if (!Array.isArray(messages)) return [];
  return messages.flatMap((value, index): AgentItem[] => {
    const message = record(value);
    const role = message?.role;
    if (role !== "user" && role !== "assistant") return [];
    const text = textContent(message);
    if (!text) return [];
    return [
      {
        id: `restored-${index}`,
        kind: role,
        text,
        ...(role === "assistant"
          ? { thinkingText: thinkingContent(message) || undefined }
          : {}),
      },
    ];
  });
};

const finiteNumber = (value: unknown): number | undefined =>
  typeof value === "number" && Number.isFinite(value) ? value : undefined;

const sessionStats = (value: unknown): AgentSessionStats | undefined => {
  const data = record(value);
  const tokens = record(data?.tokens);
  if (!data || !tokens) return undefined;
  const userMessages = finiteNumber(data.userMessages);
  const assistantMessages = finiteNumber(data.assistantMessages);
  const toolCalls = finiteNumber(data.toolCalls);
  const totalMessages = finiteNumber(data.totalMessages);
  const input = finiteNumber(tokens.input);
  const output = finiteNumber(tokens.output);
  const cacheRead = finiteNumber(tokens.cacheRead);
  const cacheWrite = finiteNumber(tokens.cacheWrite);
  const total = finiteNumber(tokens.total);
  const cost = finiteNumber(data.cost);
  if (
    userMessages === undefined ||
    assistantMessages === undefined ||
    toolCalls === undefined ||
    totalMessages === undefined ||
    input === undefined ||
    output === undefined ||
    cacheRead === undefined ||
    cacheWrite === undefined ||
    total === undefined ||
    cost === undefined
  )
    return undefined;
  const context = record(data.contextUsage);
  const contextWindow = finiteNumber(context?.contextWindow);
  const contextTokens =
    context?.tokens === null ? null : finiteNumber(context?.tokens);
  const contextPercent =
    context?.percent === null ? null : finiteNumber(context?.percent);
  return {
    userMessages,
    assistantMessages,
    toolCalls,
    totalMessages,
    tokens: {
      input,
      output,
      cacheRead,
      cacheWrite,
      total,
    },
    cost,
    ...(context &&
    contextWindow !== undefined &&
    (contextTokens === null || contextTokens !== undefined) &&
    (contextPercent === null || contextPercent !== undefined)
      ? {
          contextUsage: {
            tokens: contextTokens,
            contextWindow,
            percent: contextPercent,
          },
        }
      : {}),
  };
};

const commands = (value: unknown): readonly AgentCommand[] => {
  const values = record(value)?.commands;
  if (!Array.isArray(values)) return [];
  return values.flatMap((value): AgentCommand[] => {
    const command = record(value);
    if (typeof command?.name !== "string" || typeof command.source !== "string")
      return [];
    return [
      {
        name: command.name,
        source: command.source,
        ...(typeof command.description === "string"
          ? { description: command.description }
          : {}),
      },
    ];
  });
};

const model = (value: unknown): AgentModel | undefined => {
  const candidate = record(value);
  if (
    typeof candidate?.provider !== "string" ||
    typeof candidate.id !== "string"
  )
    return undefined;
  return {
    provider: candidate.provider,
    id: candidate.id,
    name: typeof candidate.name === "string" ? candidate.name : candidate.id,
    reasoning: candidate.reasoning === true,
    ...(finiteNumber(candidate.contextWindow) !== undefined
      ? { contextWindow: finiteNumber(candidate.contextWindow) }
      : {}),
  };
};

const models = (value: unknown): readonly AgentModel[] => {
  const values = record(value)?.models;
  if (!Array.isArray(values)) return [];
  return values.flatMap((value) => {
    const parsed = model(value);
    return parsed ? [parsed] : [];
  });
};

const availableThinkingLevels = (
  value: unknown,
): readonly AgentThinkingLevel[] => {
  const values = record(value)?.levels;
  if (!Array.isArray(values)) return [];
  return values.filter(
    (value): value is AgentThinkingLevel =>
      typeof value === "string" &&
      thinkingLevels.has(value as AgentThinkingLevel),
  );
};

function attachForkEntryIds(
  items: readonly AgentItem[],
  value: unknown,
): readonly AgentItem[] {
  const values = record(value)?.messages;
  if (!Array.isArray(values)) return items;
  const next = [...items];
  let cursor = 0;
  for (const value of values) {
    const point = record(value);
    if (typeof point?.entryId !== "string" || typeof point.text !== "string")
      continue;
    const index = next.findIndex(
      (item, index) =>
        index >= cursor && item.kind === "user" && item.text === point.text,
    );
    if (index < 0) continue;
    const item = next[index];
    if (item.kind === "user") next[index] = { ...item, entryId: point.entryId };
    cursor = index + 1;
  }
  return next;
}

const replaceItem = (
  items: readonly AgentItem[],
  id: string,
  update: (item: AgentItem) => AgentItem,
): readonly AgentItem[] =>
  items.map((item) => (item.id === id ? update(item) : item));

const insertBeforeItem = (
  items: readonly AgentItem[],
  beforeId: string | undefined,
  item: AgentItem,
): readonly AgentItem[] => {
  if (!beforeId) return [...items, item];
  const index = items.findIndex((candidate) => candidate.id === beforeId);
  if (index < 0) return [...items, item];
  return [...items.slice(0, index), item, ...items.slice(index)];
};

function completeTurnItems(
  state: AgentViewState,
  settledAtMs: number | undefined,
): readonly AgentItem[] {
  const items = state.items.map((item) =>
    item.kind === "user" && item.queued ? { ...item, queued: false } : item,
  );
  if (
    settledAtMs === undefined ||
    state.turnStartedAtMs === undefined ||
    state.turnStartItemIndex === undefined
  ) {
    return items;
  }
  const turnStartIndex = state.turnStartItemIndex;
  const lastToolIndex = items.findLastIndex(
    (item, index) => index >= turnStartIndex && item.kind === "tool",
  );
  if (lastToolIndex < 0) return items;
  const duration = Math.max(0, settledAtMs - state.turnStartedAtMs);
  return items.map((item, index) =>
    index === lastToolIndex && item.kind === "tool"
      ? { ...item, turnDurationMs: duration }
      : item,
  );
}

export function appendUserMessage(
  state: AgentViewState,
  id: string,
  text: string,
  queued = false,
  imageNames: readonly string[] = [],
  contextPaths: readonly string[] = [],
): AgentViewState {
  return {
    ...state,
    items: [
      ...state.items,
      {
        id,
        kind: "user",
        text,
        queued,
        ...(imageNames.length > 0 ? { imageNames } : {}),
        ...(contextPaths.length > 0 ? { contextPaths } : {}),
      },
    ],
  };
}

export function reducePiEvent(
  state: AgentViewState,
  event: JsonRecord,
): AgentViewState {
  const type = typeof event.type === "string" ? event.type : "unknown";
  return match(type)
    .with("process_start", () => ({
      ...state,
      connection: "ready" as const,
      error: undefined,
      runtimeLogs: [],
    }))
    .with("process_log", () => {
      const message =
        typeof event.message === "string" ? event.message.trim() : "";
      if (!message) return state;
      return {
        ...state,
        runtimeLogs: [...state.runtimeLogs.slice(-99), message],
      };
    })
    .with("process_exit", () => ({
      ...state,
      connection: "stopped" as const,
      activity: undefined,
    }))
    .with("agent_start", () => ({
      ...state,
      connection: "running" as const,
      error: undefined,
      activity: { kind: "responding" as const },
      turnStartedAtMs: finiteNumber(event.receivedAtMs),
      turnStartItemIndex: state.items.length,
    }))
    .with("queue_update", () => ({
      ...state,
      queue: {
        steering: Array.isArray(event.steering) ? event.steering.length : 0,
        followUp: Array.isArray(event.followUp) ? event.followUp.length : 0,
      },
    }))
    .with("compaction_start", () => ({
      ...state,
      connection: "running" as const,
      activity: {
        kind: "compacting" as const,
        ...(typeof event.reason === "string" ? { reason: event.reason } : {}),
      },
    }))
    .with("compaction_end", () => ({
      ...state,
      connection:
        event.willRetry === true ? ("running" as const) : ("ready" as const),
      activity:
        event.willRetry === true
          ? ({ kind: "responding" } as const)
          : undefined,
    }))
    .with("auto_retry_start", () => ({
      ...state,
      connection: "running" as const,
      activity: {
        kind: "retrying" as const,
        ...(finiteNumber(event.attempt) !== undefined
          ? { attempt: finiteNumber(event.attempt) }
          : {}),
        ...(finiteNumber(event.maxAttempts) !== undefined
          ? { maxAttempts: finiteNumber(event.maxAttempts) }
          : {}),
        ...(finiteNumber(event.delayMs) !== undefined
          ? { delayMs: finiteNumber(event.delayMs) }
          : {}),
      },
    }))
    .with("auto_retry_end", () => ({
      ...state,
      activity:
        event.success === true ? ({ kind: "responding" } as const) : undefined,
    }))
    .with(
      P.union(
        "summarization_retry_scheduled",
        "summarization_retry_attempt_start",
      ),
      () => ({
        ...state,
        connection: "running" as const,
        activity: { kind: "summarizing" as const },
      }),
    )
    .with("summarization_retry_finished", () => ({
      ...state,
      activity: { kind: "compacting" as const },
    }))
    .with("response", () => {
      if (event.success !== true) {
        const message = String(event.error ?? "Pi RPC command failed");
        const requestId =
          typeof event.id === "string"
            ? event.id.match(/^wabou-request:(.+)$/)?.[1]
            : undefined;
        if (
          requestId &&
          (event.command === "prompt" ||
            event.command === "steer" ||
            event.command === "follow_up")
        ) {
          return {
            ...state,
            error: message,
            items: [
              ...state.items.filter((item) => item.id !== requestId),
              {
                id: `notice-${state.items.length + 1}`,
                kind: "notice" as const,
                text: message,
                tone: "error" as const,
                recovery: "retry_prompt" as const,
              },
            ],
          };
        }
        return { ...state, error: message };
      }
      if (event.command === "new_session") {
        return {
          ...state,
          items: [],
          activeAssistantId: undefined,
          stats: undefined,
        };
      }
      const data = record(event.data);
      if (event.command === "get_messages") {
        return {
          ...state,
          items: restoredItems(data),
          activeAssistantId: undefined,
        };
      }
      if (event.command === "get_session_stats") {
        return { ...state, stats: sessionStats(event.data) ?? state.stats };
      }
      if (event.command === "get_commands") {
        return { ...state, commands: commands(event.data) };
      }
      if (event.command === "get_available_models") {
        return { ...state, models: models(event.data) };
      }
      if (event.command === "get_available_thinking_levels") {
        return {
          ...state,
          availableThinkingLevels: availableThinkingLevels(event.data),
        };
      }
      if (event.command === "get_fork_messages") {
        return { ...state, items: attachForkEntryIds(state.items, event.data) };
      }
      const selectedModel = record(
        event.command === "set_model" ? event.data : data?.model,
      );
      if (event.command === "cycle_thinking_level") {
        return {
          ...state,
          thinking:
            typeof data?.level === "string" &&
            thinkingLevels.has(data.level as AgentThinkingLevel)
              ? (data.level as AgentThinkingLevel)
              : state.thinking,
        };
      }
      if (
        event.command !== "get_state" &&
        event.command !== "cycle_model" &&
        event.command !== "set_model"
      ) {
        return state;
      }
      return {
        ...state,
        model:
          typeof selectedModel?.name === "string"
            ? selectedModel.name
            : state.model,
        modelId:
          typeof selectedModel?.id === "string"
            ? selectedModel.id
            : state.modelId,
        modelProvider:
          typeof selectedModel?.provider === "string"
            ? selectedModel.provider
            : state.modelProvider,
        thinking:
          typeof data?.thinkingLevel === "string" &&
          thinkingLevels.has(data.thinkingLevel as AgentThinkingLevel)
            ? (data.thinkingLevel as AgentThinkingLevel)
            : state.thinking,
        autoCompactionEnabled:
          typeof data?.autoCompactionEnabled === "boolean"
            ? data.autoCompactionEnabled
            : state.autoCompactionEnabled,
        steeringMode:
          data?.steeringMode === "all" || data?.steeringMode === "one-at-a-time"
            ? data.steeringMode
            : state.steeringMode,
        followUpMode:
          data?.followUpMode === "all" || data?.followUpMode === "one-at-a-time"
            ? data.followUpMode
            : state.followUpMode,
        sessionId:
          typeof data?.sessionId === "string"
            ? data.sessionId
            : state.sessionId,
        sessionFile:
          typeof data?.sessionFile === "string"
            ? data.sessionFile
            : state.sessionFile,
        sessionName:
          typeof data?.sessionName === "string"
            ? data.sessionName
            : state.sessionName,
      };
    })
    .with("agent_end", () => ({
      ...state,
      activeAssistantId: undefined,
      items: state.items.map((item) =>
        item.kind === "assistant" && item.streaming
          ? { ...item, streaming: false }
          : item,
      ),
    }))
    .with("agent_settled", () => ({
      ...state,
      connection: "ready" as const,
      activity: undefined,
      queue: { steering: 0, followUp: 0 },
      activeAssistantId: undefined,
      turnStartedAtMs: undefined,
      turnStartItemIndex: undefined,
      items: completeTurnItems(state, finiteNumber(event.receivedAtMs)),
    }))
    .with("message_start", () => {
      const message = record(event.message);
      if (message?.role !== "assistant") return state;
      const id = `assistant-${state.items.length + 1}`;
      return {
        ...state,
        activeAssistantId: id,
        items: [
          ...state.items,
          {
            id,
            kind: "assistant" as const,
            text: textContent(message),
            thinkingText: thinkingContent(message) || undefined,
            streaming: true,
          },
        ],
      };
    })
    .with("message_update", () => {
      const update = record(event.assistantMessageEvent);
      if (!state.activeAssistantId || !update) return state;
      const delta = typeof update.delta === "string" ? update.delta : "";
      if (update.type !== "text_delta" && update.type !== "thinking_delta")
        return state;
      return {
        ...state,
        items: replaceItem(state.items, state.activeAssistantId, (item) =>
          item.kind === "assistant"
            ? update.type === "thinking_delta"
              ? { ...item, thinkingText: (item.thinkingText ?? "") + delta }
              : { ...item, text: item.text + delta }
            : item,
        ),
      };
    })
    .with("message_end", () => ({
      ...state,
      activeAssistantId: undefined,
      items: state.items.map((item) =>
        item.id === state.activeAssistantId && item.kind === "assistant"
          ? { ...item, streaming: false }
          : item,
      ),
    }))
    .with("tool_execution_start", () => {
      const id = String(event.toolCallId ?? `tool-${state.items.length + 1}`);
      return {
        ...state,
        // Pi starts the assistant message before it emits tool events, then
        // continues writing the final answer into that same message. Keep the
        // retained message identity, but place work before the answer so the
        // transcript reads reasoning → tools → response.
        items: insertBeforeItem(state.items, state.activeAssistantId, {
          id,
          kind: "tool" as const,
          name: String(event.toolName ?? "tool"),
          state: "running" as const,
          input: JSON.stringify(event.args ?? {}, null, 2),
          output: "",
        }),
      };
    })
    .with(P.union("tool_execution_update", "tool_execution_end"), () => {
      const id = String(event.toolCallId ?? "");
      const result = record(event.partialResult ?? event.result);
      const output = textContent(result);
      const failed = record(event.result)?.isError === true;
      return {
        ...state,
        items: replaceItem(state.items, id, (item) =>
          item.kind === "tool"
            ? {
                ...item,
                output: output || item.output,
                state: match({ type, failed })
                  .with(
                    { type: "tool_execution_update" },
                    () => "running" as const,
                  )
                  .with({ failed: true }, () => "failed" as const)
                  .otherwise(() => "success" as const),
              }
            : item,
        ),
      };
    })
    .with("bridge_error", () => {
      const message = String(event.message ?? "Pi reported an error");
      return {
        ...state,
        connection: "failed" as const,
        error: message,
        items: [
          ...state.items,
          {
            id: `notice-${state.items.length + 1}`,
            kind: "notice" as const,
            text: message,
            tone: "error" as const,
          },
        ],
      };
    })
    .with("request_error", () => {
      const message = String(event.message ?? "Pi request failed");
      const userMessageId =
        typeof event.userMessageId === "string"
          ? event.userMessageId
          : undefined;
      return {
        ...state,
        error: message,
        items: [
          ...state.items.filter((item) => item.id !== userMessageId),
          {
            id: `notice-${state.items.length + 1}`,
            kind: "notice" as const,
            text: message,
            tone: "error" as const,
            recovery: "retry_prompt" as const,
          },
        ],
      };
    })
    .with("extension_error", () => {
      const message = String(
        event.message ?? "A Pi extension reported an error",
      );
      return {
        ...state,
        error: message,
        items: [
          ...state.items,
          {
            id: `notice-${state.items.length + 1}`,
            kind: "notice" as const,
            text: message,
            tone: "error" as const,
          },
        ],
      };
    })
    .otherwise(() => state);
}

export function reducePiEvents(
  state: AgentViewState,
  events: readonly JsonRecord[],
): AgentViewState {
  return events.reduce(reducePiEvent, state);
}
