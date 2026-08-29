import { Button, Icon, View, WorkbenchHeader } from "@wabou/ui";
import filePlus from "lucide-static/icons/file-plus-2.svg?raw";
import folder from "lucide-static/icons/folder.svg?raw";
import gitBranch from "lucide-static/icons/git-branch.svg?raw";
import search from "lucide-static/icons/search.svg?raw";
import square from "lucide-static/icons/square.svg?raw";
import squareTerminal from "lucide-static/icons/square-terminal.svg?raw";
import { type JSX, Show } from "solid-js";
import type { AgentViewState } from "./agent-state";
import { ConversationContext } from "./conversation-context";
import { i18n, m } from "./i18n";
import { SessionActions } from "./session-actions";

export interface ConversationHeaderProps {
  project: string;
  session: string;
  branch?: string;
  state: AgentViewState;
  titleAction?: JSX.Element;
  cwdAvailable: boolean;
  repository: boolean;
  terminalOpen: boolean;
  filesOpen: boolean;
  changesOpen: boolean;
  searchOpen: boolean;
  toggleTerminal(): void;
  toggleFiles(): void;
  toggleChanges(): void;
  toggleSearch(): void;
  newSession(): void;
  compactSession(): void;
  cloneSession(): void;
  exportSession(): void;
  abort(): void;
}

/** Fixed-height conversation chrome with explicit shrink and action groups. */
export function ConversationHeader(props: ConversationHeaderProps) {
  const interactive = () =>
    props.state.connection === "ready" || props.state.connection === "running";
  return (
    <WorkbenchHeader class="bg-canvas border-0 justify-between">
      <ConversationContext
        project={props.project}
        branch={props.branch}
        session={props.session}
        state={props.state}
        titleAction={props.titleAction}
      />
      <Show when={interactive()}>
        <View
          role="toolbar"
          aria-label="Conversation actions"
          class="flex-none flex flex-row items-center gap-1"
        >
          <View class="flex-none flex flex-row items-center gap-0.5">
            <Button
              variant="ghost"
              size="icon"
              aria-label="Toggle terminal"
              aria-pressed={props.terminalOpen}
              disabled={!props.cwdAvailable}
              onClick={props.toggleTerminal}
            >
              <Icon source={squareTerminal} size={15} />
            </Button>
            <Button
              variant="ghost"
              size="icon"
              aria-label={i18n.message(m.workspace_files, {})}
              aria-pressed={props.filesOpen}
              disabled={!props.cwdAvailable}
              onClick={props.toggleFiles}
            >
              <Icon source={folder} size={15} />
            </Button>
            <Show when={props.repository}>
              <Button
                variant="ghost"
                size="icon"
                aria-label={i18n.message(m.code_changes, {})}
                aria-pressed={props.changesOpen}
                onClick={props.toggleChanges}
              >
                <Icon source={gitBranch} size={15} />
              </Button>
            </Show>
          </View>
          <View aria-hidden="true" class="w-px h-5 flex-none bg-subtle" />
          <View class="flex-none flex flex-row items-center gap-0.5">
            <Button
              variant="ghost"
              size="icon"
              aria-label={i18n.message(m.search_transcript, {})}
              disabled={props.state.items.length === 0}
              aria-pressed={props.searchOpen}
              onClick={props.toggleSearch}
            >
              <Icon source={search} size={15} />
            </Button>
            <Show when={props.state.connection === "ready"}>
              <Button
                variant="ghost"
                size="icon"
                aria-label={i18n.message(m.new_session, {})}
                onClick={props.newSession}
              >
                <Icon source={filePlus} size={15} />
              </Button>
              <SessionActions
                disabled={!props.state.sessionId}
                compact={props.compactSession}
                clone={props.cloneSession}
                exportHtml={props.exportSession}
              />
            </Show>
            <Show when={props.state.connection === "running"}>
              <Button variant="outline" onClick={props.abort}>
                <Icon source={square} size={12} /> {i18n.message(m.stop, {})}
              </Button>
            </Show>
          </View>
        </View>
      </Show>
    </WorkbenchHeader>
  );
}
