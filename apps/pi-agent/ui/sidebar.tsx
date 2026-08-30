import {
  Button,
  Icon,
  IconFrame,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  Sidebar as SidebarRoot,
  SidebarSearch,
  Text,
  View,
} from "@wabou/ui";
import bot from "lucide-static/icons/bot.svg?raw";
import calendarClock from "lucide-static/icons/calendar-clock.svg?raw";
import chevronDown from "lucide-static/icons/chevron-down.svg?raw";
import chevronRight from "lucide-static/icons/chevron-right.svg?raw";
import folder from "lucide-static/icons/folder.svg?raw";
import folders from "lucide-static/icons/folders.svg?raw";
import messageSquare from "lucide-static/icons/message-square.svg?raw";
import plus from "lucide-static/icons/plus.svg?raw";
import settings from "lucide-static/icons/settings.svg?raw";
import sparkles from "lucide-static/icons/sparkles.svg?raw";
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
  groupSessionsByRecency,
  nextSessionClockDelay,
  sessionGroupLabel,
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
  activePage?: "agents" | "skills" | "settings";
  openSkills?: () => void;
  openSettings: () => void;
  sessions: readonly PiSession[];
  selectSession: (agentId: string, sessionId: string) => void;
  /** Unix seconds. Supplying this keeps component and layout tests deterministic. */
  nowSeconds?: number;
  /** Deterministic initial view for tests and app-specific shells. */
  initialGrouping?: SidebarGrouping;
}

type SidebarGrouping = "recent" | "projects";

function RecentSessionRow(props: {
  session: PiSession;
  project: string;
  selected: boolean;
  nowSeconds: number;
  select(): void;
}) {
  return (
    <SidebarMenuButton
      value={`session:${props.session.agentId}:${props.session.sessionId}`}
      class="h-11 px-2"
      aria-label={sessionLabel(props.session)}
      onClick={props.select}
    >
      <Icon
        source={messageSquare}
        size={14}
        class={
          props.selected ? "flex-none text-secondary" : "flex-none text-muted"
        }
      />
      <View class="min-w-0 flex-1 gap-0">
        <Text class="min-w-0 truncate text-sm font-medium">
          {sessionLabel(props.session)}
        </Text>
        <Text
          class={
            props.selected
              ? "min-w-0 truncate text-xs text-secondary"
              : "min-w-0 truncate text-xs text-muted"
          }
        >
          {props.project}
        </Text>
      </View>
      <Text
        class={
          props.selected
            ? "flex-none text-xs text-secondary"
            : "flex-none text-xs text-muted"
        }
      >
        {sessionTimeLabel(props.session.updatedAt, props.nowSeconds)}
      </Text>
    </SidebarMenuButton>
  );
}

