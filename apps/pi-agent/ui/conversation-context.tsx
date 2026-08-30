import {
  Badge,
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
  Text,
  View,
} from "@wabou/ui";
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
    <View class="min-w-0 flex-1 overflow-hidden flex flex-row items-center gap-2">
      <Breadcrumb aria-label={label()} class="min-w-0 flex-1 overflow-hidden">
        <BreadcrumbList class="flex-nowrap overflow-hidden gap-1">
          <BreadcrumbItem class="max-w-28 flex-none">
            <Text class="min-w-0 truncate text-xs font-medium text-secondary">
              {props.project}
            </Text>
          </BreadcrumbItem>
          <BreadcrumbSeparator class="w-3" />
          <Show when={props.branch}>
            {(branch) => (
              <>
                <BreadcrumbItem class="flex-none">
                  <Badge variant="outline" class="h-5 px-2 text-xs">
                    {compactBranchLabel(branch())}
                  </Badge>
                </BreadcrumbItem>
                <BreadcrumbSeparator class="w-3" />
              </>
            )}
          </Show>
          <BreadcrumbItem class="min-w-0 flex-1 overflow-hidden">
            <BreadcrumbPage class="truncate text-sm font-semibold">
              {props.session}
            </BreadcrumbPage>
          </BreadcrumbItem>
        </BreadcrumbList>
      </Breadcrumb>
      {props.titleAction}
      <AgentActivityStatus state={props.state} />
    </View>
  );
}
