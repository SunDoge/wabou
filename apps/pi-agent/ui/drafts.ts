export type AgentDrafts = Readonly<Record<string, string>>;

const NEW_SESSION = "new";

export function agentDraftKey(agentId: string, sessionId?: string): string {
  return `${agentId}\0${sessionId || NEW_SESSION}`;
}

export function readAgentDraft(
  drafts: AgentDrafts,
  agentId: string,
  sessionId?: string,
): string {
  return drafts[agentDraftKey(agentId, sessionId)] ?? "";
}

export function writeAgentDraft(
  drafts: AgentDrafts,
  agentId: string,
  sessionId: string | undefined,
  value: string,
): AgentDrafts {
  const key = agentDraftKey(agentId, sessionId);
  if (!value) {
    if (!(key in drafts)) return drafts;
    const next = { ...drafts };
    delete next[key];
    return next;
  }
  if (drafts[key] === value) return drafts;
  return { ...drafts, [key]: value };
}

export function removeAgentDrafts(
  drafts: AgentDrafts,
  agentId: string,
): AgentDrafts {
  const prefix = `${agentId}\0`;
  const next = Object.fromEntries(
    Object.entries(drafts).filter(([key]) => !key.startsWith(prefix)),
  );
  return Object.keys(next).length === Object.keys(drafts).length
    ? drafts
    : next;
}
