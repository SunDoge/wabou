import { Badge, Text, View } from "@wabou/ui";
import { type JSX, Show } from "solid-js";
import { AgentActivityStatus } from "./agent-activity";
import type { AgentViewState } from "./agent-state";

export interface ConversationContextProps {
  project: string;
  session: string;
  branch?: string;
  state: AgentViewState;
  titleAction?: JSX.Element;
}

export function compactBranchLabel(branch: string): string {
  const maximum = 28;
  return branch.length <= maximum ? branch : `…${branch.slice(1 - maximum)}`;
}

/** A compact project / branch / session breadcrumb for the conversation chrome. */
export function ConversationContext(props: ConversationContextProps) {
  const label = () =>
    [props.project, props.branch, props.session].filter(Boolean).join(", ");
  return (
    <View
      role="group"
      aria-label={label()}
      class="min-w-0 flex-1 overflow-hidden flex flex-row items-center gap-2"
    >
      <Text class="max-w-28 min-w-0 flex-none truncate text-xs font-semibold text-muted">
        {props.project}
      </Text>
      <Text aria-hidden="true" class="flex-none text-xs text-muted">
        /
      </Text>
      <Show when={props.branch}>
        {(branch) => (
          <>
            <Badge variant="outline" class="h-5 flex-none px-2 text-xs">
              {compactBranchLabel(branch())}
            </Badge>
            <Text aria-hidden="true" class="flex-none text-xs text-muted">
              /
            </Text>
          </>
        )}
      </Show>
      <Text class="min-w-0 truncate text-sm font-semibold text-primary">
        {props.session}
      </Text>
      {props.titleAction}
      <AgentActivityStatus state={props.state} />
    </View>
  );
}
