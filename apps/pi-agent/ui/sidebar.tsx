import {
  Button,
  Icon,
  SearchField,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenuButton,
  Sidebar as SidebarRoot,
  Text,
  View,
} from "@wabou/ui";
import bot from "lucide-static/icons/bot.svg?raw";
import plus from "lucide-static/icons/plus.svg?raw";
import settings from "lucide-static/icons/settings.svg?raw";
import { createMemo, createSignal, For, Show } from "solid-js";
import type { PiSession } from "./api";
import { i18n, m } from "./i18n";
import type { AgentWorkspace } from "./workspace";

interface SidebarProps {
  agents: readonly AgentWorkspace[];
  activeId: string;
  select: (id: string) => void;
  add: () => void;
  openSettings: () => void;
  sessions: readonly PiSession[];
  selectSession: (agentId: string, sessionId: string) => void;
}

export function Sidebar(props: SidebarProps) {
  const [query, setQuery] = createSignal("");
  const normalizedQuery = () => query().trim().toLocaleLowerCase();
  const visibleAgents = createMemo(() => {
    const needle = normalizedQuery();
    if (!needle) return props.agents;
    return props.agents.filter((agent) => {
      if (`${agent.name}\n${agent.cwd}`.toLocaleLowerCase().includes(needle)) {
        return true;
      }
      return props.sessions.some(
        (session) =>
          session.agentId === agent.id &&
          `${session.name ?? ""}\n${session.sessionId}`
            .toLocaleLowerCase()
            .includes(needle),
      );
    });
  });
  const visibleSessions = (agentId: string) => {
    const needle = normalizedQuery();
    const sessions = props.sessions.filter(
      (session) => session.agentId === agentId,
    );
    if (!needle) return sessions;
    const agent = props.agents.find((candidate) => candidate.id === agentId);
    if (
      `${agent?.name ?? ""}\n${agent?.cwd ?? ""}`
        .toLocaleLowerCase()
        .includes(needle)
    ) {
      return sessions;
    }
    return sessions.filter((session) =>
      `${session.name ?? ""}\n${session.sessionId}`
        .toLocaleLowerCase()
        .includes(needle),
    );
  };
  return (
    <SidebarRoot class="w-72 border-r border-subtle" elevation="floating">
      <SidebarHeader class="flex items-center gap-3 px-4 py-4">
        <Icon source={bot} size={19} class="text-accent" />
        <Text class="min-w-0 flex-1 font-semibold text-base">
          {i18n.message(m.app_name, {})}
        </Text>
      </SidebarHeader>

      <SidebarContent contentClass="gap-3">
        <Button
          class="w-full justify-start"
          variant="outline"
          onClick={props.add}
        >
          <Icon source={plus} size={16} />
          {i18n.message(m.new_agent, {})}
        </Button>

        <SearchField
          aria-label={i18n.message(m.search_agents, {})}
          value={query()}
          onValueChange={setQuery}
          placeholder={i18n.message(m.search_agents, {})}
          clearLabel={i18n.message(m.clear_search, {})}
        />

        <SidebarGroup>
          <SidebarGroupLabel>{i18n.message(m.agents, {})}</SidebarGroupLabel>
          <For each={visibleAgents()} keyed={(agent) => agent.id}>
            {(agent) => (
              <View>
                <SidebarMenuButton
                  selected={agent().id === props.activeId}
                  onClick={() => props.select(agent().id)}
                >
                  <Text class="min-w-0 flex-1 truncate">{agent().name}</Text>
                  <View
                    class={
                      agent().state.connection === "running"
                        ? "w-2 h-2 flex-none rounded-full bg-accent"
                        : agent().state.connection === "ready"
                          ? "w-2 h-2 flex-none rounded-full bg-success-primary"
                          : "w-2 h-2 flex-none rounded-full bg-strong"
                    }
                  />
                </SidebarMenuButton>
                <For
                  each={visibleSessions(agent().id)}
                  keyed={(session) => session.sessionId}
                >
                  {(session) => (
                    <SidebarMenuButton
                      class="pl-8 text-sm"
                      selected={
                        agent().state.sessionId === session().sessionId &&
                        agent().id === props.activeId
                      }
                      onClick={() =>
                        props.selectSession(agent().id, session().sessionId)
                      }
                    >
                      <Text class="min-w-0 flex-1 truncate">
                        {session().name ?? session().sessionId.slice(0, 8)}
                      </Text>
                    </SidebarMenuButton>
                  )}
                </For>
              </View>
            )}
          </For>
          <Show when={visibleAgents().length === 0}>
            <Text role="status" class="px-3 py-4 text-sm text-muted">
              {i18n.message(m.no_agents_found, {})}
            </Text>
          </Show>
        </SidebarGroup>
      </SidebarContent>

      <SidebarFooter class="px-2 py-3">
        <SidebarMenuButton onClick={props.openSettings}>
          <Icon source={settings} size={16} />
          {i18n.message(m.settings, {})}
        </SidebarMenuButton>
      </SidebarFooter>
    </SidebarRoot>
  );
}
