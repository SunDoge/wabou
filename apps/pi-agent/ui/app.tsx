import { currentWindow, useDialog } from "@wabou/core";
import type { Handle } from "@wabou/core/renderer";
import {
  Button,
  createToasts,
  Icon,
  MessageGroup,
  MessageScroller,
  MessageScrollerButton,
  MessageScrollerContent,
  MessageScrollerItem,
  MessageScrollerViewport,
  Text,
  TextArea,
  Toaster,
  useLocation,
  useNavigate,
  useParams,
  View,
} from "@wabou/ui";
import filePlus from "lucide-static/icons/file-plus-2.svg?raw";
import search from "lucide-static/icons/search.svg?raw";
import send from "lucide-static/icons/send.svg?raw";
import square from "lucide-static/icons/square.svg?raw";
import { createEffect, createSignal, For, onCleanup, Show } from "solid-js";
import { AgentActivityStatus } from "./agent-activity";
import {
  appendUserMessage,
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
import { ConversationItem } from "./conversation";
import { ConversationWelcome } from "./conversation-welcome";
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
import { SessionActions } from "./session-actions";
import { SessionForkDialog } from "./session-fork";
import { SessionTitle } from "./session-title";
import { SessionUsage } from "./session-usage";
import { type AgentDefaults, SettingsPage } from "./settings";
import { Sidebar } from "./sidebar";
import { TranscriptSearch } from "./transcript-search";
import {
  type AgentWorkspace,
  agentProfile,
  createAgentWorkspace,
  restoreAgentWorkspace,
} from "./workspace";
import { WorkspaceSetup } from "./workspace-setup";

function ExtensionWindowTitle(props: { title: string }) {
  createEffect(() => {
    currentWindow().setTitle(props.title);
  });
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
  const [defaults, setDefaults] = createSignal<AgentDefaults>({
    proxy: "",
    noProxy: "127.0.0.1,localhost",
    provider: "",
    model: "",
  });
  const [agents, setAgents] = createSignal<readonly AgentWorkspace[]>([
    createAgentWorkspace(1),
  ]);
  const [sessions, setSessions] = createSignal<readonly PiSession[]>([]);
  const [lastActiveId, setLastActiveId] = createSignal("agent-1");
  const [drafts, setDrafts] = createSignal<AgentDrafts>({});
  const [draftImages, setDraftImages] = createSignal<AgentDraftLists>({});
  const [draftContext, setDraftContext] = createSignal<AgentDraftLists>({});
  const [searchOpen, setSearchOpen] = createSignal(false);
  const [deliveryMode, setDeliveryMode] =
    createSignal<ComposerDeliveryMode>("followUp");
  const [activeSearchItem, setActiveSearchItem] = createSignal<string>();
  const [pendingFork, setPendingFork] = createSignal<{
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
  const itemHandles = new Map<string, Handle>();
  let nextMessage = 1;
  let profilesHydrated = false;
  let saveProfilesTimer: ReturnType<typeof setTimeout> | undefined;

  void api
    .listAgents()
    .then(async (profiles) => {
      if (profiles.length > 0) {
        const restored = profiles.map(restoreAgentWorkspace);
        setAgents(restored);
        setLastActiveId(restored[0].id);
      } else {
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
    });

  createEffect(
    () => JSON.stringify(agents().map(agentProfile)),
    (serialized) => {
      if (!profilesHydrated) return;
      if (saveProfilesTimer !== undefined) clearTimeout(saveProfilesTimer);
      saveProfilesTimer = setTimeout(() => {
        saveProfilesTimer = undefined;
        void api.saveAgents(JSON.parse(serialized));
      }, 150);
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
  const activeSession = () =>
    sessions().find(
      (session) =>
        session.agentId === activeId() &&
        session.sessionId === active().state.sessionId,
    );
  const activeSessionId = () => params().sessionId;
  createEffect(
    () => `${activeId()}\0${activeSessionId() ?? ""}`,
    () => {
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
          void api.getMessages(id);
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
          const stateEvent = batch.find(
            (event) =>
              event.type === "response" &&
              event.command === "get_state" &&
              event.success === true,
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
    if (saveProfilesTimer !== undefined) clearTimeout(saveProfilesTimer);
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
                connection: status.running ? "ready" : "stopped",
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
        proxy: agent.proxy.trim() || undefined,
        noProxy: agent.noProxy.trim() || undefined,
        provider: agent.provider.trim() || undefined,
        model: agent.model.trim() || undefined,
        sessionId,
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
    const agent = { ...createAgentWorkspace(nextIndex), ...defaults() };
    setAgents((current) => [...current, agent]);
    setLastActiveId(agent.id);
    void navigate({ to: `/agents/${agent.id}` });
    void api
      .defaultWorkspace(agent.id)
      .then((cwd) => {
        updateAgent(agent.id, (current) =>
          current.cwd ? current : { ...current, cwd },
        );
      })
      .catch((error) => {
        console.error(
          `[pi-agent] could not prepare the default workspace: ${String(error)}`,
        );
      });
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
            value={defaults()}
            agent={active()}
            state={active().state}
            update={(patch) =>
              setDefaults((current) => ({ ...current, ...patch }))
            }
            updateAgent={patchActive}
            deleteAgent={() => void deleteActiveAgent()}
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
            <View class="min-w-0 flex-1 flex flex-row items-center gap-2">
              <View class="min-w-0 flex flex-col gap-0">
                <Text class="font-semibold">
                  {activeSession()?.name ??
                    active().state.sessionName ??
                    active().name}
                </Text>
                <View class="min-w-0 flex flex-row items-center gap-2">
                  <Text class="min-w-0 flex-1 truncate text-xs text-muted">
                    {(active().state.model ?? active().model) ||
                      i18n.message(m.no_model, {})}{" "}
                    ·{" "}
                    {i18n.message(m.thinking, {
                      level: active().state.thinking ?? "default",
                    })}
                  </Text>
                  <AgentActivityStatus state={active().state} />
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
            <View class="flex items-center gap-1">
              <ModelControls
                models={active().state.models}
                modelProvider={active().state.modelProvider}
                modelId={active().state.modelId}
                thinking={active().state.thinking}
                thinkingLevels={active().state.availableThinkingLevels}
                disabled={active().state.connection !== "ready"}
                chooseModel={(provider, modelId) =>
                  void api.setModel(active().id, provider, modelId)
                }
                chooseThinking={(level) =>
                  void api.setThinking(active().id, level)
                }
              />
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
              <Button
                variant="ghost"
                size="icon"
                aria-label={i18n.message(m.new_session, {})}
                disabled={active().state.connection !== "ready"}
                onClick={() => void api.newSession(active().id)}
              >
                <Icon source={filePlus} size={15} />
              </Button>
              <SessionActions
                disabled={
                  active().state.connection !== "ready" ||
                  !active().state.sessionId
                }
                compact={() => void api.compactSession(active().id)}
                clone={() => void api.cloneSession(active().id)}
                exportHtml={() => void exportActiveSession()}
              />
              <Show when={active().state.connection === "running"}>
                <Button
                  variant="outline"
                  onClick={() => void api.abort(active().id)}
                >
                  <Icon source={square} size={12} /> {i18n.message(m.stop, {})}
                </Button>
              </Show>
            </View>
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
                proxy={active().proxy}
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
                  resolveItem={(id) => itemHandles.get(id)}
                  activeChanged={setActiveSearchItem}
                  close={() => setSearchOpen(false)}
                />
              </Show>
              <MessageScrollerViewport>
                <MessageScrollerContent class="max-w-4xl mx-auto p-5">
                  <Show
                    when={active().state.items.length > 0}
                    fallback={<ConversationWelcome choosePrompt={setDraft} />}
                  >
                    <MessageGroup>
                      <For each={active().state.items}>
                        {(item) => (
                          <MessageScrollerItem
                            ref={(node) => itemHandles.set(item.id, node)}
                            class={
                              activeSearchItem() === item.id
                                ? "rounded-lg bg-selected"
                                : undefined
                            }
                          >
                            <ConversationItem
                              item={item}
                              fork={
                                item.kind === "user" && item.entryId
                                  ? () =>
                                      setPendingFork({
                                        entryId: item.entryId ?? "",
                                        text: item.text,
                                      })
                                  : undefined
                              }
                            />
                          </MessageScrollerItem>
                        )}
                      </For>
                    </MessageGroup>
                  </Show>
                </MessageScrollerContent>
              </MessageScrollerViewport>
              <MessageScrollerButton />
            </MessageScroller>

            <View class="flex-none border-t border-subtle bg-surface p-4">
              <View class="max-w-4xl mx-auto min-w-0 rounded-xl border border-strong bg-input shadow-sm p-2 gap-2">
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
                  class="h-20 border-transparent shadow-none bg-input"
                  value={draft()}
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
        </View>
      </Show>
      <SessionForkDialog
        open={pendingFork() !== undefined}
        cancel={() => setPendingFork(undefined)}
        confirm={() => {
          const target = pendingFork();
          if (target) void api.fork(activeId(), target.entryId);
          setPendingFork(undefined);
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
