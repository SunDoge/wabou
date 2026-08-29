import {
  type CommandItem,
  createAsyncQuery,
  createShortcuts,
  createToasts,
  currentWindow,
  type Handle,
  MessageScroller,
  MessageScrollerButton,
  MessageScrollerContent,
  MessageScrollerViewport,
  Toaster,
  useDialog,
  useLocation,
  useNavigate,
  useParams,
  Workbench,
  WorkbenchMain,
} from "@wabou/ui";
import {
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  Show,
} from "solid-js";
import {
  appendUserMessage,
  reconcileProcessConnection,
  reducePiEvent,
  reducePiEvents,
} from "./agent-state";
import { type PiSession, usePiApi } from "./api";
import { AppCommandPalette } from "./app-command-palette";
import type { ComposerDeliveryMode } from "./composer-delivery";
import { imageFileName } from "./composer-images";
import { ConversationList } from "./conversation";
import { ConversationComposer } from "./conversation-composer";
import { ConversationHeader } from "./conversation-header";
import { ConversationNavigator } from "./conversation-navigator";
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
import { createOwnedOverlay } from "./owned-overlay";
import { createPersistedRecord } from "./persisted-record";
import { ScopedHandleRegistry } from "./scoped-handle-registry";
import { SessionForkDialog } from "./session-fork";
import { SessionTitle } from "./session-title";
import { type AppSettings, SettingsPage } from "./settings";
import { Sidebar } from "./sidebar";
import { AgentTerminalPanel } from "./terminal-panel";
import { TranscriptSearch } from "./transcript-search";
import {
  type AgentWorkspace,
  agentProfile,
  createAgentWorkspace,
  resolveActiveAgentId,
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
  const [commandPaletteOpen, setCommandPaletteOpen] = createSignal(false);
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
    return (
      resolveActiveAgentId(agents(), params().agentId, lastActiveId()) ?? ""
    );
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

  // Status updates replace agent objects. Keep the effect keyed by the stable ID
  // value so those updates cannot start another status-polling cycle.
  const agentIds = createMemo(() =>
    agents()
      .map((agent) => agent.id)
      .join("\0"),
  );
  createEffect(agentIds, (agentIds) => {
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
  });

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
    if (agents().length <= 1) return;
    const removed = active();
    const before = agents();
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
    const remaining = before.filter((agent) => agent.id !== removed.id);
    const next = remaining[0];
    if (!next) return;
    // Publish the replacement identity before removing the old object. The
    // host event flush commits these consecutive writes together, and the
    // settings route has no agent parameter, so resolveActiveAgentId selects
    // `next` without exposing an absent project identity.
    setLastActiveId(next.id);
    setAgents(remaining);
    // Keep the project settings surface open. Navigating while the alert
    // dialog's click event is still unwinding can recursively re-enter Router
    // Core; `/settings` already resolves the newly published active identity.
    // Historical sessions are harmless here because every read is scoped by
    // the surviving project IDs. Remove them after the dialog event has fully
    // unwound so keyed sidebar reconciliation never runs inside its own click.
    queueMicrotask(() => {
      setSessions((current) =>
        current.filter((session) => session.agentId !== removed.id),
      );
    });
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
      const nextTitles = { ...current };
      delete nextTitles[removed.id];
      return nextTitles;
    });
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

  const toggleTerminal = () => {
    if (!active().cwd.trim()) return;
    if (!terminalOpen()) {
      setTerminalOwnerId(activeId());
      setTerminalMounted(true);
    }
    setTerminalOpen((open) => !open);
  };
  const toggleFiles = () =>
    setSidePanel((current) => (current === "files" ? undefined : "files"));
  const toggleChanges = () =>
    setSidePanel((current) => (current === "changes" ? undefined : "changes"));
  const startNewSession = () => {
    if (active().state.connection === "ready") {
      void api.newSession(active().id);
    }
  };
  const applicationCommands = createMemo<readonly CommandItem[]>(() => [
    {
      id: "new-session",
      label: i18n.message(m.new_session, {}),
      description: i18n.message(m.command_new_session_detail, {}),
      shortcut: "⌘/Ctrl N",
      disabled: active().state.connection !== "ready",
      keywords: ["thread", "conversation"],
      onSelect: startNewSession,
    },
    {
      id: "search-conversation",
      label: i18n.message(m.search_transcript, {}),
      description: i18n.message(m.command_search_detail, {}),
      shortcut: "⌘/Ctrl F",
      disabled: active().state.items.length === 0,
      keywords: ["find", "transcript"],
      onSelect: () => setSearchOpen(true),
    },
    {
      id: "toggle-terminal",
      label: i18n.message(m.command_terminal, {}),
      description: i18n.message(m.command_terminal_detail, {}),
      shortcut: "⌘/Ctrl J",
      disabled: !active().cwd.trim(),
      keywords: ["shell", "console"],
      onSelect: toggleTerminal,
    },
    {
      id: "workspace-files",
      label: i18n.message(m.workspace_files, {}),
      description: i18n.message(m.command_files_detail, {}),
      disabled: !active().cwd.trim(),
      keywords: ["project", "browse"],
      onSelect: toggleFiles,
    },
    {
      id: "code-changes",
      label: i18n.message(m.code_changes, {}),
      description: i18n.message(m.command_changes_detail, {}),
      disabled: !workspaceInfo.latest()?.repository,
      keywords: ["git", "diff"],
      onSelect: toggleChanges,
    },
    {
      id: "settings",
      label: i18n.message(m.settings, {}),
      description: i18n.message(m.command_settings_detail, {}),
      shortcut: "⌘/Ctrl ,",
      keywords: ["preferences", "configuration"],
      onSelect: () => void navigate({ to: "/settings" }),
    },
  ]);
  const shortcuts = createShortcuts({
    "Primary+K": () => setCommandPaletteOpen(true),
    "Primary+N": startNewSession,
    "Primary+F": () => {
      if (active().state.items.length > 0) setSearchOpen(true);
    },
    "Primary+J": toggleTerminal,
    "Primary+,": () => navigate({ to: "/settings" }),
  });

  return (
    <Workbench {...shortcuts.bindings}>
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
          <WorkbenchMain>
            <SettingsPage
              app={defaults.value()}
              project={active()}
              state={active().state}
              canDeleteProject={agents().length > 1}
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
          </WorkbenchMain>
        }
      >
        <WorkbenchMain>
          <ConversationHeader
            project={active().name}
            branch={workspaceInfo.latest()?.branch}
            session={
              activeSession()?.name ??
              active().state.sessionName ??
              active().name
            }
            state={active().state}
            titleAction={
              <Show when={active().state.sessionId}>
                <SessionTitle
                  name={
                    activeSession()?.name ?? active().state.sessionName ?? ""
                  }
                  rename={(name) => api.renameSession(active().id, name)}
                />
              </Show>
            }
            cwdAvailable={Boolean(active().cwd.trim())}
            repository={Boolean(workspaceInfo.latest()?.repository)}
            terminalOpen={terminalOpen()}
            filesOpen={sidePanel() === "files"}
            changesOpen={sidePanel() === "changes"}
            searchOpen={searchOpen()}
            toggleTerminal={toggleTerminal}
            toggleFiles={toggleFiles}
            toggleChanges={toggleChanges}
            toggleSearch={() => setSearchOpen((open) => !open)}
            newSession={startNewSession}
            compactSession={() => void api.compactSession(active().id)}
            cloneSession={() => void api.cloneSession(active().id)}
            exportSession={() => void exportActiveSession()}
            abort={() => void api.abort(active().id)}
          />

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
                <MessageScrollerContent class="max-w-3xl mx-auto px-6 py-5">
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
              <ConversationNavigator items={active().state.items} />
              <MessageScrollerButton />
            </MessageScroller>

            <ConversationComposer
              connection={active().state.connection}
              project={active().name}
              cwd={active().cwd}
              draft={draft()}
              images={images()}
              contextFiles={contextFiles()}
              deliveryMode={deliveryMode()}
              models={active().state.models}
              modelProvider={active().state.modelProvider}
              modelId={active().state.modelId}
              thinking={active().state.thinking}
              thinkingLevels={active().state.availableThinkingLevels}
              commands={active().state.commands}
              stats={active().state.stats}
              statuses={extensionStatuses().filter(
                (status) => status.agentId === activeId(),
              )}
              widgets={extensionWidgets().filter(
                (widget) => widget.agentId === activeId(),
              )}
              changeDraft={setDraft}
              changeImages={setImages}
              changeContextFiles={setContextFiles}
              changeDeliveryMode={setDeliveryMode}
              chooseModel={(provider, modelId) =>
                void api.setModel(active().id, provider, modelId)
              }
              chooseThinking={(level) =>
                void api.setThinking(active().id, level)
              }
              loadWorkspaceFiles={api.listWorkspaceFiles}
              submit={() => void submit()}
            />
          </Show>
          <Show when={terminalMounted()}>
            <AgentTerminalPanel
              cwd={active().cwd}
              open={terminalOpen()}
              close={() => setTerminalOpen(false)}
              dispose={disposeTerminal}
            />
          </Show>
        </WorkbenchMain>
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
      <AppCommandPalette
        open={commandPaletteOpen()}
        items={applicationCommands()}
        close={() => setCommandPaletteOpen(false)}
        label={i18n.message(m.command_palette, {})}
        placeholder={i18n.message(m.command_palette_search, {})}
        emptyText={i18n.message(m.command_palette_empty, {})}
      />
      <Toaster toasts={toasts} />
    </Workbench>
  );
}
