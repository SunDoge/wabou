import { describe, expect, test } from "bun:test";
import {
  type AgentDraftLists,
  agentDraftKey,
  readAgentDraft,
  readAgentDraftList,
  removeAgentDraftLists,
  removeAgentDrafts,
  writeAgentDraft,
  writeAgentDraftList,
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

test("keeps attachment lists scoped to an agent session", () => {
  let values: AgentDraftLists = {};
  values = writeAgentDraftList(values, "agent-1", "session-a", ["src/a.ts"]);
  values = writeAgentDraftList(values, "agent-2", undefined, ["src/b.ts"]);
  expect(readAgentDraftList(values, "agent-1", "session-a")).toEqual([
    "src/a.ts",
  ]);
  values = removeAgentDraftLists(values, "agent-1");
  expect(readAgentDraftList(values, "agent-1", "session-a")).toEqual([]);
  expect(readAgentDraftList(values, "agent-2")).toEqual(["src/b.ts"]);
});
