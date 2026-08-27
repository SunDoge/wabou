import { describe, expect, test } from "vitest";
import {
  appendUserMessage,
  initialAgentState,
  reducePiEvent,
  reducePiEvents,
} from "./agent-state";

describe("Pi agent event projection", () => {
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

  test("keeps RPC failures attached to the affected agent state", () => {
    const state = reducePiEvent(initialAgentState, {
      type: "response",
      command: "set_model",
      success: false,
      error: "model unavailable",
    });
    expect(state.error).toBe("model unavailable");
  });
});
