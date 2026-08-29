import {
  Button,
  createAsyncQuery,
  createToasts,
  currentWindow,
  type Handle,
  Icon,
  MessageScroller,
  MessageScrollerButton,
  MessageScrollerContent,
  MessageScrollerViewport,
  Text,
  TextArea,
  Toaster,
  useDialog,
  useLocation,
  useNavigate,
  useParams,
  View,
} from "@wabou/ui";
import filePlus from "lucide-static/icons/file-plus-2.svg?raw";
import folder from "lucide-static/icons/folder.svg?raw";
import gitBranch from "lucide-static/icons/git-branch.svg?raw";
import search from "lucide-static/icons/search.svg?raw";
import send from "lucide-static/icons/send.svg?raw";
import square from "lucide-static/icons/square.svg?raw";
import squareTerminal from "lucide-static/icons/square-terminal.svg?raw";
import { createEffect, createSignal, onCleanup, Show } from "solid-js";
import { AgentActivityStatus } from "./agent-activity";
import {
  appendUserMessage,
  reconcileProcessConnection,
  reducePiEvent,
  reducePiEvents,
} from "./agent-state";
import { type PiSession, usePiApi } from "./api";
import { CommandPicker } from "./command-picker";
import {
  ComposerContextFiles,
  WorkspaceContextPicker,
} from "./composer-context";
import {
  ComposerDeliveryControl,
  type ComposerDeliveryMode,
} from "./composer-delivery";
import {
  ComposerImagePicker,
  ComposerImages,
  imageFileName,
} from "./composer-images";
import { ConversationList } from "./conversation";
import { ConversationWelcome } from "./conversation-welcome";
import { createDeferredWriter } from "./deferred-writer";
import {
  type AgentDraftLists,
  type AgentDrafts,
  readAgentDraft,
  readAgentDraftList,
  removeAgentDraftLists,
  removeAgentDrafts,
  writeAgentDraft,
  writeAgentDraftList,
} from "./drafts";
import {
  type ExtensionUiAnswer,
  ExtensionUiChrome,
  ExtensionUiDialog,
  type ExtensionUiDialogRequest,
  type ExtensionUiEffect,
  type ExtensionUiStatus,
  type ExtensionUiWidget,
  parseExtensionUiEffect,
  parseExtensionUiRequest,
  reduceExtensionUiStatuses,
  reduceExtensionUiWidgets,
} from "./extension-ui";
import { i18n, m } from "./i18n";
import { ModelControls } from "./model-controls";
import { createOwnedOverlay } from "./owned-overlay";
import { createPersistedRecord } from "./persisted-record";
import { ScopedHandleRegistry } from "./scoped-handle-registry";
import { SessionActions } from "./session-actions";
import { SessionForkDialog } from "./session-fork";
import { SessionTitle } from "./session-title";
import { SessionUsage } from "./session-usage";
import { type AppSettings, SettingsPage } from "./settings";
import { Sidebar } from "./sidebar";
import { AgentTerminalPanel } from "./terminal-panel";
import { TranscriptSearch } from "./transcript-search";
import {
  type AgentWorkspace,
  agentProfile,
  createAgentWorkspace,
  restoreAgentWorkspace,
} from "./workspace";
import { WorkspaceChangesPanel } from "./workspace-changes-panel";
import { WorkspacePanel } from "./workspace-panel";
import { WorkspaceSetup } from "./workspace-setup";

function ExtensionWindowTitle(props: { title: string }) {
  createEffect(
    () => props.title,
    (title) => currentWindow().setTitle(title),
  );
  return null;
}

function dispatchExtensionUiEffect(
  event: Record<string, unknown>,
  handlers: {
    notify(effect: Extract<ExtensionUiEffect, { kind: "notify" }>): void;
    status(effect: Extract<ExtensionUiEffect, { kind: "status" }>): void;
    widget(effect: Extract<ExtensionUiEffect, { kind: "widget" }>): void;
    title(effect: Extract<ExtensionUiEffect, { kind: "title" }>): void;
    editorText(
      effect: Extract<ExtensionUiEffect, { kind: "editorText" }>,
    ): void;
  },
) {
  const effect = parseExtensionUiEffect(event);
  if (!effect) return;
  switch (effect.kind) {
    case "notify":
      handlers.notify(effect);
      break;
    case "status":
      handlers.status(effect);
      break;
    case "widget":
      handlers.widget(effect);
      break;
    case "title":
      handlers.title(effect);
      break;
    case "editorText":
      handlers.editorText(effect);
      break;
  }
}

