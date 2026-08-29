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
import folder from "lucide-static/icons/folder.svg?raw";
import messageSquare from "lucide-static/icons/message-square.svg?raw";
import plus from "lucide-static/icons/plus.svg?raw";
import settings from "lucide-static/icons/settings.svg?raw";
import { createMemo, createSignal, For as ForValue, Show } from "solid-js";
import type { PiSession } from "./api";
import { AgentSidebarStatus } from "./agent-activity";
import { i18n, m } from "./i18n";
import type { AgentWorkspace } from "./workspace";

interface SidebarProps {
  agents: readonly AgentWorkspace[];
  activeId: string;
  select: (id: string) => void;
  add: () => void;
  newSession: () => void;
  canCreateSession: boolean;
  openSettings: () => void;
  sessions: readonly PiSession[];
  selectSession: (agentId: string, sessionId: string) => void;
}

export function sessionLabel(session: Pick<PiSession, "name" | "sessionId">) {
  const name = session.name?.trim();
  return name || session.sessionId.slice(0, 8);
}

export function projectRowSelected(
  agent: AgentWorkspace,
  activeId: string,
  sessions: readonly PiSession[],
): boolean {
  if (agent.id !== activeId) return false;
  return !sessions.some(
    (session) =>
      session.agentId === agent.id &&
      session.sessionId === agent.state.sessionId,
  );
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
          `${sessionLabel(session)}\n${session.sessionId}`
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
      `${sessionLabel(session)}\n${session.sessionId}`
        .toLocaleLowerCase()
        .includes(needle),
    );
  };
  return (
    <SidebarRoot class="w-60 border-r border-subtle bg-surface-muted">
      <SidebarHeader class="h-12 border-0 bg-surface-muted flex items-center gap-2 px-4">
        <Icon source={bot} size={16} class="text-secondary" />
        <Text class="min-w-0 flex-1 text-sm font-semibold text-secondary">
          {i18n.message(m.app_name, {})}
        </Text>
      </SidebarHeader>

      <SidebarContent contentClass="px-2 py-1 gap-2">
        <Show when={props.canCreateSession}>
          <SidebarMenuButton class="h-8" onClick={props.newSession}>
            <Icon source={messageSquare} size={16} />
            {i18n.message(m.new_thread, {})}
          </SidebarMenuButton>
        </Show>

        <Show when={props.sessions.length > 0}>
          <SearchField
            aria-label={i18n.message(m.search_agents, {})}
            value={query()}
            onValueChange={setQuery}
            placeholder={i18n.message(m.search_short, {})}
            clearLabel={i18n.message(m.clear_search, {})}
            class="border-transparent bg-transparent shadow-none"
          />
        </Show>

        <SidebarGroup>
          <View class="px-2 pt-2 flex flex-row items-center justify-between gap-2">
            <SidebarGroupLabel>
              {i18n.message(m.projects, {})}
            </SidebarGroupLabel>
            <Button
              variant="ghost"
              size="icon"
              class="w-7 h-7"
              aria-label={i18n.message(m.new_project, {})}
              onClick={props.add}
            >
              <Icon source={plus} size={13} />
            </Button>
          </View>
          <ForValue each={visibleAgents()} keyed={(agent) => agent.id}>
            {(agent) => (
              <View class="gap-1">
                <SidebarMenuButton
                  class="h-10 px-2"
                  aria-label={agent().name}
                  selected={projectRowSelected(
                    agent(),
                    props.activeId,
                    props.sessions,
                  )}
                  onClick={() => props.select(agent().id)}
                >
                  <Icon source={folder} size={15} class="text-secondary" />
                  <View class="min-w-0 flex-1 gap-0">
                    <Text class="truncate text-sm font-medium">
                      {agent().name}
                    </Text>
                    <Text class="truncate text-xs text-muted">
                      {workspaceName(agent().cwd)}
                    </Text>
                  </View>
                  <AgentSidebarStatus state={agent().state} />
                </SidebarMenuButton>
                <Show when={agent().id === props.activeId || normalizedQuery()}>
                  <ForValue
                    each={visibleSessions(agent().id)}
                    keyed={(session) => session.sessionId}
                  >
                    {(session) => (
                      <View class="min-w-0 pl-3">
                        <SidebarMenuButton
                          class="h-8 pl-2 text-sm"
                          aria-label={sessionLabel(session())}
                          selected={
                            agent().state.sessionId === session().sessionId &&
                            agent().id === props.activeId
                          }
                          onClick={() =>
                            props.selectSession(agent().id, session().sessionId)
                          }
                        >
                          <Icon
                            source={messageSquare}
                            size={13}
                            class="flex-none text-muted"
                          />
                          <Text class="min-w-0 flex-1 truncate">
                            {sessionLabel(session())}
                          </Text>
                        </SidebarMenuButton>
                      </View>
                    )}
                  </ForValue>
                </Show>
              </View>
            )}
          </ForValue>
          <Show when={visibleAgents().length === 0}>
            <Text role="status" class="px-3 py-4 text-sm text-muted">
              {i18n.message(m.no_agents_found, {})}
            </Text>
          </Show>
        </SidebarGroup>
      </SidebarContent>

      <SidebarFooter class="border-0 bg-surface-muted px-2 py-2">
        <SidebarMenuButton onClick={props.openSettings}>
          <Icon source={settings} size={16} />
          {i18n.message(m.settings, {})}
        </SidebarMenuButton>
      </SidebarFooter>
    </SidebarRoot>
  );
}

export function workspaceName(path: string): string {
  const normalized = path.replace(/[\\/]+$/, "");
  if (!normalized) return i18n.message(m.workspace_not_selected, {});
  return normalized.split(/[\\/]/).at(-1) || normalized;
}
