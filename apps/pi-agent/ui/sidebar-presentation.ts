import { match } from "ts-pattern";
import type { PiSession } from "./api";
import { i18n, m } from "./i18n";
import type { AgentWorkspace } from "./workspace";

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

export type SessionRecencyGroup = "today" | "previous-seven-days" | "older";

export interface GroupedSessions {
  key: SessionRecencyGroup;
  sessions: PiSession[];
}

/** Groups a project's history into stable, user-scannable time ranges. */
export function groupSessionsByRecency(
  sessions: readonly PiSession[],
  nowSeconds: number,
): GroupedSessions[] {
  const groups = new Map<SessionRecencyGroup, PiSession[]>();
  const startOfToday = new Date(nowSeconds * 1_000);
  startOfToday.setHours(0, 0, 0, 0);
  const todaySeconds = Math.floor(startOfToday.getTime() / 1_000);
  const previousWeekSeconds = todaySeconds - 6 * 86_400;

  for (const session of sortSessionsByRecency(sessions)) {
    const key: SessionRecencyGroup =
      session.updatedAt >= todaySeconds
        ? "today"
        : session.updatedAt >= previousWeekSeconds
          ? "previous-seven-days"
          : "older";
    const group = groups.get(key) ?? [];
    group.push(session);
    groups.set(key, group);
  }

  return (["today", "previous-seven-days", "older"] as const).flatMap((key) => {
    const grouped = groups.get(key);
    return grouped ? [{ key, sessions: grouped }] : [];
  });
}

export function sessionGroupLabel(
  group: SessionRecencyGroup,
  locale = i18n.locale(),
): string {
  return match(group)
    .with("today", () => m.session_group_today({}, { locale }))
    .with("previous-seven-days", () =>
      m.session_group_previous_week({}, { locale }),
    )
    .with("older", () => m.session_group_older({}, { locale }))
    .exhaustive();
}

/** Seconds until any relative session label can visibly change. */
export function nextSessionClockDelay(
  sessions: readonly Pick<PiSession, "updatedAt">[],
  nowSeconds: number,
): number | undefined {
  let next: number | undefined;
  for (const session of sessions) {
    const elapsed = Math.max(0, Math.floor(nowSeconds - session.updatedAt));
    const step = elapsed < 3_600 ? 60 : elapsed < 604_800 ? 3_600 : 86_400;
    const delay = Math.max(1, step - (elapsed % step));
    next = next === undefined ? delay : Math.min(next, delay);
  }
  return next;
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
