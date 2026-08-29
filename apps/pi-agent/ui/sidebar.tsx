import {
  Button,
  Icon,
  SearchField,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  Sidebar as SidebarRoot,
  Text,
  View,
} from "@wabou/ui";
import bot from "lucide-static/icons/bot.svg?raw";
import chevronDown from "lucide-static/icons/chevron-down.svg?raw";
import chevronRight from "lucide-static/icons/chevron-right.svg?raw";
import folder from "lucide-static/icons/folder.svg?raw";
import messageSquare from "lucide-static/icons/message-square.svg?raw";
import plus from "lucide-static/icons/plus.svg?raw";
import settings from "lucide-static/icons/settings.svg?raw";
import {
  createEffect,
  createMemo,
  createSignal,
  For as ForValue,
  onCleanup,
  Show,
} from "solid-js";
import { AgentSidebarStatus } from "./agent-activity";
import type { PiSession } from "./api";
import { i18n, m } from "./i18n";
import {
  activeSidebarValue,
  nextSessionClockDelay,
  sessionLabel,
  sessionTimeLabel,
  sortSessionsByRecency,
} from "./sidebar-presentation";
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
  /** Unix seconds. Supplying this keeps component and layout tests deterministic. */
  nowSeconds?: number;
}

export function Sidebar(props: SidebarProps) {
  const [query, setQuery] = createSignal("");
  const [collapsedAgents, setCollapsedAgents] = createSignal<
    ReadonlySet<string>
  >(new Set());
  const [liveNowSeconds, setLiveNowSeconds] = createSignal(
    Math.floor(Date.now() / 1_000),
  );
  const nowSeconds = () => props.nowSeconds ?? liveNowSeconds();
  let clockTimer: ReturnType<typeof setTimeout> | undefined;
  createEffect(
    () =>
      props.nowSeconds === undefined
        ? {
            now: liveNowSeconds(),
            timestamps: props.sessions.map((session) => session.updatedAt),
          }
        : undefined,
    (clock) => {
      if (clockTimer !== undefined) clearTimeout(clockTimer);
      clockTimer = undefined;
      if (!clock) return;
      const delay = nextSessionClockDelay(
        clock.timestamps.map((updatedAt) => ({ updatedAt })),
        clock.now,
      );
      if (delay === undefined) return;
      clockTimer = setTimeout(
        () => setLiveNowSeconds(Math.floor(Date.now() / 1_000)),
        delay * 1_000,
      );
    },
  );
  onCleanup(() => {
    if (clockTimer !== undefined) clearTimeout(clockTimer);
  });
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
  const activeValue = createMemo(() =>
    activeSidebarValue(props.agents, props.activeId, props.sessions),
  );
  const activeSessionKey = createMemo(() => {
    const agent = props.agents.find(
      (candidate) => candidate.id === props.activeId,
    );
    if (!agent?.state.sessionId) return undefined;
    return props.sessions.some(
      (session) =>
        session.agentId === agent.id &&
        session.sessionId === agent.state.sessionId,
    )
      ? `${agent.id}\0${agent.state.sessionId}`
      : undefined;
  });
  let previousActiveSessionKey: string | undefined;
  createEffect(activeSessionKey, (activeKey) => {
    const previous = previousActiveSessionKey;
    previousActiveSessionKey = activeKey;
    if (!activeKey || activeKey === previous) return;
    const activeAgentId = activeKey.slice(0, activeKey.indexOf("\0"));
    setCollapsedAgents((current) => {
      if (!current.has(activeAgentId)) return current;
      const next = new Set(current);
      next.delete(activeAgentId);
      return next;
    });
  });
  const sessionsExpanded = (agentId: string) =>
    normalizedQuery().length > 0 || !collapsedAgents().has(agentId);
  const toggleSessions = (agentId: string) => {
    setCollapsedAgents((current) => {
      const next = new Set(current);
      if (next.has(agentId)) next.delete(agentId);
      else next.add(agentId);
      return next;
    });
  };
  const visibleSessions = (agentId: string) => {
    const needle = normalizedQuery();
    const sessions = sortSessionsByRecency(
      props.sessions.filter((session) => session.agentId === agentId),
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
          <SidebarMenu
            aria-label={i18n.message(m.projects, {})}
            value={activeValue()}
          >
            <ForValue each={visibleAgents()} keyed={(agent) => agent.id}>
              {(agent) => {
                const sessions = () => visibleSessions(agent().id);
                const expanded = () => sessionsExpanded(agent().id);
                return (
                  <View class="gap-1">
                    <View class="min-w-0 flex flex-row items-center gap-0.5">
                      <SidebarMenuButton
                        value={`project:${agent().id}`}
                        class="min-w-0 flex-1 h-10 px-2"
                        aria-label={agent().name}
                        onClick={() => props.select(agent().id)}
                      >
                        <Icon
                          source={folder}
                          size={15}
                          class="text-secondary"
                        />
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
                      <Show when={sessions().length > 0}>
                        <Button
                          variant="ghost"
                          size="icon"
                          class="w-7 h-7 flex-none"
                          aria-label={i18n.message(
                            expanded()
                              ? m.collapse_project_sessions
                              : m.expand_project_sessions,
                            { project: agent().name },
                          )}
                          aria-expanded={expanded()}
                          onClick={() => toggleSessions(agent().id)}
                        >
                          <Icon
                            source={expanded() ? chevronDown : chevronRight}
                            size={14}
                            class="text-muted"
                          />
                        </Button>
                      </Show>
                    </View>
                    <Show when={expanded()}>
                      <ForValue
                        each={sessions()}
                        keyed={(session) => session.sessionId}
                      >
                        {(session) => (
                          <View class="min-w-0 pl-3">
                            <SidebarMenuButton
                              value={`session:${agent().id}:${session().sessionId}`}
                              class="h-8 pl-2 text-sm"
                              aria-label={sessionLabel(session())}
                              onClick={() =>
                                props.selectSession(
                                  agent().id,
                                  session().sessionId,
                                )
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
                              <Text class="flex-none text-xs text-muted">
                                {sessionTimeLabel(
                                  session().updatedAt,
                                  nowSeconds(),
                                )}
                              </Text>
                            </SidebarMenuButton>
                          </View>
                        )}
                      </ForValue>
                    </Show>
                  </View>
                );
              }}
            </ForValue>
          </SidebarMenu>
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
