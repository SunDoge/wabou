import { reducePiEvents } from "./agent-state";
import type { PiSession, usePiApi } from "./api";
import {
  type ExtensionUiDialogRequest,
  parseExtensionUiRequest,
} from "./extension-ui";
import type { AgentWorkspace } from "./workspace";

type PiApi = ReturnType<typeof usePiApi>;
type Event = Record<string, unknown>;

export interface AgentEventSubscriptionOptions {
  api: PiApi;
  activeAgentId: () => string;
  updateAgent: (
    id: string,
    update: (agent: AgentWorkspace) => AgentWorkspace,
  ) => void;
  updateSessions: (
    update: (current: readonly PiSession[]) => readonly PiSession[],
  ) => void;
  updateDialogs: (
    update: (
      current: readonly ExtensionUiDialogRequest[],
    ) => readonly ExtensionUiDialogRequest[],
  ) => void;
  navigateToSession: (agentId: string, sessionId: string) => void;
  refreshWorkspaceInfo: () => void;
  restoreForkDraft: (text: string) => void;
  exported: (path: string) => void;
}

function successfulResponse(events: readonly Event[], command: string) {
  return events.find(
    (event) =>
      event.type === "response" &&
      event.command === command &&
      event.success === true,
  );
}

function groupByAgent(events: readonly Event[]) {
  const grouped = new Map<string, Event[]>();
  for (const event of events) {
    const id = typeof event.agentId === "string" ? event.agentId : "agent-1";
    const group = grouped.get(id) ?? [];
    group.push(event);
    grouped.set(id, group);
  }
  return grouped;
}

/** Keep host-event orchestration out of the retained view component. */
export function subscribeAgentEvents(
  options: AgentEventSubscriptionOptions,
): () => void {
  const { api } = options;
  return api.subscribe((events) => {
    for (const event of events) {
      const id = typeof event.agentId === "string" ? event.agentId : "agent-1";
      const dialog = parseExtensionUiRequest(event);
      if (dialog) {
        options.updateDialogs((current) =>
          current.some(
            (candidate) =>
              candidate.agentId === dialog.agentId &&
              candidate.id === dialog.id,
          )
            ? current
            : [...current, dialog],
        );
      }
      if (event.type === "process_exit") {
        options.updateDialogs((current) =>
          current.filter((candidate) => candidate.agentId !== id),
        );
      }
    }

    for (const [id, batch] of groupByAgent(events)) {
      options.updateAgent(id, (agent) => ({
        ...agent,
        state: reducePiEvents(agent.state, batch),
      }));

      const stateEvent = successfulResponse(batch, "get_state");
      if (stateEvent) {
        if (
          stateEvent.id === "wabou-bootstrap-state" ||
          stateEvent.id === "wabou-new-session-state" ||
          stateEvent.id === "wabou-clone-state"
        ) {
          void api.getMessages(id);
        }
        void api.getSessionStats(id);
        void api.getCommands(id);
        void api.getModelOptions(id);
        void api.listSessions(id).then((next) =>
          options.updateSessions((current) => [
            ...current.filter((session) => session.agentId !== id),
            ...next,
          ]),
        );
        const data = stateEvent.data as Record<string, unknown> | null;
        if (
          (stateEvent.id === "wabou-new-session-state" ||
            stateEvent.id === "wabou-clone-state") &&
          id === options.activeAgentId() &&
          typeof data?.sessionId === "string"
        ) {
          options.navigateToSession(id, data.sessionId);
        }
      }

      if (batch.some((event) => event.type === "agent_settled")) {
        void api.getSessionStats(id);
        void api.getForkMessages(id);
        if (id === options.activeAgentId()) options.refreshWorkspaceInfo();
      }
      if (successfulResponse(batch, "compact")) {
        void api.getMessages(id);
        void api.getSessionStats(id);
      }

      const exportEvent = successfulResponse(batch, "export_html");
      const exported = exportEvent?.data as Record<string, unknown> | undefined;
      if (typeof exported?.path === "string") options.exported(exported.path);

      if (successfulResponse(batch, "get_messages")) {
        void api.getForkMessages(id);
      }

      const forkEvent = successfulResponse(batch, "fork");
      const forkData = forkEvent?.data as Record<string, unknown> | undefined;
      if (forkEvent && forkData?.cancelled !== true) {
        if (id === options.activeAgentId() && typeof forkData?.text === "string") {
          options.restoreForkDraft(forkData.text);
        }
        void api.getMessages(id);
        void api.getSessionStats(id);
      }
    }
  });
}