export function Sidebar(props: SidebarProps) {
  const [query, setQuery] = createSignal("");
  const [grouping, setGrouping] = createSignal<SidebarGrouping>(
    props.initialGrouping ?? "projects",
  );
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
  const recentSessionGroups = createMemo(() =>
    groupSessionsByRecency(
      visibleAgents().flatMap((agent) => visibleSessions(agent.id)),
      nowSeconds(),
    ),
  );
  const projectName = (agentId: string) =>
    props.agents.find((agent) => agent.id === agentId)?.name ?? agentId;
  return (
    <SidebarRoot class="w-60 border-r border-subtle bg-surface-muted">
      <SidebarHeader class="h-12 border-0 bg-surface-muted flex items-center gap-2 px-4">
        <IconFrame source={bot} size="sm" variant="selected" />
        <Text class="min-w-0 flex-1 text-sm font-semibold text-primary">
          {i18n.message(m.app_name, {})}
        </Text>
      </SidebarHeader>

      <Show when={props.canCreateSession}>
        <View class="flex-none bg-surface-muted px-2 pt-2">
          <SidebarMenuButton
            class="h-9"
            aria-label={i18n.message(m.new_thread, {})}
            onClick={props.newSession}
          >
            <Icon source={messageSquare} size={16} />
            {i18n.message(m.new_thread, {})}
          </SidebarMenuButton>
        </View>
      </Show>

      <Show when={props.sessions.length > 0}>
        <SidebarSearch
          variant="quiet"
          aria-label={i18n.message(m.search_agents, {})}
          value={query()}
          onValueChange={setQuery}
          placeholder={i18n.message(m.search_short, {})}
          clearLabel={i18n.message(m.clear_search, {})}
        />
      </Show>

      <SidebarContent
        role="region"
        aria-label={i18n.message(m.projects, {})}
        contentClass="px-2 py-1 gap-2"
      >
        <SidebarGroup>
          <View class="px-2 pt-2 flex flex-row items-center justify-between gap-2">
            <SidebarGroupLabel class="text-secondary">
              {i18n.message(
                grouping() === "recent" ? m.recent_sessions : m.projects,
                {},
              )}
            </SidebarGroupLabel>
            <View class="flex flex-row items-center gap-0.5">
              <Button
                variant="ghost"
                size="icon"
                class="w-7 h-7"
                aria-label={i18n.message(
                  grouping() === "recent"
                    ? m.group_by_project
                    : m.group_by_recent,
                  {},
                )}
                onClick={() =>
                  setGrouping((value) =>
                    value === "recent" ? "projects" : "recent",
                  )
                }
              >
                <Icon
                  source={grouping() === "recent" ? folders : calendarClock}
                  size={13}
                />
              </Button>
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
          </View>
          <SidebarMenu
            aria-label={i18n.message(m.projects, {})}
            value={activeValue()}
          >
            <Show when={grouping() === "recent"}>
              <ForValue
                each={recentSessionGroups()}
                keyed={(group) => group.key}
              >
                {(group) => (
                  <View class="min-w-0 gap-0.5">
                    <Text class="px-2 pt-2 pb-1 text-xs font-medium text-secondary">
                      {sessionGroupLabel(group().key)}
                    </Text>
                    <ForValue
                      each={group().sessions}
                      keyed={(session) =>
                        `${session.agentId}\0${session.sessionId}`
                      }
                    >
                      {(session) => (
                        <RecentSessionRow
                          session={session()}
                          project={projectName(session().agentId)}
                          selected={
                            activeSessionKey() ===
                            `${session().agentId}\0${session().sessionId}`
                          }
                          nowSeconds={nowSeconds()}
                          select={() =>
                            props.selectSession(
                              session().agentId,
                              session().sessionId,
                            )
                          }
                        />
                      )}
                    </ForValue>
                  </View>
                )}
              </ForValue>
              <Show when={recentSessionGroups().length === 0}>
                <Text role="status" class="px-3 py-4 text-sm text-muted">
                  {i18n.message(m.no_sessions_found, {})}
                </Text>
              </Show>
            </Show>
            <Show when={grouping() === "projects"}>
              <ForValue each={visibleAgents()} keyed={(agent) => agent.id}>
                {(agent) => {
                  const sessions = () => visibleSessions(agent().id);
                  const sessionGroups = () =>
                    groupSessionsByRecency(sessions(), nowSeconds());
                  const expanded = () => sessionsExpanded(agent().id);
                  const showGroupLabels = () => sessionGroups().length > 1;
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
                            <Show
                              when={workspaceDisplayName(
                                agent().cwd,
                                agent().id,
                              )}
                            >
                              {(name) => (
                                <Text class="truncate text-xs text-secondary">
                                  {name()}
                                </Text>
                              )}
                            </Show>
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
                          each={sessionGroups()}
                          keyed={(group) => group.key}
                        >
                          {(group) => (
                            <View class="min-w-0 gap-0.5">
                              <Show when={showGroupLabels()}>
                                <Text class="px-5 pt-1 text-xs font-medium text-muted">
                                  {sessionGroupLabel(group().key)}
                                </Text>
                              </Show>
                              <ForValue
                                each={group().sessions}
                                keyed={(session) => session.sessionId}
                              >
                                {(session) => {
                                  const selected = () =>
                                    activeSessionKey() ===
                                    `${agent().id}\0${session().sessionId}`;
                                  return (
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
                                          class={
                                            selected()
                                              ? "flex-none text-secondary"
                                              : "flex-none text-muted"
                                          }
                                        />
                                        <Text class="min-w-0 flex-1 truncate">
                                          {sessionLabel(session())}
                                        </Text>
                                        <Text
                                          class={
                                            selected()
                                              ? "flex-none text-xs text-secondary"
                                              : "flex-none text-xs text-muted"
                                          }
                                        >
                                          {sessionTimeLabel(
                                            session().updatedAt,
                                            nowSeconds(),
                                          )}
                                        </Text>
                                      </SidebarMenuButton>
                                    </View>
                                  );
                                }}
                              </ForValue>
                            </View>
                          )}
                        </ForValue>
                      </Show>
                    </View>
                  );
                }}
              </ForValue>
            </Show>
          </SidebarMenu>
          <Show
            when={grouping() === "projects" && visibleAgents().length === 0}
          >
            <Text role="status" class="px-3 py-4 text-sm text-muted">
              {i18n.message(m.no_agents_found, {})}
            </Text>
          </Show>
        </SidebarGroup>
      </SidebarContent>

      <SidebarFooter class="border-0 bg-surface-muted px-2 py-2">
        <SidebarMenu value={props.activePage ?? "agents"}>
          <SidebarMenuButton value="skills" onClick={props.openSkills}>
            <Icon source={sparkles} size={16} />
            {i18n.message(m.skills, {})}
          </SidebarMenuButton>
          <SidebarMenuButton value="settings" onClick={props.openSettings}>
            <Icon source={settings} size={16} />
            {i18n.message(m.settings, {})}
          </SidebarMenuButton>
        </SidebarMenu>
      </SidebarFooter>
    </SidebarRoot>
  );
}

export function workspaceName(path: string): string {
  const normalized = path.replace(/[\\/]+$/, "");
  if (!normalized) return i18n.message(m.workspace_not_selected, {});
  return normalized.split(/[\\/]/).at(-1) || normalized;
}

/** Avoid exposing managed storage ids such as `agent-1` as product copy. */
export function workspaceDisplayName(
  path: string,
  agentId: string,
): string | undefined {
  const name = workspaceName(path);
  return name === agentId || name === i18n.message(m.workspace_not_selected, {})
    ? undefined
    : name;
}
