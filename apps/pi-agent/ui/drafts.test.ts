import { describe, expect, test } from "bun:test";
import {
  agentDraftKey,
  readAgentDraft,
  removeAgentDrafts,
  writeAgentDraft,
} from "./drafts";

describe("Pi Agent conversation drafts", () => {
  test("keeps independent drafts for agents and restored sessions", () => {
    let drafts = {};
    drafts = writeAgentDraft(drafts, "agent-1", undefined, "new task");
    drafts = writeAgentDraft(drafts, "agent-1", "session-a", "follow up");
    drafts = writeAgentDraft(drafts, "agent-2", undefined, "other agent");

    expect(readAgentDraft(drafts, "agent-1")).toBe("new task");
    expect(readAgentDraft(drafts, "agent-1", "session-a")).toBe("follow up");
    expect(readAgentDraft(drafts, "agent-2")).toBe("other agent");
  });

  test("clears only the submitted draft and removes deleted agent drafts", () => {
    let drafts = {
      [agentDraftKey("agent-1")]: "new task",
      [agentDraftKey("agent-1", "session-a")]: "follow up",
      [agentDraftKey("agent-2")]: "keep me",
    };

    drafts = writeAgentDraft(drafts, "agent-1", "session-a", "");
    expect(readAgentDraft(drafts, "agent-1")).toBe("new task");
    expect(readAgentDraft(drafts, "agent-1", "session-a")).toBe("");

    drafts = removeAgentDrafts(drafts, "agent-1");
    expect(readAgentDraft(drafts, "agent-1")).toBe("");
    expect(readAgentDraft(drafts, "agent-2")).toBe("keep me");
  });
});
