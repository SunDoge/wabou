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
import folder from "lucide-static/icons/folder.svg?raw";
import messageSquare from "lucide-static/icons/message-square.svg?raw";
import plus from "lucide-static/icons/plus.svg?raw";
import settings from "lucide-static/icons/settings.svg?raw";
import { createMemo, createSignal, For as ForValue, Show } from "solid-js";
import { match } from "ts-pattern";
import { AgentSidebarStatus } from "./agent-activity";
import type { PiSession } from "./api";
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
  /** Unix seconds. Supplying this keeps component and layout tests deterministic. */
  nowSeconds?: number;
}

export function sessionLabel(session: Pick<PiSession, "name" | "sessionId">) {
  const name = session.name?.trim();
  return name || session.sessionId.slice(0, 8);
}

export type SessionRecency =
  | { kind: "now" }
  | { kind: "minutes"; value: number }
  | { kind: "hours"; value: number }
  | { kind: "days"; value: number }
  | { kind: "date"; value: Date };

export function sessionRecency(
  updatedAt: number,
  nowSeconds: number,
): SessionRecency {
  const elapsed = Math.max(0, Math.floor(nowSeconds - updatedAt));
  if (elapsed < 60) return { kind: "now" };
  if (elapsed < 3_600) {
    return { kind: "minutes", value: Math.floor(elapsed / 60) };
  }
  if (elapsed < 86_400) {
    return { kind: "hours", value: Math.floor(elapsed / 3_600) };
  }
  if (elapsed < 604_800) {
    return { kind: "days", value: Math.floor(elapsed / 86_400) };
  }
  return { kind: "date", value: new Date(updatedAt * 1_000) };
}

export function sessionTimeLabel(
  updatedAt: number,
  nowSeconds: number,
  locale = i18n.locale(),
): string {
  return match(sessionRecency(updatedAt, nowSeconds))
    .with({ kind: "now" }, () => m.session_now({}, { locale }))
    .with({ kind: "minutes" }, ({ value }) =>
      m.session_minutes_ago({ count: value }, { locale }),
    )
    .with({ kind: "hours" }, ({ value }) =>
      m.session_hours_ago({ count: value }, { locale }),
    )
    .with({ kind: "days" }, ({ value }) =>
      m.session_days_ago({ count: value }, { locale }),
    )
    .with({ kind: "date" }, ({ value }) =>
      new Intl.DateTimeFormat(locale, {
        month: "short",
        day: "numeric",
        year:
          value.getFullYear() === new Date(nowSeconds * 1_000).getFullYear()
            ? undefined
            : "numeric",
      }).format(value),
    )
    .exhaustive();
}

export function sortSessionsByRecency(
  sessions: readonly PiSession[],
): PiSession[] {
  return [...sessions].sort(
    (left, right) =>
      right.updatedAt - left.updatedAt ||
      left.sessionId.localeCompare(right.sessionId),
  );
}

export function activeSidebarValue(
  agents: readonly AgentWorkspace[],
  activeId: string,
  sessions: readonly PiSession[],
): string | undefined {
  const agent = agents.find((candidate) => candidate.id === activeId);
  if (!agent) return undefined;
  const session = sessions.find(
    (session) =>
      session.agentId === agent.id &&
      session.sessionId === agent.state.sessionId,
  );
  return session
    ? `session:${agent.id}:${session.sessionId}`
    : `project:${agent.id}`;
}

export function Sidebar(props: SidebarProps) {
  const [query, setQuery] = createSignal("");
  const nowSeconds = props.nowSeconds ?? Date.now() / 1_000;
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
              {(agent) => (
                <View class="gap-1">
                  <SidebarMenuButton
                    value={`project:${agent().id}`}
                    class="h-10 px-2"
                    aria-label={agent().name}
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
                  <ForValue
                    each={visibleSessions(agent().id)}
                    keyed={(session) => session.sessionId}
                  >
                    {(session) => (
                      <View class="min-w-0 pl-3">
                        <SidebarMenuButton
                          value={`session:${agent().id}:${session().sessionId}`}
                          class="h-8 pl-2 text-sm"
                          aria-label={sessionLabel(session())}
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
                          <Text class="flex-none text-xs text-muted">
                            {sessionTimeLabel(session().updatedAt, nowSeconds)}
                          </Text>
                        </SidebarMenuButton>
                      </View>
                    )}
                  </ForValue>
                </View>
              )}
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
