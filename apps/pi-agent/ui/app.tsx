import {
  Button,
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
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
  View,
} from "@wabou/ui";
import bot from "lucide-static/icons/bot.svg?raw";
import brain from "lucide-static/icons/brain.svg?raw";
import chevronsUpDown from "lucide-static/icons/chevrons-up-down.svg?raw";
import filePlus from "lucide-static/icons/file-plus-2.svg?raw";
import send from "lucide-static/icons/send.svg?raw";
import square from "lucide-static/icons/square.svg?raw";
import { createEffect, createSignal, For, onCleanup, Show } from "solid-js";
import {
  appendUserMessage,
  reducePiEvent,
  reducePiEvents,
} from "./agent-state";
import { usePiApi } from "./api";
import { ConversationItem } from "./conversation";
import { i18n, m } from "./i18n";
import { type AgentDefaults, SettingsPage } from "./settings";
import { Sidebar } from "./sidebar";
import { type AgentWorkspace, createAgentWorkspace } from "./workspace";

export function App() {
  const api = usePiApi();
  const navigate = useNavigate();
  const location = useLocation();
  const [defaults, setDefaults] = createSignal<AgentDefaults>({
    proxy: "",
    noProxy: "127.0.0.1,localhost",
    provider: "",
    model: "",
  });
  const [agents, setAgents] = createSignal<readonly AgentWorkspace[]>([
    createAgentWorkspace(1),
  ]);
  const [activeId, setActiveId] = createSignal("agent-1");
  const [draft, setDraft] = createSignal("");
  let nextAgent = 2;
  let nextMessage = 1;

  const active = () =>
    agents().find((agent) => agent.id === activeId()) ?? agents()[0];
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
    }
  });
  onCleanup(unsubscribe);

  createEffect(
    () =>
      agents()
        .map((agent) => agent.id)
        .join("\0"),
    (agentIds) => {
      for (const id of agentIds.split("\0").filter(Boolean)) {
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

  const start = async (): Promise<boolean> => {
    const agent = active();
    try {
      const status = await api.start({
        agentId: agent.id,
        cwd: agent.cwd,
        proxy: agent.proxy.trim() || undefined,
        noProxy: agent.noProxy.trim() || undefined,
        provider: agent.provider.trim() || undefined,
        model: agent.model.trim() || undefined,
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
    const agent = active();
    if (!message || agent.state.connection === "running") return;
    if (agent.state.connection !== "ready" && !(await start())) return;
    setDraft("");
    updateAgent(agent.id, (current) => ({
      ...current,
      state: appendUserMessage(current.state, `user-${nextMessage++}`, message),
    }));
    try {
      await api.prompt(agent.id, message);
    } catch (error) {
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
    const agent = { ...createAgentWorkspace(nextAgent++), ...defaults() };
    setAgents((current) => [...current, agent]);
    setActiveId(agent.id);
    setDraft("");
  };

  const setConfiguredModel = async () => {
    const agent = active();
    if (!agent.provider.trim() || !agent.model.trim()) return;
    await api.setModel(agent.id, agent.provider.trim(), agent.model.trim());
  };

  return (
    <View class="w-full h-full min-w-0 min-h-0 flex bg-canvas text-primary">
      <Sidebar
        agents={agents()}
        activeId={active().id}
        select={(id) => {
          setActiveId(id);
          setDraft("");
        }}
        add={addAgent}
        openSettings={() => navigate({ to: "/settings" })}
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
            close={() => navigate({ to: "/" })}
          />
        }
      >
        <View class="flex-1 min-w-0 min-h-0 flex flex-col">
          <View class="h-14 flex-none px-5 border-b border-subtle bg-surface flex items-center justify-between gap-3">
            <View class="min-w-0">
              <Text class="font-semibold">{active().name}</Text>
              <Text class="text-xs text-muted">
                {(active().state.model ?? active().model) ||
                  i18n.message(m.no_model, {})}{" "}
                ·{" "}
                {i18n.message(m.thinking, {
                  level: active().state.thinking ?? "default",
                })}
              </Text>
            </View>
            <View class="flex items-center gap-1">
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

          <MessageScroller class="flex-1 min-h-0" followEnd>
            <MessageScrollerViewport>
              <MessageScrollerContent class="max-w-4xl mx-auto p-5">
                <Show
                  when={active().state.items.length > 0}
                  fallback={
                    <Empty variant="plain" class="min-h-72">
                      <EmptyHeader>
                        <EmptyMedia variant="icon">
                          <Icon source={bot} size={20} class="text-accent" />
                        </EmptyMedia>
                        <EmptyTitle>
                          {i18n.message(m.empty_title, {})}
                        </EmptyTitle>
                        <EmptyDescription>
                          {i18n.message(m.empty_detail, {})}
                        </EmptyDescription>
                      </EmptyHeader>
                    </Empty>
                  }
                >
                  <MessageGroup>
                    <For each={active().state.items}>
                      {(item) => (
                        <MessageScrollerItem>
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
              <TextArea
                class="h-20 border-transparent shadow-none bg-input"
                value={draft()}
                disabled={active().state.connection === "running"}
                placeholder={
                  active().state.connection === "running"
                    ? i18n.message(m.working, {})
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
                <Text class="text-xs text-muted">
                  {i18n.message(m.send_hint, {})}
                </Text>
                <Button
                  disabled={
                    !draft().trim() || active().state.connection === "running"
                  }
                  onClick={() => void submit()}
                >
                  <Icon source={send} size={14} /> {i18n.message(m.send, {})}
                </Button>
              </View>
            </View>
          </View>
        </View>
      </Show>
    </View>
  );
}
