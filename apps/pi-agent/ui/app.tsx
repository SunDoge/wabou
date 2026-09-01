import { openKv } from "@wabou/ui";
import {
  type CommandItem,
  createAsyncQuery,
  createKeyedAsyncAction,
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
  WorkbenchContentColumn,
  WorkbenchMain,
} from "@wabou/ui";
import {
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  Show,
} from "solid-js";
import { subscribeAgentEvents } from "./agent-event-subscription";
import { createAgentProfiles } from "./agent-profiles";
import {
  type AgentItem,
  appendUserMessage,
  reconcileProcessConnection,
  reducePiEvent,
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
import { createAgentDraftController } from "./drafts";
import {
  type ExtensionUiAnswer,
  ExtensionUiDialog,
  type ExtensionUiDialogRequest,
  type ExtensionUiStatus,
  type ExtensionUiWidget,
} from "./extension-ui";
import { subscribeExtensionUi } from "./extension-ui-subscription";
import { i18n, m } from "./i18n";
import { createOwnedOverlay } from "./owned-overlay";
import { createPersistedRecord } from "./persisted-record";
import { ScopedHandleRegistry } from "./scoped-handle-registry";
import { SessionForkDialog } from "./session-fork";
import { createSessionNavigation } from "./session-navigation";
import { SessionTitle } from "./session-title";
import { type AppSettings, SettingsPage } from "./settings";
import { Sidebar } from "./sidebar";
import { SkillsPage } from "./skills-page";
import { AgentTerminalPanel } from "./terminal-panel";
import { TranscriptSearch } from "./transcript-search";
import {
  type PreparedRewind,
  TurnCheckpointCoordinator,
} from "./turn-checkpoints";
import { type AgentWorkspace, createAgentWorkspace } from "./workspace";
import { WorkspaceChangesPanel } from "./workspace-changes-panel";
import { WorkspacePanel } from "./workspace-panel";
import { WorkspaceSetupBoundary } from "./workspace-setup";

function ExtensionWindowTitle(props: { title: string }) {
  createEffect(
    () => props.title,
    (title) => currentWindow().setTitle(title),
  );
  return null;
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
  const profiles = createAgentProfiles({
    api,
    routeAgentId: () => params().agentId,
  });
  const {
    agents,
    setAgents,
    active,
    activeId,
    setLastActiveId,
    updateAgent,
    patchActive,
    prepareDefaultWorkspace,
    workspacePending,
    loadError: projectLoadError,
    saveError: projectSaveError,
    reload: reloadProjects,
    retrySave: retryProjectSave,
  } = profiles;
  createEffect(projectLoadError, (error) => {
    if (!error) return;
    toasts.error(i18n.message(m.projects_load_failed, {}), {
      description: String(error),
      duration: 0,
      action: {
        label: i18n.message(m.retry, {}),
        onAction: () => void reloadProjects(),
      },
    });
  });
  createEffect(projectSaveError, (error) => {
    if (!error) return;
    toasts.error(i18n.message(m.projects_save_failed, {}), {
      description: String(error),
      duration: 0,
      action: {
        label: i18n.message(m.retry, {}),
        onAction: retryProjectSave,
      },
    });
  });
  const newSessionAction = createKeyedAsyncAction(
    (agentId: string) => agentId,
    (agentId: string) => api.newSession(agentId),
  );
  const abortAction = createKeyedAsyncAction(
    (agentId: string) => agentId,
    (agentId: string) => api.abort(agentId),
  );
  const activeSessionId = () => params().sessionId;
  const drafts = createAgentDraftController({
    kv: openKv(["pi-agent"]),
    activeAgentId: activeId,
    activeSessionId,
  });
  const [sessions, setSessions] = createSignal<readonly PiSession[]>([]);
  const sessionNavigation = createSessionNavigation();
  const [workspaceRevision, setWorkspaceRevision] = createSignal(0);
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
  const [forkCheckpoint, setForkCheckpoint] = createSignal<
    "checking" | "available" | "unavailable"
  >("unavailable");
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
  const itemHandles = new ScopedHandleRegistry<Handle>();
  const turnCheckpoints = new TurnCheckpointCoordinator(api);
  const pendingRewinds = new Map<
    string,
    { cwd: string; rewind: PreparedRewind }
  >();
  let nextMessage = 1;
  const workspaceInfo = createAsyncQuery({
    source: () => active().cwd.trim() || undefined,
    // Repository metadata is decorative; losing it must not replace the app's
    // root error boundary. Queries that affect the page should instead render
    // through a local <Errored> boundary.
    load: (cwd) => api.workspaceInfo(cwd).catch(() => undefined),
  });
  const openFork = (item: Extract<AgentItem, { kind: "user" }>) => {
    const agent = active();
    const entryId = item.entryId;
    if (!entryId) return;
    pendingFork.open(agent.id, { entryId, text: item.text });
    setForkCheckpoint("checking");
    void (async () => {
      try {
        const sessionId = agent.state.sessionId;
        const info = sessionId
          ? (workspaceInfo.latest() ?? (await api.workspaceInfo(agent.cwd)))
          : undefined;
        const checkpoint =
          sessionId && info?.repository
            ? await api.findCheckpoint(agent.cwd, sessionId, entryId)
            : undefined;
        const current = pendingFork.value();
        if (current?.ownerId === agent.id && current.data.entryId === entryId) {
          setForkCheckpoint(checkpoint ? "available" : "unavailable");
        }
      } catch (error) {
        console.warn(
          `[pi-agent] could not inspect turn checkpoint: ${String(error)}`,
        );
        const current = pendingFork.value();
        if (current?.ownerId === agent.id && current.data.entryId === entryId) {
          setForkCheckpoint("unavailable");
        }
      }
    })();
  };
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
  const draft = drafts.draft;
  const setDraft = drafts.setDraft;
  const images = drafts.images;
  const setImages = drafts.setImages;
  const contextFiles = drafts.contextFiles;
  const setContextFiles = drafts.setContextFiles;
  const pendingSubmissions = new Map<
    string,
    {
      agentId: string;
      sessionId?: string;
      message: string;
      images: readonly string[];
      contextFiles: readonly string[];
    }
  >();
  const unsubscribe = subscribeAgentEvents({
    api,
    activeAgentId: activeId,
    updateAgent,
    updateSessions: setSessions,
    updateDialogs: setExtensionDialogs,
    navigateToSession: (agentId, sessionId) =>
      void navigate({ to: `/agents/${agentId}/sessions/${sessionId}` }),
    refreshWorkspaceInfo,
    restoreForkDraft: setDraft,
    settleRequest: (agentId, requestId, error) => {
      const submission = pendingSubmissions.get(requestId);
      if (!submission || submission.agentId !== agentId) return;
      pendingSubmissions.delete(requestId);
      if (!error) return;
      turnCheckpoints.discard(requestId);
      drafts.restore(submission.agentId, submission.sessionId, {
        text: submission.message,
        images: submission.images,
        contextFiles: submission.contextFiles,
      });
    },
    settleFork: (agentId, error) => {
      const pending = pendingRewinds.get(agentId);
      if (!pending) return;
      pendingRewinds.delete(agentId);
      if (!error) return;
      void turnCheckpoints
        .rollback(pending.cwd, pending.rewind)
        .catch((rollbackError) =>
          toasts.error("Could not restore the safety checkpoint", {
            description: String(rollbackError),
          }),
        );
    },
    exported: (path) =>
      toasts.success(i18n.message(m.export_complete, {}), {
        description: i18n.message(m.export_complete_detail, { path }),
      }),
  });
  const unsubscribeExtensionUi = subscribeExtensionUi({
    api,
    agentName: (agentId) =>
      agents().find((agent) => agent.id === agentId)?.name ?? agentId,
    sessionId: (agentId) =>
      agents().find((agent) => agent.id === agentId)?.state.sessionId,
    notify: (effect, title) => {
      const input = { description: effect.message };
      if (effect.tone === "error") toasts.error(title, input);
      else if (effect.tone === "warning") toasts.warning(title, input);
      else toasts.show({ title, ...input });
    },
    updateStatuses: setExtensionStatuses,
    updateWidgets: setExtensionWidgets,
    updateTitles: setExtensionTitles,
    writeEditorText: drafts.setDraftFor,
  });
  onCleanup(() => {
    unsubscribe();
    unsubscribeExtensionUi();
    itemHandles.clear();
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

  createEffect(
    () => ({
      cwd: active().cwd,
      sessionId: active().state.sessionId,
      items: active().state.items,
    }),
    ({ cwd, sessionId, items }) =>
      turnCheckpoints.synchronize(cwd, sessionId, items),
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
    const userMessageId = `user-${nextMessage++}`;
    const submissionSessionId = activeSessionId();
    pendingSubmissions.set(userMessageId, {
      agentId: agent.id,
      sessionId: submissionSessionId,
      message,
      images: attachedImages,
      contextFiles: attachedContext,
    });
    setDraft("");
    setImages([]);
    setContextFiles([]);
    updateAgent(agent.id, (current) => ({
      ...current,
      state: appendUserMessage(
        current.state,
        userMessageId,
        message,
        queueing,
        attachedImages.map(imageFileName),
        attachedContext,
      ),
    }));
    // Commit the optimistic UI before checkpoint/network awaits. Keeping the
    // old native editor value alive across an async boundary lets a subsequent
    // input event append to an already submitted prompt.
    const repository = !queueing
      ? (workspaceInfo.latest() ??
        (await api.workspaceInfo(agent.cwd).catch(() => undefined)))
      : undefined;
    if (repository?.repository) {
      try {
        await turnCheckpoints.capture(userMessageId, agent.cwd, agent.id);
      } catch (error) {
        console.warn(
          `[pi-agent] could not capture turn checkpoint: ${String(error)}`,
        );
      }
    }
    try {
      await (queueing
        ? deliveryMode() === "steer"
          ? api.steer(
              agent.id,
              userMessageId,
              message,
              attachedImages,
              attachedContext,
            )
          : api.followUp(
              agent.id,
              userMessageId,
              message,
              attachedImages,
              attachedContext,
            )
        : api.prompt(
            agent.id,
            userMessageId,
            message,
            attachedImages,
            attachedContext,
          ));
    } catch (error) {
      pendingSubmissions.delete(userMessageId);
      turnCheckpoints.discard(userMessageId);
      drafts.restore(agent.id, submissionSessionId, {
        text: message,
        images: attachedImages,
        contextFiles: attachedContext,
      });
      updateAgent(agent.id, (current) => ({
        ...current,
        state: reducePiEvent(current.state, {
          type: "request_error",
          message: String(error),
          userMessageId,
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
    void prepareDefaultWorkspace(agent.id);
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
      throw error;
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
    sessionNavigation.removeAgent(removed.id);
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
    await drafts
      .removeAgent(removed.id)
      .catch((error) =>
        console.warn(
          `[pi-agent] could not remove persisted project drafts: ${String(error)}`,
        ),
      );
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
    if (!next.cwd) void prepareDefaultWorkspace(next.id);
  };

  const selectAgent = (id: string) => {
    setLastActiveId(id);
    void navigate({ to: `/agents/${id}` });
  };

  const selectSession = async (agentId: string, sessionId: string) => {
    setLastActiveId(agentId);
    await navigate({ to: `/agents/${agentId}/sessions/${sessionId}` });
  };

  createEffect(
    () => {
      const { agentId, sessionId } = params();
      return agentId && sessionId ? { agentId, sessionId } : undefined;
    },
    (target) => {
      if (target) sessionNavigation.visit(target);
    },
  );
  const traverseSessionHistory = (direction: "back" | "forward") => {
    const target =
      direction === "back"
        ? sessionNavigation.back()
        : sessionNavigation.forward();
    if (!target) return;
    setLastActiveId(target.agentId);
    void navigate({
      to: `/agents/${target.agentId}/sessions/${target.sessionId}`,
    });
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
    try {
      await api.respondExtensionUi(request.agentId, {
        id: request.id,
        ...answer,
      });
      setExtensionDialogs((current) =>
        current.filter(
          (candidate) =>
            candidate.agentId !== request.agentId ||
            candidate.id !== request.id,
        ),
      );
    } catch (error) {
      updateAgent(request.agentId, (agent) => ({
        ...agent,
        state: reducePiEvent(agent.state, {
          type: "bridge_error",
          message: String(error),
        }),
      }));
      throw error;
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
  const startNewSession = async () => {
    const agent = active();
    if (agent.state.connection !== "ready") return;
    const result = await newSessionAction.run(agent.id);
    if (!result.ok)
      toasts.error(i18n.message(m.new_session_failed, {}), {
        description: String(result.error),
      });
  };
  const abortActive = async () => {
    const agentId = activeId();
    const result = await abortAction.run(agentId);
    if (!result.ok)
      toasts.error(i18n.message(m.stop_failed, {}), {
        description: String(result.error),
      });
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
      id: "skills",
      label: i18n.message(m.skills, {}),
      description: i18n.message(m.command_skills_detail, {}),
      disabled: !active().cwd.trim(),
      keywords: ["skill", "prompt", "extension"],
      onSelect: () => void navigate({ to: "/skills" }),
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
  const activePage = (): "agents" | "skills" | "settings" => {
    switch (location().pathname) {
      case "/skills":
        return "skills";
      case "/settings":
        return "settings";
      default:
        return "agents";
    }
  };

  return (
    <Workbench {...shortcuts.bindings}>
      <Sidebar
        agents={agents()}
        activeId={activeId()}
        select={selectAgent}
        add={addAgent}
        newSession={() => void startNewSession()}
        canCreateSession={
          active().state.connection === "ready" &&
          !newSessionAction.pending(activeId())
        }
        activePage={activePage()}
        openSkills={() => navigate({ to: "/skills" })}
        openSettings={() => navigate({ to: "/settings" })}
        sessions={sessions()}
        selectSession={(agentId, sessionId) =>
          void selectSession(agentId, sessionId)
        }
      />

      <Show
        when={
          location().pathname !== "/settings" &&
          location().pathname !== "/skills"
        }
        fallback={
          <WorkbenchMain projectionBoundary>
            <Show
              when={location().pathname === "/skills"}
              fallback={
                <SettingsPage
                  app={defaults.value()}
                  appLoadError={defaults.loadError()}
                  appSaveError={defaults.saveError()}
                  project={active()}
                  state={active().state}
                  canDeleteProject={agents().length > 1}
                  updateApp={defaults.update}
                  reloadApp={defaults.reload}
                  retryAppSave={defaults.retrySave}
                  updateProject={patchActive}
                  deleteProject={deleteActiveAgent}
                  setAutoCompaction={(enabled) =>
                    api.setAutoCompaction(active().id, enabled)
                  }
                  setSteeringMode={(mode) =>
                    api.setSteeringMode(active().id, mode)
                  }
                  setFollowUpMode={(mode) =>
                    api.setFollowUpMode(active().id, mode)
                  }
                  close={() => navigate({ to: `/agents/${activeId()}` })}
                />
              }
            >
              <SkillsPage
                cwd={active().cwd}
                project={active().name}
                load={api.listSkills}
                close={() => navigate({ to: `/agents/${activeId()}` })}
              />
            </Show>
          </WorkbenchMain>
        }
      >
        <WorkbenchMain projectionBoundary>
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
            canGoBack={sessionNavigation.canGoBack()}
            canGoForward={sessionNavigation.canGoForward()}
            goBack={() => traverseSessionHistory("back")}
            goForward={() => traverseSessionHistory("forward")}
            toggleTerminal={toggleTerminal}
            toggleFiles={toggleFiles}
            toggleChanges={toggleChanges}
            toggleSearch={() => setSearchOpen((open) => !open)}
            newSession={startNewSession}
            newSessionPending={newSessionAction.pending(activeId())}
            compactSession={() => api.compactSession(active().id)}
            cloneSession={() => api.cloneSession(active().id)}
            exportSession={exportActiveSession}
            sessionActionError={(_action, error) =>
              toasts.error(i18n.message(m.session_action_failed, {}), {
                description: String(error),
              })
            }
            abort={abortActive}
            abortPending={abortAction.pending(activeId())}
          />

          <Show
            when={active().cwd.trim()}
            fallback={
              <WorkspaceSetupBoundary
                pending={workspacePending(active().id)}
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
            <MessageScroller
              projectionBoundary
              class="flex-1 min-h-0"
              followEnd={active().state.items.length > 0}
            >
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
              <MessageScrollerViewport
                role="region"
                aria-label="Conversation transcript"
              >
                <MessageScrollerContent>
                  <WorkbenchContentColumn class="px-6 py-5">
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
                        fork={openFork}
                        retry={() => void submit()}
                      />
                    </Show>
                  </WorkbenchContentColumn>
                </MessageScrollerContent>
              </MessageScrollerViewport>
              <Show when={active().state.items.length > 0}>
                <ConversationNavigator items={active().state.items} />
                <MessageScrollerButton />
              </Show>
            </MessageScroller>

            <ConversationComposer
              connection={active().state.connection}
              error={
                active().state.error &&
                !active().state.items.some(
                  (item) =>
                    item.kind === "notice" &&
                    item.tone === "error" &&
                    item.text === active().state.error,
                )
                  ? active().state.error
                  : undefined
              }
              runtimeLog={active().state.runtimeLogs.at(-1)}
              project={active().name}
              cwd={active().cwd}
              branch={workspaceInfo.latest()?.branch}
              repository={workspaceInfo.latest()?.repository}
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
                api.setModel(active().id, provider, modelId)
              }
              chooseThinking={(level) => api.setThinking(active().id, level)}
              modelActionError={(_action, error) =>
                toasts.error(i18n.message(m.model_action_failed, {}), {
                  description: String(error),
                })
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
        checkpoint={forkCheckpoint()}
        cancel={() => {
          pendingFork.close();
          setForkCheckpoint("unavailable");
        }}
        confirm={() => {
          const target = pendingFork.value();
          pendingFork.close();
          if (!target) return;
          void (async () => {
            const agent = agents().find(
              (candidate) => candidate.id === target.ownerId,
            );
            if (!agent) return;
            let rewind: PreparedRewind | undefined;
            try {
              const sessionId = agent.state.sessionId;
              const repository = sessionId
                ? await api.workspaceInfo(agent.cwd)
                : undefined;
              if (sessionId && repository?.repository) {
                rewind = await turnCheckpoints.prepareRewind(
                  agent.cwd,
                  sessionId,
                  target.data.entryId,
                );
              }
              if (rewind) {
                pendingRewinds.set(agent.id, { cwd: agent.cwd, rewind });
              }
              await api.fork(agent.id, target.data.entryId);
            } catch (error) {
              pendingRewinds.delete(agent.id);
              if (rewind) {
                await turnCheckpoints
                  .rollback(agent.cwd, rewind)
                  .catch((rollbackError) =>
                    console.error(
                      `[pi-agent] rewind rollback failed: ${String(rollbackError)}`,
                    ),
                  );
              }
              toasts.error("Could not fork from this message", {
                description: String(error),
              });
            }
          })();
        }}
      />
      <ExtensionUiDialog
        request={extensionDialogs()[0]}
        respond={(answer) => {
          const request = extensionDialogs()[0];
          return request ? respondToExtension(request, answer) : undefined;
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
