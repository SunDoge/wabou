import { describe, expect, test } from "vitest";
import {
  appendUserMessage,
  initialAgentState,
  reconcileProcessConnection,
  reducePiEvent,
  reducePiEvents,
} from "./agent-state";

describe("Pi agent event projection", () => {
  test("does not let a stale process status erase active response state", () => {
    expect(reconcileProcessConnection("running", true)).toBe("running");
    expect(reconcileProcessConnection("failed", true)).toBe("failed");
    expect(reconcileProcessConnection("stopped", true)).toBe("ready");
    expect(reconcileProcessConnection("running", false)).toBe("stopped");
  });

  test("streams an assistant message without duplicating it", () => {
    let state = appendUserMessage(initialAgentState, "user-1", "hello");
    state = reducePiEvent(state, { type: "agent_start" });
    state = reducePiEvent(state, {
      type: "message_start",
      message: { role: "assistant", content: [] },
    });
    state = reducePiEvent(state, {
      type: "message_update",
      assistantMessageEvent: { type: "text_delta", delta: "Hello " },
    });
    state = reducePiEvent(state, {
      type: "message_update",
      assistantMessageEvent: { type: "text_delta", delta: "world" },
    });
    state = reducePiEvent(state, { type: "message_end" });
    expect(state.items).toHaveLength(2);
    expect(state.items[1]).toMatchObject({
      kind: "assistant",
      text: "Hello world",
      streaming: false,
    });
  });

  test("keeps model reasoning separate from the user-facing answer", () => {
    let state = reducePiEvent(initialAgentState, {
      type: "message_start",
      message: { role: "assistant", content: [] },
    });
    state = reducePiEvent(state, {
      type: "message_update",
      assistantMessageEvent: {
        type: "thinking_delta",
        delta: "Inspecting files…",
      },
    });
    state = reducePiEvent(state, {
      type: "message_update",
      assistantMessageEvent: { type: "text_delta", delta: "Found it." },
    });

    expect(state.items[0]).toMatchObject({
      kind: "assistant",
      thinkingText: "Inspecting files…",
      text: "Found it.",
    });
  });

  test("tracks tool execution by call id", () => {
    let state = reducePiEvent(initialAgentState, {
      type: "tool_execution_start",
      toolCallId: "call-1",
      toolName: "bash",
      args: { command: "pwd" },
    });
    state = reducePiEvent(state, {
      type: "tool_execution_end",
      toolCallId: "call-1",
      result: { content: [{ type: "text", text: "/tmp" }] },
    });
    expect(state.items[0]).toMatchObject({
      kind: "tool",
      state: "success",
      output: "/tmp",
    });
  });

  test("projects the Rust event clock into the completed tool turn", () => {
    let state = reducePiEvent(initialAgentState, {
      type: "agent_start",
      receivedAtMs: 1_000,
    });
    state = reducePiEvent(state, {
      type: "tool_execution_start",
      toolCallId: "call-timed",
      toolName: "read",
      args: { path: "README.md" },
    });
    state = reducePiEvent(state, {
      type: "tool_execution_end",
      toolCallId: "call-timed",
      result: { content: [{ type: "text", text: "ok" }] },
    });
    state = reducePiEvent(state, {
      type: "agent_settled",
      receivedAtMs: 13_200,
    });

    expect(state.items[0]).toMatchObject({
      kind: "tool",
      turnDurationMs: 12_200,
    });
    expect(state.turnStartedAtMs).toBeUndefined();
    expect(state.turnStartItemIndex).toBeUndefined();
  });

  test("places tool work before the retained assistant answer", () => {
    let state = reducePiEvent(initialAgentState, {
      type: "message_start",
      message: { role: "assistant", content: [] },
    });
    const assistantId = state.activeAssistantId;
    state = reducePiEvent(state, {
      type: "tool_execution_start",
      toolCallId: "read-1",
      toolName: "read",
      args: { path: "README.md" },
    });
    state = reducePiEvent(state, {
      type: "message_update",
      assistantMessageEvent: { type: "text_delta", delta: "Finished." },
    });

    expect(state.items.map((item) => item.kind)).toEqual(["tool", "assistant"]);
    expect(state.items[1]).toMatchObject({
      id: assistantId,
      kind: "assistant",
      text: "Finished.",
    });
  });

  test("commits a batch of deltas and bootstrap state as one projection", () => {
    const state = reducePiEvents(initialAgentState, [
      { type: "process_start" },
      {
        type: "response",
        command: "get_state",
        success: true,
        data: {
          model: { name: "GPT-5.5" },
          thinkingLevel: "high",
          sessionId: "session-1",
        },
      },
      { type: "message_start", message: { role: "assistant", content: [] } },
      {
        type: "message_update",
        assistantMessageEvent: { type: "text_delta", delta: "batched" },
      },
    ]);
    expect(state).toMatchObject({
      connection: "ready",
      model: "GPT-5.5",
      thinking: "high",
      sessionId: "session-1",
    });
    expect(state.items[0]).toMatchObject({
      kind: "assistant",
      text: "batched",
    });
  });

  test("projects explicit provider model changes", () => {
    const state = reducePiEvent(initialAgentState, {
      type: "response",
      command: "set_model",
      success: true,
      data: { provider: "openai", id: "gpt-5.5", name: "GPT-5.5" },
    });
    expect(state.model).toBe("GPT-5.5");
  });

  test("projects authoritative session behavior settings", () => {
    const state = reducePiEvent(initialAgentState, {
      type: "response",
      command: "get_state",
      success: true,
      data: {
        autoCompactionEnabled: true,
        steeringMode: "all",
        followUpMode: "one-at-a-time",
      },
    });
    expect(state).toMatchObject({
      autoCompactionEnabled: true,
      steeringMode: "all",
      followUpMode: "one-at-a-time",
    });
  });

  test("marks follow-ups as queued until the agent fully settles", () => {
    const running = reducePiEvent(initialAgentState, { type: "agent_start" });
    const queued = appendUserMessage(running, "user-1", "Next", true);
    expect(queued.items[0]).toMatchObject({ kind: "user", queued: true });

    const betweenRuns = reducePiEvent(queued, { type: "agent_end" });
    expect(betweenRuns.connection).toBe("running");
    expect(betweenRuns.items[0]).toMatchObject({ queued: true });

    const ready = reducePiEvent(betweenRuns, { type: "agent_settled" });
    expect(ready.connection).toBe("ready");
    expect(ready.items[0]).toMatchObject({ kind: "user", queued: false });
  });

  test("projects compaction, retry and queue activity without appearing idle", () => {
    let state = reducePiEvent(initialAgentState, {
      type: "queue_update",
      steering: ["Now"],
      followUp: ["Later", "Finally"],
    });
    expect(state.queue).toEqual({ steering: 1, followUp: 2 });

    state = reducePiEvent(state, {
      type: "compaction_start",
      reason: "threshold",
    });
    expect(state).toMatchObject({
      connection: "running",
      activity: { kind: "compacting", reason: "threshold" },
    });

    state = reducePiEvent(state, {
      type: "auto_retry_start",
      attempt: 2,
      maxAttempts: 3,
      delayMs: 2_000,
    });
    expect(state.activity).toEqual({
      kind: "retrying",
      attempt: 2,
      maxAttempts: 3,
      delayMs: 2_000,
    });

    state = reducePiEvent(state, { type: "agent_settled" });
    expect(state).toMatchObject({
      connection: "ready",
      queue: { steering: 0, followUp: 0 },
    });
    expect(state.activity).toBeUndefined();
  });

  test("restores persisted Pi messages when a session is opened", () => {
    const state = reducePiEvent(initialAgentState, {
      type: "response",
      command: "get_messages",
      success: true,
      data: {
        messages: [
          { role: "user", content: [{ type: "text", text: "Fix it" }] },
          {
            role: "assistant",
            content: [
              { type: "thinking", text: "Checked the repository" },
              { type: "text", text: "Done" },
            ],
          },
          { role: "toolResult", content: [{ type: "text", text: "ignored" }] },
        ],
      },
    });
    expect(state.items).toEqual([
      { id: "restored-0", kind: "user", text: "Fix it" },
      {
        id: "restored-1",
        kind: "assistant",
        text: "Done",
        thinkingText: "Checked the repository",
      },
    ]);
  });

  test("keeps RPC failures attached to the affected agent state", () => {
    const state = reducePiEvent(initialAgentState, {
      type: "response",
      command: "set_model",
      success: false,
      error: "model unavailable",
    });
    expect(state.error).toBe("model unavailable");
  });

  test("reports extension failures without stopping a healthy agent", () => {
    const running = reducePiEvent(initialAgentState, { type: "agent_start" });
    const state = reducePiEvent(running, {
      type: "extension_error",
      message: "broken status extension",
    });

    expect(state.connection).toBe("running");
    expect(state.error).toBe("broken status extension");
    expect(state.items.at(-1)).toMatchObject({
      kind: "notice",
      tone: "error",
      text: "broken status extension",
    });
  });

  test("rolls back a rejected prompt without stopping a healthy agent", () => {
    const ready = reducePiEvent(initialAgentState, { type: "process_start" });
    const optimistic = appendUserMessage(
      ready,
      "user-pending",
      "Retry this request",
    );
    const failed = reducePiEvent(optimistic, {
      type: "request_error",
      userMessageId: "user-pending",
      message: "provider unavailable",
    });

    expect(failed.connection).toBe("ready");
    expect(failed.items).not.toContainEqual(
      expect.objectContaining({ id: "user-pending" }),
    );
    expect(failed.items.at(-1)).toMatchObject({
      kind: "notice",
      tone: "error",
      text: "provider unavailable",
      recovery: "retry_prompt",
    });

    const retried = reducePiEvent(failed, { type: "agent_start" });
    expect(retried.connection).toBe("running");
    expect(retried.error).toBeUndefined();
  });

  test("projects an asynchronous prompt rejection onto its optimistic message", () => {
    const ready = reducePiEvent(initialAgentState, { type: "process_start" });
    const optimistic = appendUserMessage(ready, "user-async", "Rejected later");
    const failed = reducePiEvent(optimistic, {
      id: "wabou-request:user-async",
      type: "response",
      command: "prompt",
      success: false,
      error: "provider unavailable",
    });

    expect(failed.connection).toBe("ready");
    expect(failed.items).not.toContainEqual(
      expect.objectContaining({ id: "user-async" }),
    );
    expect(failed.items.at(-1)).toMatchObject({
      kind: "notice",
      tone: "error",
      text: "provider unavailable",
      recovery: "retry_prompt",
    });
  });

  test("projects authoritative session and context usage", () => {
    const state = reducePiEvent(initialAgentState, {
      type: "response",
      command: "get_session_stats",
      success: true,
      data: {
        userMessages: 3,
        assistantMessages: 2,
        toolCalls: 4,
        toolResults: 4,
        totalMessages: 9,
        tokens: {
          input: 10_000,
          output: 2_000,
          cacheRead: 8_000,
          cacheWrite: 500,
          total: 20_500,
        },
        cost: 0.1234,
        contextUsage: {
          tokens: 24_000,
          contextWindow: 128_000,
          percent: 18.75,
        },
      },
    });

    expect(state.stats).toMatchObject({
      totalMessages: 9,
      tokens: { total: 20_500 },
      cost: 0.1234,
      contextUsage: { percent: 18.75 },
    });
  });

  test("keeps only valid commands reported by Pi", () => {
    const state = reducePiEvent(initialAgentState, {
      type: "response",
      command: "get_commands",
      success: true,
      data: {
        commands: [
          {
            name: "fix-tests",
            description: "Fix failing tests",
            source: "prompt",
          },
          { name: "skill:review", source: "skill" },
          { description: "missing a name", source: "extension" },
        ],
      },
    });

    expect(state.commands).toEqual([
      {
        name: "fix-tests",
        description: "Fix failing tests",
        source: "prompt",
      },
      { name: "skill:review", source: "skill" },
    ]);
  });

  test("projects available models and supported thinking levels", () => {
    let state = reducePiEvent(initialAgentState, {
      type: "response",
      command: "get_available_models",
      success: true,
      data: {
        models: [
          {
            provider: "openai",
            id: "gpt-5.6",
            name: "GPT-5.6",
            reasoning: true,
            contextWindow: 400_000,
          },
          { provider: "broken", name: "Missing id" },
        ],
      },
    });
    state = reducePiEvent(state, {
      type: "response",
      command: "get_available_thinking_levels",
      success: true,
      data: { levels: ["off", "medium", "max", "invalid"] },
    });

    expect(state.models).toEqual([
      {
        provider: "openai",
        id: "gpt-5.6",
        name: "GPT-5.6",
        reasoning: true,
        contextWindow: 400_000,
      },
    ]);
    expect(state.availableThinkingLevels).toEqual(["off", "medium", "max"]);
  });

  test("associates stable fork ids with repeated user messages in order", () => {
    let state = appendUserMessage(initialAgentState, "first", "Retry");
    state = appendUserMessage(state, "second", "Retry");
    state = reducePiEvent(state, {
      type: "response",
      command: "get_fork_messages",
      success: true,
      data: {
        messages: [
          { entryId: "entry-a", text: "Retry" },
          { entryId: "entry-b", text: "Retry" },
        ],
      },
    });

    expect(state.items).toMatchObject([
      { id: "first", entryId: "entry-a" },
      { id: "second", entryId: "entry-b" },
    ]);
  });

  test("retains bounded runtime diagnostics and clears them on restart", () => {
    let state = initialAgentState;
    for (let index = 0; index < 105; index += 1) {
      state = reducePiEvent(state, {
        type: "process_log",
        message: `diagnostic ${index}`,
      });
    }

    expect(state.runtimeLogs).toHaveLength(100);
    expect(state.runtimeLogs[0]).toBe("diagnostic 5");
    expect(state.runtimeLogs.at(-1)).toBe("diagnostic 104");

    state = reducePiEvent(state, { type: "process_start" });
    expect(state.runtimeLogs).toEqual([]);
  });
});
