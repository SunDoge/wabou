import { match, P } from "ts-pattern";

export type AgentConnection = "stopped" | "ready" | "running" | "failed";
export type AgentItem =
  | {
      id: string;
      kind: "user" | "assistant";
      text: string;
      /** Model reasoning is kept separate from the user-facing answer. */
      thinkingText?: string;
      streaming?: boolean;
      queued?: boolean;
    }
  | {
      id: string;
      kind: "tool";
      name: string;
      state: "running" | "success" | "failed";
      input: string;
      output: string;
    }
  | { id: string; kind: "notice"; text: string; tone?: "default" | "error" };

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

export interface AgentViewState {
  connection: AgentConnection;
  items: readonly AgentItem[];
  activeAssistantId?: string;
  error?: string;
  model?: string;
  thinking?: string;
  sessionId?: string;
  sessionFile?: string;
  sessionName?: string;
  stats?: AgentSessionStats;
}

export const initialAgentState: AgentViewState = {
  connection: "stopped",
  items: [],
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

const replaceItem = (
  items: readonly AgentItem[],
  id: string,
  update: (item: AgentItem) => AgentItem,
): readonly AgentItem[] =>
  items.map((item) => (item.id === id ? update(item) : item));

export function appendUserMessage(
  state: AgentViewState,
  id: string,
  text: string,
  queued = false,
): AgentViewState {
  return {
    ...state,
    items: [...state.items, { id, kind: "user", text, queued }],
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
    }))
    .with("process_exit", () => ({ ...state, connection: "stopped" as const }))
    .with("agent_start", () => ({ ...state, connection: "running" as const }))
    .with("response", () => {
      if (event.success !== true) {
        const message = String(event.error ?? "Pi RPC command failed");
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
      const model = record(
        event.command === "set_model" ? event.data : data?.model,
      );
      if (event.command === "cycle_thinking_level") {
        return {
          ...state,
          thinking:
            typeof data?.level === "string" ? data.level : state.thinking,
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
        model: typeof model?.name === "string" ? model.name : state.model,
        thinking:
          typeof data?.thinkingLevel === "string"
            ? data.thinkingLevel
            : state.thinking,
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
      connection: "ready" as const,
      activeAssistantId: undefined,
      items: state.items.map((item) =>
        item.kind === "assistant" && item.streaming
          ? { ...item, streaming: false }
          : item.kind === "user" && item.queued
            ? { ...item, queued: false }
            : item,
      ),
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
        items: [
          ...state.items,
          {
            id,
            kind: "tool" as const,
            name: String(event.toolName ?? "tool"),
            state: "running" as const,
            input: JSON.stringify(event.args ?? {}, null, 2),
            output: "",
          },
        ],
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
    .with(P.union("bridge_error", "extension_error"), () => {
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
    .otherwise(() => state);
}

export function reducePiEvents(
  state: AgentViewState,
  events: readonly JsonRecord[],
): AgentViewState {
  return events.reduce(reducePiEvent, state);
}
