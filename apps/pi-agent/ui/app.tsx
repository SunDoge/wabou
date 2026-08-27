import type { Handle } from "@wabou/core/renderer";
import {
  Button,
  Icon,
  MessageGroup,
  MessageScroller,
  MessageScrollerButton,
  MessageScrollerContent,
  MessageScrollerItem,
  MessageScrollerViewport,
  Text,
  TextArea,
  useLocation,
  useNavigate,
  useParams,
  View,
} from "@wabou/ui";
import brain from "lucide-static/icons/brain.svg?raw";
import chevronsUpDown from "lucide-static/icons/chevrons-up-down.svg?raw";
import filePlus from "lucide-static/icons/file-plus-2.svg?raw";
import search from "lucide-static/icons/search.svg?raw";
import send from "lucide-static/icons/send.svg?raw";
import square from "lucide-static/icons/square.svg?raw";
import { createEffect, createSignal, For, onCleanup, Show } from "solid-js";
import {
  appendUserMessage,
  reducePiEvent,
  reducePiEvents,
} from "./agent-state";
import { type PiSession, usePiApi } from "./api";
import { CommandPicker } from "./command-picker";
import { ComposerImagePicker, ComposerImages } from "./composer-images";
import { ConversationItem } from "./conversation";
import { ConversationWelcome } from "./conversation-welcome";
import {
  type AgentDrafts,
  agentDraftKey,
  readAgentDraft,
  removeAgentDrafts,
  writeAgentDraft,
} from "./drafts";
import { i18n, m } from "./i18n";
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

export function App() {
  const api = usePiApi();
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
  const [draftImages, setDraftImages] = createSignal<
    Readonly<Record<string, readonly string[]>>
  >({});
  const [searchOpen, setSearchOpen] = createSignal(false);
  const [activeSearchItem, setActiveSearchItem] = createSignal<string>();
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
    draftImages()[agentDraftKey(activeId(), activeSessionId())] ?? [];
  const setImages = (paths: readonly string[]) => {
    const key = agentDraftKey(activeId(), activeSessionId());
    setDraftImages((current) => {
      if (paths.length > 0) return { ...current, [key]: paths };
      const next = { ...current };
      delete next[key];
      return next;
    });
  };
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

  const unsubscribe = api.subscribe((events) => {
    const grouped = new Map<string, Record<string, unknown>[]>();
    for (const event of events) {
      const id = typeof event.agentId === "string" ? event.agentId : "agent-1";
      const group = grouped.get(id) ?? [];
      group.push(event);
      grouped.set(id, group);
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
          stateEvent?.id === "wabou-new-session-state" &&
          id === activeId() &&
          typeof data?.sessionId === "string"
        ) {
          void navigate({ to: `/agents/${id}/sessions/${data.sessionId}` });
        }
      }
      if (batch.some((event) => event.type === "agent_end")) {
        void api.getSessionStats(id);
      }
    }
  });
  onCleanup(() => {
    unsubscribe();
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
    const agent = active();
    if (!message) return;
    const queueing = agent.state.connection === "running";
    if (agent.state.connection !== "ready" && !(await start())) return;
    setDraft("");
    setImages([]);
    updateAgent(agent.id, (current) => ({
      ...current,
      state: appendUserMessage(
        current.state,
        `user-${nextMessage++}`,
        message,
        queueing,
      ),
    }));
    try {
      await (queueing
        ? api.followUp(agent.id, message, attachedImages)
        : api.prompt(agent.id, message, attachedImages));
    } catch (error) {
      setDraft(message);
      setImages(attachedImages);
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

  const setConfiguredModel = async () => {
    const agent = active();
    if (!agent.provider.trim() || !agent.model.trim()) return;
    await api.setModel(agent.id, agent.provider.trim(), agent.model.trim());
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
            update={(patch) =>
              setDefaults((current) => ({ ...current, ...patch }))
            }
            updateAgent={patchActive}
            deleteAgent={() => void deleteActiveAgent()}
            close={() => navigate({ to: `/agents/${activeId()}` })}
          />
        }
      >
        <View class="flex-1 min-w-0 min-h-0 flex flex-col">
          <View class="h-14 flex-none px-5 border-b border-subtle bg-surface flex items-center justify-between gap-3">
            <View class="min-w-0 flex-1 flex flex-row items-center gap-2">
              <View class="min-w-0">
                <Text class="font-semibold">
                  {activeSession()?.name ??
                    active().state.sessionName ??
                    active().name}
                </Text>
                <Text class="text-xs text-muted">
                  {(active().state.model ?? active().model) ||
                    i18n.message(m.no_model, {})}{" "}
                  ·{" "}
                  {i18n.message(m.thinking, {
                    level: active().state.thinking ?? "default",
                  })}
                </Text>
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
                size="sm"
                disabled={
                  active().state.connection !== "ready" ||
                  !active().provider.trim() ||
                  !active().model.trim()
                }
                onClick={() => void setConfiguredModel()}
              >
                {i18n.message(m.apply_model, {})}
              </Button>
              <Button
                variant="ghost"
                size="icon"
                aria-label={i18n.message(m.cycle_model, {})}
                disabled={active().state.connection !== "ready"}
                onClick={() => void api.cycleModel(active().id)}
              >
                <Icon source={chevronsUpDown} size={15} />
              </Button>
              <Button
                variant="ghost"
                size="icon"
                aria-label={i18n.message(m.cycle_thinking, {})}
                disabled={active().state.connection !== "ready"}
                onClick={() => void api.cycleThinking(active().id)}
              >
                <Icon source={brain} size={15} />
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
                            <ConversationItem item={item} />
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
                <ComposerImages paths={images()} change={setImages} />
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
                <View class="flex items-center justify-between gap-3 px-1">
                  <View class="min-w-0 flex flex-row items-center gap-3">
                    <ComposerImagePicker paths={images()} change={setImages} />
                    <CommandPicker
                      commands={active().state.commands}
                      choose={setDraft}
                    />
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
    </View>
  );
}
