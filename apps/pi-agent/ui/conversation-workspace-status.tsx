import {
  Icon,
  StatusBar,
  StatusBarGroup,
  StatusBarIndicator,
  Text,
} from "@wabou/ui";
import folder from "lucide-static/icons/folder.svg?raw";
import gitBranch from "lucide-static/icons/git-branch.svg?raw";
import { Show } from "solid-js";
import { match } from "ts-pattern";
import type { AgentConnection } from "./agent-state";
import { i18n, m } from "./i18n";

export function ConversationWorkspaceStatus(props: {
  project: string;
  branch?: string;
  repository: boolean;
  connection: AgentConnection;
  error?: string;
  runtimeLog?: string;
}) {
  const connectionPresentation = () =>
    match(props.connection)
      .with("running", () => ({
        label: i18n.message(m.working, {}),
        tone: "accent" as const,
        textClass: undefined,
      }))
      .with("failed", () => ({
        label: i18n.message(m.agent_status_failed, {}),
        tone: "danger" as const,
        textClass: "text-danger-primary",
      }))
      .with("stopped", () => ({
        label: i18n.message(m.agent_status_stopped, {}),
        tone: "muted" as const,
        textClass: undefined,
      }))
      .with("ready", () => ({
        label: i18n.message(m.local, {}),
        tone: "success" as const,
        textClass: undefined,
      }))
      .exhaustive();
  return (
    <StatusBar
      role={props.connection === "failed" ? "alert" : "status"}
      aria-label={i18n.message(m.workspace_status, {})}
      class="max-w-4xl mx-auto h-6 px-1 gap-3 border-0 bg-transparent"
    >
      <StatusBarGroup shrink class="px-0">
        <Icon source={folder} size={12} class="flex-none" />
        <Text class="min-w-0 max-w-48 truncate">{props.project}</Text>
      </StatusBarGroup>
      <StatusBarGroup class="px-0">
        <StatusBarIndicator tone={connectionPresentation().tone} />
        <Text class={connectionPresentation().textClass}>
          {connectionPresentation().label}
        </Text>
      </StatusBarGroup>
      <Show when={props.error}>
        {(error) => (
          <Text class="min-w-0 flex-1 truncate text-danger-primary">
            {error()}
          </Text>
        )}
      </Show>
      <Show when={!props.error && props.runtimeLog}>
        <Text class="min-w-0 flex-1 truncate text-muted">
          {props.runtimeLog}
        </Text>
      </Show>
      <Show when={props.repository && props.branch}>
        <StatusBarGroup shrink class="px-0">
          <Icon source={gitBranch} size={12} class="flex-none" />
          <Text class="min-w-0 max-w-48 truncate">{props.branch}</Text>
        </StatusBarGroup>
      </Show>
    </StatusBar>
  );
}
