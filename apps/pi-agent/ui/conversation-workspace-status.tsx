import { Icon, Text, View } from "@wabou/ui";
import folder from "lucide-static/icons/folder.svg?raw";
import gitBranch from "lucide-static/icons/git-branch.svg?raw";
import { Show } from "solid-js";
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
  const active = () =>
    props.connection === "ready" || props.connection === "running";
  return (
    <View
      role={props.connection === "failed" ? "alert" : "status"}
      aria-label={i18n.message(m.workspace_status, {})}
      class="w-full max-w-4xl mx-auto min-w-0 h-6 px-1 flex flex-row items-center gap-3 text-xs text-muted"
    >
      <View class="min-w-0 flex flex-row items-center gap-1.5">
        <Icon source={folder} size={12} class="flex-none" />
        <Text class="min-w-0 max-w-48 truncate">{props.project}</Text>
      </View>
      <View class="flex-none flex flex-row items-center gap-1.5">
        <View
          aria-hidden="true"
          class={
            active()
              ? "w-1.5 h-1.5 rounded-full bg-accent"
              : "w-1.5 h-1.5 rounded-full bg-muted"
          }
        />
        <Text
          class={
            props.connection === "failed"
              ? "whitespace-nowrap text-danger-primary"
              : "whitespace-nowrap"
          }
        >
          {i18n.message(
            props.connection === "running"
              ? m.working
              : props.connection === "failed"
                ? m.agent_status_failed
                : props.connection === "stopped"
                  ? m.agent_status_stopped
                  : m.local,
            {},
          )}
        </Text>
      </View>
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
        <View class="min-w-0 flex flex-row items-center gap-1.5">
          <Icon source={gitBranch} size={12} class="flex-none" />
          <Text class="min-w-0 max-w-48 truncate">{props.branch}</Text>
        </View>
      </Show>
    </View>
  );
}