export function App() {
  const api = usePiApi();
  const dialog = useDialog();
  const toasts = createToasts();
  const navigate = useNavigate();
  const location = useLocation();
  const params = useParams<{ agentId?: string; sessionId?: string }>();
  const defaults = createPersistedRecord<AppSettings>({
    initial: {
      locale: "en",
      proxy: "",
      noProxy: "127.0.0.1,localhost",
      provider: "",
      model: "",
      subagentsEnabled: true,
    },
    load: api.getAppSettings,
    save: api.saveAppSettings,
    onLoadError: (error) =>
      console.error(`[pi-agent] could not load app settings: ${String(error)}`),
    onSaveError: (error) =>
      console.error(`[pi-agent] could not save app settings: ${String(error)}`),
  });
  createEffect(
    () => defaults.value().locale,
    (locale) => i18n.set(locale),
  );
  const [agents, setAgents] = createSignal<readonly AgentWorkspace[]>([
    createAgentWorkspace(1),
  ]);
  const [sessions, setSessions] = createSignal<readonly PiSession[]>([]);
  const [workspaceRevision, setWorkspaceRevision] = createSignal(0);
  const [lastActiveId, setLastActiveId] = createSignal("agent-1");
  const [drafts, setDrafts] = createSignal<AgentDrafts>({});
  const [draftImages, setDraftImages] = createSignal<AgentDraftLists>({});
  const [draftContext, setDraftContext] = createSignal<AgentDraftLists>({});
  const [searchOpen, setSearchOpen] = createSignal(false);
  const [sidePanel, setSidePanel] = createSignal<"files" | "changes">();
  const [terminalOpen, setTerminalOpen] = createSignal(false);
  const [terminalMounted, setTerminalMounted] = createSignal(false);
  const [terminalOwnerId, setTerminalOwnerId] = createSignal<string>();
  const [deliveryMode, setDeliveryMode] =
    createSignal<ComposerDeliveryMode>("followUp");
  const [activeSearchItem, setActiveSearchItem] = createSignal<string>();
  const pendingFork = createOwnedOverlay<{
    entryId: string;
    text: string;
  }>();
  const [extensionDialogs, setExtensionDialogs] = createSignal<
    readonly ExtensionUiDialogRequest[]
  >([]);
  const [extensionStatuses, setExtensionStatuses] = createSignal<
    readonly ExtensionUiStatus[]
  >([]);
  const [extensionWidgets, setExtensionWidgets] = createSignal<
    readonly ExtensionUiWidget[]
  >([]);
  const [extensionTitles, setExtensionTitles] = createSignal<
    Readonly<Record<string, string>>
  >({});
  const deliveredNotifications = new Set<string>();
  const itemHandles = new ScopedHandleRegistry<Handle>();
  let nextMessage = 1;
  let profilesHydrated = false;
  let restoredProfiles: boolean | undefined;
  const profileWriter = createDeferredWriter({
    write: (serialized: string) => api.saveAgents(JSON.parse(serialized)),
    onError: (error) =>
      console.error(`[pi-agent] could not save projects: ${String(error)}`),
    equals: Object.is,
  });

  void api
    .listAgents()
    .then(async (profiles) => {
      if (profiles.length > 0) {
        restoredProfiles = true;
        const restored = profiles.map(restoreAgentWorkspace);
        setAgents(restored);
        setLastActiveId(restored[0].id);
      } else {
        restoredProfiles = false;
        const cwd = await api.defaultWorkspace("agent-1");
        setAgents((current) =>
          current.map((agent) =>
            agent.id === "agent-1" && !agent.cwd ? { ...agent, cwd } : agent,
          ),
        );
      }
    })
    .catch((error) => {
      console.error(
        `[pi-agent] could not prepare the default workspace: ${String(error)}`,
      );
    })
    .finally(() => {
      profilesHydrated = true;
      const serialized = JSON.stringify(agents().map(agentProfile));
      if (restoredProfiles === true) profileWriter.prime(serialized);
      else if (restoredProfiles === false) profileWriter.schedule(serialized);
    });

  createEffect(
    () => JSON.stringify(agents().map(agentProfile)),
    (serialized) => {
      if (!profilesHydrated) return;
      profileWriter.schedule(serialized);
    },
  );
  const activeId = () => {
    const routeId = params().agentId;
    return routeId && agents().some((agent) => agent.id === routeId)
      ? routeId
      : lastActiveId();
  };
  const active = () =>
    agents().find((agent) => agent.id === activeId()) ?? agents()[0];
  const workspaceInfo = createAsyncQuery({
    source: () => active().cwd.trim() || undefined,
    // Repository metadata is decorative; losing it must not replace the app's
    // root error boundary. Queries that affect the page should instead render
    // through a local <Errored> boundary.
    load: (cwd) => api.workspaceInfo(cwd).catch(() => undefined),
  });
  createEffect(
    () => workspaceInfo.latest(),
    (info) => {
      if (info) setWorkspaceRevision((revision) => revision + 1);
    },
  );
  const disposeTerminal = () => {
    setTerminalOpen(false);
    setTerminalMounted(false);
    setTerminalOwnerId(undefined);
  };
  createEffect(
    () => ({
      activeId: activeId(),
      mounted: terminalMounted(),
      ownerId: terminalOwnerId(),
    }),
    ({ activeId, mounted, ownerId }) => {
      if (mounted && ownerId !== activeId) disposeTerminal();
    },
  );
  const activeSession = () =>
    sessions().find(
      (session) =>
        session.agentId === activeId() &&
        session.sessionId === active().state.sessionId,
    );
  const activeSessionId = () => params().sessionId;
  const itemHandleScope = () =>
    `${activeId()}\0${active().state.sessionId ?? activeSessionId() ?? ""}`;
  createEffect(
    () => ({
      scope: itemHandleScope(),
      ids: active().state.items.map((item) => item.id),
    }),
    ({ scope, ids }) => itemHandles.synchronize(scope, ids),
  );
  const refreshWorkspaceInfo = () => void workspaceInfo.refresh();
  let previousSearchScope: { agentId: string; sessionId: string } | undefined;
  createEffect(
    () => ({
      agentId: activeId(),
      sessionId: active().state.sessionId ?? activeSessionId() ?? "",
    }),
    (nextScope) => {
      const previous = previousSearchScope;
      previousSearchScope = nextScope;
      pendingFork.retainOwner(nextScope.agentId);
      // Assigning the first durable ID to an already-visible conversation is
      // synchronization, not navigation. A prior durable ID disappearing or
      // changing does represent a new conversation and closes scoped UI.
      if (
        !previous ||
        (previous.agentId === nextScope.agentId &&
          (!previous.sessionId || previous.sessionId === nextScope.sessionId))
      ) {
        return;
      }
      setSearchOpen(false);
      setActiveSearchItem(undefined);
    },
  );
  const draft = () => readAgentDraft(drafts(), activeId(), activeSessionId());
  const setDraft = (value: string) =>
    setDrafts((current) =>
      writeAgentDraft(current, activeId(), activeSessionId(), value),
    );
  const images = () =>
    readAgentDraftList(draftImages(), activeId(), activeSessionId());
  const setImages = (paths: readonly string[]) =>
    setDraftImages((current) =>
      writeAgentDraftList(current, activeId(), activeSessionId(), paths),
    );
  const contextFiles = () =>
    readAgentDraftList(draftContext(), activeId(), activeSessionId());
  const setContextFiles = (paths: readonly string[]) =>
    setDraftContext((current) =>
      writeAgentDraftList(current, activeId(), activeSessionId(), paths),
    );
  createEffect(
    () => params().agentId,
    (routeId) => {
      if (routeId && agents().some((agent) => agent.id === routeId)) {
        setLastActiveId(routeId);
      }
    },
  );
  const updateAgent = (
    id: string,
    update: (agent: AgentWorkspace) => AgentWorkspace,
  ) =>
    setAgents((current) =>
      current.map((agent) => (agent.id === id ? update(agent) : agent)),
    );
  const patchActive = (patch: Partial<AgentWorkspace>) => {
    const id = active().id;
    updateAgent(id, (agent) => ({ ...agent, ...patch }));
  };
  const prepareDefaultWorkspace = (id: string) => {
    void api
      .defaultWorkspace(id)
      .then((cwd) => {
        updateAgent(id, (agent) => (agent.cwd ? agent : { ...agent, cwd }));
      })
      .catch((error) => {
        console.error(
          `[pi-agent] could not prepare the default workspace: ${String(error)}`,
        );
      });
  };

  const unsubscribe = api.subscribe(
    (events: readonly Record<string, unknown>[]) => {
      const grouped = new Map<string, Record<string, unknown>[]>();
      for (const event of events) {
        const id =
          typeof event.agentId === "string" ? event.agentId : "agent-1";
        const group = grouped.get(id) ?? [];
        group.push(event);
        grouped.set(id, group);
        const dialog = parseExtensionUiRequest(event);
        if (dialog) {
          setExtensionDialogs((current) =>
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
          setExtensionDialogs((current) =>
            current.filter((candidate) => candidate.agentId !== id),
          );
        }
      }
      for (const [id, batch] of grouped) {
        updateAgent(id, (agent) => ({
          ...agent,
          state: reducePiEvents(agent.state, batch),
        }));
        if (
          batch.some(
            (event) =>
              event.type === "response" &&
              event.command === "get_state" &&
              event.success === true,
          )
        ) {
          const stateEvent = batch.find(
            (event) =>
              event.type === "response" &&
              event.command === "get_state" &&
              event.success === true,
          );
          if (
            stateEvent?.id === "wabou-bootstrap-state" ||
            stateEvent?.id === "wabou-new-session-state" ||
            stateEvent?.id === "wabou-clone-state"
          ) {
            void api.getMessages(id);
          }
          void api.getSessionStats(id);
          void api.getCommands(id);
          void api.getModelOptions(id);
          void api
            .listSessions(id)
            .then((next) =>
              setSessions((current) => [
                ...current.filter((session) => session.agentId !== id),
                ...next,
              ]),
            );
          const data = stateEvent?.data as
            | Record<string, unknown>
            | null
            | undefined;
          if (
            (stateEvent?.id === "wabou-new-session-state" ||
              stateEvent?.id === "wabou-clone-state") &&
            id === activeId() &&
            typeof data?.sessionId === "string"
          ) {
            void navigate({ to: `/agents/${id}/sessions/${data.sessionId}` });
          }
        }
        if (batch.some((event) => event.type === "agent_settled")) {
          void api.getSessionStats(id);
          void api.getForkMessages(id);
          if (id === activeId()) refreshWorkspaceInfo();
        }
        if (
          batch.some(
            (event) =>
              event.type === "response" &&
              event.command === "compact" &&
              event.success === true,
          )
        ) {
          void api.getMessages(id);
          void api.getSessionStats(id);
        }
        const exportEvent = batch.find(
          (event) =>
            event.type === "response" &&
            event.command === "export_html" &&
            event.success === true,
        );
        const exported = exportEvent?.data as
          | Record<string, unknown>
          | undefined;
        if (typeof exported?.path === "string") {
          toasts.success(i18n.message(m.export_complete, {}), {
            description: i18n.message(m.export_complete_detail, {
              path: exported.path,
            }),
          });
        }
        if (
          batch.some(
            (event) =>
              event.type === "response" &&
              event.command === "get_messages" &&
              event.success === true,
          )
        ) {
          void api.getForkMessages(id);
        }
        const forkEvent = batch.find(
          (event) =>
            event.type === "response" &&
            event.command === "fork" &&
            event.success === true,
        );
        const forkData = forkEvent?.data as Record<string, unknown> | undefined;
        if (forkEvent && forkData?.cancelled !== true) {
          if (id === activeId() && typeof forkData?.text === "string") {
            setDraft(forkData.text);
          }
          void api.getMessages(id);
          void api.getSessionStats(id);
        }
      }
    },
  );
  const unsubscribeExtensionUi = api.subscribe((events) => {
    for (const event of events) {
      const id = typeof event.agentId === "string" ? event.agentId : "agent-1";
      dispatchExtensionUiEffect(event, {
        notify: (effect) => {
          const notificationKey = `${effect.agentId}\0${effect.id}`;
          if (deliveredNotifications.has(notificationKey)) return;
          deliveredNotifications.add(notificationKey);
          const title =
            agents().find((agent) => agent.id === effect.agentId)?.name ??
            effect.agentId;
          const input = { description: effect.message };
          if (effect.tone === "error") toasts.error(title, input);
          else if (effect.tone === "warning") toasts.warning(title, input);
          else toasts.show({ title, ...input });
        },
        status: (effect) =>
          setExtensionStatuses((current) =>
            reduceExtensionUiStatuses(current, effect),
          ),
        widget: (effect) =>
          setExtensionWidgets((current) =>
            reduceExtensionUiWidgets(current, effect),
          ),
        title: (effect) =>
          setExtensionTitles((current) => ({
            ...current,
            [effect.agentId]: effect.title,
          })),
        editorText: (effect) => {
          const sessionId = agents().find(
            (agent) => agent.id === effect.agentId,
          )?.state.sessionId;
          setDrafts((current) =>
            writeAgentDraft(current, effect.agentId, sessionId, effect.text),
          );
        },
      });
      if (event.type !== "process_exit") continue;
      setExtensionStatuses((current) =>
        current.filter((candidate) => candidate.agentId !== id),
      );
      setExtensionWidgets((current) =>
        current.filter((candidate) => candidate.agentId !== id),
      );
      setExtensionTitles((current) => {
        if (!(id in current)) return current;
        const next = { ...current };
        delete next[id];
        return next;
      });
    }
  });
  onCleanup(() => {
    unsubscribe();
    unsubscribeExtensionUi();
    itemHandles.clear();
    profileWriter.flush();
  });

  createEffect(
    () =>
      agents()
        .map((agent) => agent.id)
        .join("\0"),
    (agentIds) => {
      for (const id of agentIds.split("\0").filter(Boolean)) {
        void api
          .listSessions(id)
          .then((next) =>
            setSessions((current) => [
              ...current.filter((session) => session.agentId !== id),
              ...next,
            ]),
          );
        void api
          .getStatus(id)
          .then((status) => {
            updateAgent(id, (agent) => ({
              ...agent,
              cwd: status.cwd ?? agent.cwd,
              state: {
                ...agent.state,
                connection: reconcileProcessConnection(
                  agent.state.connection,
                  status.running,
                ),
                error: status.error,
              },
            }));
          })
          .catch((error) => {
            updateAgent(id, (agent) => ({
              ...agent,
              state: {
                ...agent.state,
                connection: "failed",
                error: String(error),
              },
            }));
          });
      }
    },
  );

  const start = async (
    agent: AgentWorkspace = active(),
    sessionId?: string,
  ): Promise<boolean> => {
    try {
      const status = await api.start({
        agentId: agent.id,
        cwd: agent.cwd,
        proxy: defaults.value().proxy.trim() || undefined,
        noProxy: defaults.value().noProxy.trim() || undefined,
        provider:
          agent.provider.trim() ||
          defaults.value().provider.trim() ||
          undefined,
        model: agent.model.trim() || defaults.value().model.trim() || undefined,
        sessionId,
        subagentsEnabled: defaults.value().subagentsEnabled,
      });
      updateAgent(agent.id, (current) => ({
        ...current,
        cwd: status.cwd ?? current.cwd,
        state: { ...current.state, connection: "ready", error: undefined },
      }));
      return true;
    } catch (error) {
      updateAgent(agent.id, (current) => ({
        ...current,
        state: {
          ...current.state,
          connection: "failed",
          error: String(error),
        },
      }));
      return false;
    }
  };

  const submit = async () => {
    const message = draft().trim();
    const attachedImages = images();
    const attachedContext = contextFiles();
    const agent = active();
    if (!message) return;
    const queueing = agent.state.connection === "running";
    if (agent.state.connection !== "ready" && !(await start())) return;
    setDraft("");
    setImages([]);
    setContextFiles([]);
    updateAgent(agent.id, (current) => ({
      ...current,
      state: appendUserMessage(
        current.state,
        `user-${nextMessage++}`,
        message,
        queueing,
        attachedImages.map(imageFileName),
        attachedContext,
      ),
    }));
    try {
      await (queueing
        ? deliveryMode() === "steer"
          ? api.steer(agent.id, message, attachedImages, attachedContext)
          : api.followUp(agent.id, message, attachedImages, attachedContext)
        : api.prompt(agent.id, message, attachedImages, attachedContext));
    } catch (error) {
      setDraft(message);
      setImages(attachedImages);
      setContextFiles(attachedContext);
      updateAgent(agent.id, (current) => ({
        ...current,
        state: reducePiEvent(current.state, {
          type: "bridge_error",
          message: String(error),
        }),
      }));
    }
  };

  const addAgent = () => {
    const nextIndex =
      Math.max(
        0,
        ...agents().map(
          (agent) => Number(agent.id.match(/^agent-(\d+)$/)?.[1]) || 0,
        ),
      ) + 1;
    const agent = createAgentWorkspace(nextIndex);
    setAgents((current) => [...current, agent]);
    setLastActiveId(agent.id);
    void navigate({ to: `/agents/${agent.id}` });
    prepareDefaultWorkspace(agent.id);
  };

  const deleteActiveAgent = async () => {
    const removed = active();
    try {
      await api.deleteAgent(removed.id);
    } catch (error) {
      updateAgent(removed.id, (agent) => ({
        ...agent,
        state: reducePiEvent(agent.state, {
          type: "bridge_error",
          message: String(error),
        }),
      }));
      return;
    }
    setSessions((current) =>
      current.filter((session) => session.agentId !== removed.id),
    );
    setDrafts((current) => removeAgentDrafts(current, removed.id));
    setDraftImages((current) => removeAgentDraftLists(current, removed.id));
    setDraftContext((current) => removeAgentDraftLists(current, removed.id));
    setExtensionDialogs((current) =>
      current.filter((request) => request.agentId !== removed.id),
    );
    setExtensionStatuses((current) =>
      current.filter((status) => status.agentId !== removed.id),
    );
    setExtensionWidgets((current) =>
      current.filter((widget) => widget.agentId !== removed.id),
    );
    setExtensionTitles((current) => {
      if (!(removed.id in current)) return current;
      const next = { ...current };
      delete next[removed.id];
      return next;
    });
    const remaining = agents().filter((agent) => agent.id !== removed.id);
    const next =
      remaining[0] ??
      createAgentWorkspace(
        Math.max(
          0,
          ...agents().map(
            (agent) => Number(agent.id.match(/^agent-(\d+)$/)?.[1]) || 0,
          ),
        ) + 1,
      );
    setAgents(remaining.length > 0 ? remaining : [next]);
    setLastActiveId(next.id);
    await navigate({ to: `/agents/${next.id}` });
    if (!next.cwd) prepareDefaultWorkspace(next.id);
  };

  const selectAgent = (id: string) => {
    setLastActiveId(id);
    void navigate({ to: `/agents/${id}` });
  };

  const selectSession = async (agentId: string, sessionId: string) => {
    setLastActiveId(agentId);
    await navigate({ to: `/agents/${agentId}/sessions/${sessionId}` });
  };

  let openingSession = "";
  createEffect(
    () => {
      const { agentId, sessionId } = params();
      const session = sessions().find(
        (candidate) =>
          candidate.agentId === agentId && candidate.sessionId === sessionId,
      );
      const agent = agents().find((candidate) => candidate.id === agentId);
      return session && agent ? { session, agent } : undefined;
    },
    (target) => {
      if (!target) return;
      const key = `${target.agent.id}\0${target.session.sessionId}`;
      if (
        openingSession === key ||
        target.agent.state.sessionId === target.session.sessionId
      )
        return;
      openingSession = key;
      const next = {
        ...target.agent,
        cwd: target.session.cwd || target.agent.cwd,
      };
      updateAgent(target.agent.id, () => next);
      void start(next, target.session.sessionId).finally(() => {
        openingSession = "";
      });
    },
  );

  const respondToExtension = async (
    request: ExtensionUiDialogRequest,
    answer: ExtensionUiAnswer,
  ) => {
    setExtensionDialogs((current) =>
      current.filter(
        (candidate) =>
          candidate.agentId !== request.agentId || candidate.id !== request.id,
      ),
    );
    try {
      await api.respondExtensionUi(request.agentId, {
        id: request.id,
        ...answer,
      });
    } catch (error) {
      updateAgent(request.agentId, (agent) => ({
        ...agent,
        state: reducePiEvent(agent.state, {
          type: "bridge_error",
          message: String(error),
        }),
      }));
    }
  };

  const exportActiveSession = async () => {
    const agent = active();
    const path = await dialog.save({
      title: i18n.message(m.export_session, {}),
      defaultName: `${activeSession()?.name ?? agent.state.sessionName ?? "pi-session"}.html`,
      filters: [{ name: "HTML", extensions: ["html"] }],
    });
    if (path) await api.exportSession(agent.id, path);
  };

  return (
    <View class="w-full h-full min-w-0 min-h-0 flex bg-canvas text-primary">
      <Sidebar
        agents={agents()}
        activeId={activeId()}
        select={selectAgent}
        add={addAgent}
        newSession={() => void api.newSession(active().id)}
        canCreateSession={active().state.connection === "ready"}
        openSettings={() => navigate({ to: "/settings" })}
        sessions={sessions()}
        selectSession={(agentId, sessionId) =>
          void selectSession(agentId, sessionId)
        }
      />

      <Show
        when={location().pathname !== "/settings"}
        fallback={
          <SettingsPage
            app={defaults.value()}
            project={active()}
            state={active().state}
            updateApp={defaults.update}
            updateProject={patchActive}
            deleteProject={() => void deleteActiveAgent()}
            setAutoCompaction={(enabled) =>
              void api.setAutoCompaction(active().id, enabled)
            }
            setSteeringMode={(mode) =>
              void api.setSteeringMode(active().id, mode)
            }
            setFollowUpMode={(mode) =>
              void api.setFollowUpMode(active().id, mode)
            }
            close={() => navigate({ to: `/agents/${activeId()}` })}
          />
        }
      >
        <View class="flex-1 min-w-0 min-h-0 flex flex-col">
          <View class="h-14 flex-none px-5 border-b border-subtle bg-surface flex items-center justify-between gap-3">
            <View class="min-w-0 flex-1 overflow-hidden flex flex-row items-center gap-2">
              <View class="min-w-0 flex-1 overflow-hidden flex flex-col gap-0">
                <Text class="font-semibold">
                  {activeSession()?.name ??
                    active().state.sessionName ??
                    active().name}
                </Text>
                <View class="min-w-0 flex flex-row items-center gap-2">
                  <Icon
                    source={folder}
                    size={12}
                    class="flex-none text-muted"
                  />
                  <Text class="max-w-64 truncate text-xs text-muted">
                    {active().cwd}
                  </Text>
                  <Show
                    when={
                      workspaceInfo.latest()?.repository &&
                      workspaceInfo.latest()?.branch
                    }
                  >
                    <Text class="flex-none text-xs text-muted">·</Text>
                    <View class="flex-none flex flex-row items-center gap-1">
                      <Icon source={gitBranch} size={11} class="text-muted" />
                      <Text class="max-w-32 truncate text-xs text-muted">
                        {workspaceInfo.latest()?.branch}
                      </Text>
                      <Show
                        when={(workspaceInfo.latest()?.changedFiles ?? 0) > 0}
                      >
                        <Text class="text-xs text-warning-primary">
                          {i18n.message(m.changed_files, {
                            count: workspaceInfo.latest()?.changedFiles ?? 0,
                          })}
                        </Text>
                      </Show>
                    </View>
                  </Show>
                  <Show
                    when={
                      active().state.connection === "ready" ||
                      active().state.connection === "running"
                    }
                  >
                    <Text class="flex-none text-xs text-muted">·</Text>
                    <Text class="min-w-0 flex-1 truncate text-xs text-muted">
                      {(active().state.model ?? active().model) ||
                        i18n.message(m.no_model, {})}{" "}
                      ·{" "}
                      {i18n.message(m.thinking, {
                        level: active().state.thinking ?? "default",
                      })}
                    </Text>
                    <AgentActivityStatus state={active().state} />
                  </Show>
                </View>
              </View>
              <Show when={active().state.sessionId}>
                <SessionTitle
                  name={
                    activeSession()?.name ?? active().state.sessionName ?? ""
                  }
                  rename={(name) => api.renameSession(active().id, name)}
                />
              </Show>
            </View>
            <Show
              when={
                active().state.connection === "ready" ||
                active().state.connection === "running"
              }
            >
              <View class="min-w-0 flex-none overflow-hidden flex flex-row items-center gap-1">
                <Button
                  variant="ghost"
                  size="icon"
                  aria-label="Toggle terminal"
                  aria-pressed={terminalOpen()}
                  disabled={!active().cwd.trim()}
                  onClick={() => {
                    if (!terminalOpen()) {
                      setTerminalOwnerId(activeId());
                      setTerminalMounted(true);
                    }
                    setTerminalOpen((open) => !open);
                  }}
                >
                  <Icon source={squareTerminal} size={15} />
                </Button>
                <Button
                  variant="ghost"
                  size="icon"
                  aria-label={i18n.message(m.workspace_files, {})}
                  aria-pressed={sidePanel() === "files"}
                  onClick={() =>
                    setSidePanel((current) =>
                      current === "files" ? undefined : "files",
                    )
                  }
                >
                  <Icon source={folder} size={15} />
                </Button>
                <Show when={workspaceInfo.latest()?.repository}>
                  <Button
                    variant="ghost"
                    size="icon"
                    aria-label={i18n.message(m.code_changes, {})}
                    aria-pressed={sidePanel() === "changes"}
                    onClick={() =>
                      setSidePanel((current) =>
                        current === "changes" ? undefined : "changes",
                      )
                    }
                  >
                    <Icon source={gitBranch} size={15} />
                  </Button>
                </Show>
                <Show when={active().state.connection === "ready"}>
                  <ModelControls
                    models={active().state.models}
                    modelProvider={active().state.modelProvider}
                    modelId={active().state.modelId}
                    thinking={active().state.thinking}
                    thinkingLevels={active().state.availableThinkingLevels}
                    chooseModel={(provider, modelId) =>
                      void api.setModel(active().id, provider, modelId)
                    }
                    chooseThinking={(level) =>
                      void api.setThinking(active().id, level)
                    }
                  />
                </Show>
                <Button
                  variant="ghost"
                  size="icon"
                  aria-label={i18n.message(m.search_transcript, {})}
                  disabled={active().state.items.length === 0}
                  aria-pressed={searchOpen()}
                  onClick={() => setSearchOpen((open) => !open)}
                >
                  <Icon source={search} size={15} />
                </Button>
                <Show when={active().state.connection === "ready"}>
                  <Button
                    variant="ghost"
                    size="icon"
                    aria-label={i18n.message(m.new_session, {})}
                    onClick={() => void api.newSession(active().id)}
                  >
                    <Icon source={filePlus} size={15} />
                  </Button>
                  <SessionActions
                    disabled={!active().state.sessionId}
                    compact={() => void api.compactSession(active().id)}
                    clone={() => void api.cloneSession(active().id)}
                    exportHtml={() => void exportActiveSession()}
                  />
                </Show>
                <Show when={active().state.connection === "running"}>
                  <Button
                    variant="outline"
                    onClick={() => void api.abort(active().id)}
                  >
                    <Icon source={square} size={12} />{" "}
                    {i18n.message(m.stop, {})}
                  </Button>
                </Show>
              </View>
            </Show>
          </View>

          <Show
            when={
              active().cwd.trim() &&
              (active().state.connection === "ready" ||
                active().state.connection === "running")
            }
            fallback={
              <WorkspaceSetup
                path={active().cwd}
                error={active().state.error}
                runtimeLogs={active().state.runtimeLogs}
                provider={active().provider}
                model={active().model}
                proxy={defaults.value().proxy}
                updatePath={(cwd) => patchActive({ cwd })}
                start={start}
                openSettings={() => void navigate({ to: "/settings" })}
              />
            }
          >
            <MessageScroller class="flex-1 min-h-0" followEnd>
              <Show when={searchOpen()}>
                <TranscriptSearch
                  items={active().state.items}
                  resolveItem={(id) =>
                    itemHandles.resolve(itemHandleScope(), id)
                  }
                  activeChanged={setActiveSearchItem}
                  close={() => setSearchOpen(false)}
                />
              </Show>
              <MessageScrollerViewport>
                <MessageScrollerContent class="max-w-4xl mx-auto p-5">
                  <Show
                    when={active().state.items.length > 0}
                    fallback={
                      <ConversationWelcome
                        workspace={active().cwd}
                        choosePrompt={setDraft}
                      />
                    }
                  >
                    <ConversationList
                      items={active().state.items}
                      activeSearchItem={activeSearchItem()}
                      registerItem={(id, node) =>
                        itemHandles.register(itemHandleScope(), id, node)
                      }
                      fork={(item) =>
                        pendingFork.open(activeId(), {
                          entryId: item.entryId ?? "",
                          text: item.text,
                        })
                      }
                    />
                  </Show>
                </MessageScrollerContent>
              </MessageScrollerViewport>
              <MessageScrollerButton />
            </MessageScroller>

            <View class="flex-none border-t border-subtle bg-surface p-4">
              <View
                data-wabou-owns="surface focus-ring"
                class="max-w-4xl mx-auto min-w-0 rounded-xl border border-strong bg-input shadow-sm p-2 gap-2"
              >
                <ExtensionUiChrome
                  statuses={extensionStatuses().filter(
                    (status) => status.agentId === activeId(),
                  )}
                  widgets={extensionWidgets().filter(
                    (widget) => widget.agentId === activeId(),
                  )}
                  placement="aboveEditor"
                />
                <ComposerImages paths={images()} change={setImages} />
                <ComposerContextFiles
                  paths={contextFiles()}
                  change={setContextFiles}
                />
                <TextArea
                  chrome="none"
                  class="h-20"
                  value={draft()}
                  aria-label={i18n.message(m.prompt_placeholder, {})}
                  placeholder={
                    active().state.connection === "running"
                      ? i18n.message(m.queue_follow_up, {})
                      : i18n.message(m.prompt_placeholder, {})
                  }
                  onInput={(event) => setDraft(event.currentTarget.value)}
                  onKeyDown={(event) => {
                    if (event.key === "Enter" && (event.mods & 1) === 0) {
                      event.preventDefault();
                      void submit();
                    }
                  }}
                />
                <ExtensionUiChrome
                  statuses={[]}
                  widgets={extensionWidgets().filter(
                    (widget) => widget.agentId === activeId(),
                  )}
                  placement="belowEditor"
                />
                <View class="flex items-center justify-between gap-3 px-1">
                  <View class="min-w-0 flex flex-row items-center gap-3">
                    <ComposerImagePicker paths={images()} change={setImages} />
                    <WorkspaceContextPicker
                      cwd={active().cwd}
                      paths={contextFiles()}
                      change={setContextFiles}
                      loadFiles={api.listWorkspaceFiles}
                    />
                    <CommandPicker
                      commands={active().state.commands}
                      choose={setDraft}
                    />
                    <Show when={active().state.connection === "running"}>
                      <ComposerDeliveryControl
                        value={deliveryMode()}
                        change={setDeliveryMode}
                      />
                    </Show>
                    <Text class="flex-none text-xs text-muted">
                      {i18n.message(m.send_hint, {})}
                    </Text>
                    <SessionUsage stats={active().state.stats} />
                  </View>
                  <Button
                    disabled={!draft().trim()}
                    onClick={() => void submit()}
                  >
                    <Icon source={send} size={14} />{" "}
                    {active().state.connection === "running"
                      ? i18n.message(m.queue, {})
                      : i18n.message(m.send, {})}
                  </Button>
                </View>
              </View>
            </View>
          </Show>
          <Show when={terminalMounted()}>
            <AgentTerminalPanel
              cwd={active().cwd}
              open={terminalOpen()}
              close={() => setTerminalOpen(false)}
              dispose={disposeTerminal}
            />
          </Show>
        </View>
        <Show when={sidePanel() === "files" && active().cwd.trim()}>
          <WorkspacePanel
            cwd={active().cwd}
            loadFiles={api.listWorkspaceFiles}
            readFile={api.readWorkspaceFile}
            addContext={(path) =>
              setContextFiles([...new Set([...contextFiles(), path])])
            }
            close={() => setSidePanel(undefined)}
          />
        </Show>
        <Show when={sidePanel() === "changes" && active().cwd.trim()}>
          <WorkspaceChangesPanel
            cwd={active().cwd}
            revision={workspaceRevision()}
            load={api.workspaceChanges}
            close={() => setSidePanel(undefined)}
          />
        </Show>
      </Show>
      <SessionForkDialog
        open={pendingFork.value() !== undefined}
        cancel={pendingFork.close}
        confirm={() => {
          const target = pendingFork.value();
          if (target) void api.fork(target.ownerId, target.data.entryId);
          pendingFork.close();
        }}
      />
      <ExtensionUiDialog
        request={extensionDialogs()[0]}
        respond={(answer) => {
          const request = extensionDialogs()[0];
          if (request) void respondToExtension(request, answer);
        }}
      />
      <ExtensionWindowTitle
        title={extensionTitles()[activeId()] || "Pi Agent · Wabou"}
      />
      <Toaster toasts={toasts} />
    </View>
  );
}
