import {
  Button,
  Icon,
  Input,
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
import { Sidebar } from "./sidebar";
import { type AgentWorkspace, createAgentWorkspace } from "./workspace";

interface AgentDefaults {
  proxy: string;
  noProxy: string;
  provider: string;
  model: string;
}

function SettingsPage(props: {
  value: AgentDefaults;
  update: (patch: Partial<AgentDefaults>) => void;
  close: () => void;
}) {
  return (
    <View class="flex-1 min-w-0 min-h-0 overflow-y-auto bg-canvas p-8">
      <View class="max-w-3xl mx-auto gap-6">
        <View class="gap-1">
          <Text class="text-2xl font-semibold">
            {i18n.message(m.settings, {})}
          </Text>
          <Text class="text-sm text-muted whitespace-normal">
            {i18n.message(m.settings_intro, {})}
          </Text>
        </View>
        <View class="rounded-xl border border-subtle bg-surface p-5 gap-3 shadow-sm">
          <Text class="font-semibold">{i18n.message(m.language, {})}</Text>
          <View class="flex gap-2">
            <Button
              variant={i18n.locale() === "en" ? "default" : "outline"}
              onClick={() => i18n.set("en")}
            >
              {i18n.message(m.english, {})}
            </Button>
            <Button
              variant={i18n.locale() === "zh" ? "default" : "outline"}
              onClick={() => i18n.set("zh")}
            >
              {i18n.message(m.chinese, {})}
            </Button>
          </View>
        </View>
        <View class="rounded-xl border border-subtle bg-surface p-5 gap-3 shadow-sm">
          <Text class="font-semibold">
            {i18n.message(m.default_provider, {})}
          </Text>
          <Input
            aria-label="Default provider"
            value={props.value.provider}
            placeholder={i18n.message(m.provider_placeholder, {})}
            onInput={(event) =>
              props.update({ provider: event.currentTarget.value })
            }
          />
          <Input
            aria-label="Default model"
            value={props.value.model}
            placeholder={i18n.message(m.model_optional, {})}
            onInput={(event) =>
              props.update({ model: event.currentTarget.value })
            }
          />
        </View>
        <View class="rounded-xl border border-subtle bg-surface p-5 gap-3 shadow-sm">
          <Text class="font-semibold">{i18n.message(m.default_proxy, {})}</Text>
          <Input
            aria-label="Default proxy URL"
            value={props.value.proxy}
            placeholder="http://127.0.0.1:7890"
            onInput={(event) =>
              props.update({ proxy: event.currentTarget.value })
            }
          />
          <Input
            aria-label="Default proxy bypass list"
            value={props.value.noProxy}
            onInput={(event) =>
              props.update({ noProxy: event.currentTarget.value })
            }
          />
          <Text class="text-sm text-muted whitespace-normal">
            {i18n.message(m.proxy_detail, {})}
          </Text>
        </View>
        <View class="rounded-xl border border-subtle bg-surface p-5 gap-2 shadow-sm">
          <Text class="font-semibold">{i18n.message(m.runtime, {})}</Text>
          <Text class="text-sm text-secondary">
            {i18n.message(m.runtime_kind, {})}
          </Text>
          <Text class="text-sm text-muted whitespace-normal">
            {i18n.message(m.runtime_detail, {})}
          </Text>
        </View>
        <Button variant="outline" onClick={props.close}>
          {i18n.message(m.back_to_agents, {})}
        </Button>
      </View>
    </View>
  );
}

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
  const [pendingAgent, setPendingAgent] = createSignal<string>();
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
    setPendingAgent(agent.id);
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
    } finally {
      setPendingAgent(undefined);
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
        active={active()}
        pending={pendingAgent() === active().id}
        select={(id) => {
          setActiveId(id);
          setDraft("");
        }}
        add={addAgent}
        update={patchActive}
        start={start}
        openSettings={() => navigate({ to: "/settings" })}
      />

      <Show
        when={location().pathname !== "/settings"}
        fallback={
          <SettingsPage
            value={defaults()}
            update={(patch) =>
              setDefaults((current) => ({ ...current, ...patch }))
            }
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
                    <View class="flex-1 min-h-72 items-center justify-center gap-3">
                      <View class="w-12 h-12 rounded-xl bg-selected flex items-center justify-center">
                        <Icon source={bot} size={24} class="text-accent" />
                      </View>
                      <Text class="text-lg font-semibold">
                        {i18n.message(m.empty_title, {})}
                      </Text>
                      <Text class="max-w-md text-center text-sm text-muted whitespace-normal">
                        {i18n.message(m.empty_detail, {})}
                      </Text>
                    </View>
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
