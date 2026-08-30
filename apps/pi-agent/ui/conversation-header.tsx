import { Button, Icon, View, WorkbenchHeader } from "@wabou/ui";
import chevronLeft from "lucide-static/icons/chevron-left.svg?raw";
import chevronRight from "lucide-static/icons/chevron-right.svg?raw";
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
  canGoBack?: boolean;
  canGoForward?: boolean;
  goBack?(): void;
  goForward?(): void;
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
      <View class="min-w-0 flex-1 flex flex-row items-center gap-1">
        <View
          role="group"
          aria-label={i18n.message(m.session_navigation, {})}
          class="flex-none flex flex-row items-center gap-0"
        >
          <Button
            variant="ghost"
            size="icon"
            aria-label={i18n.message(m.previous_session, {})}
            disabled={!props.canGoBack}
            onClick={props.goBack}
          >
            <Icon source={chevronLeft} size={15} />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            aria-label={i18n.message(m.next_session, {})}
            disabled={!props.canGoForward}
            onClick={props.goForward}
          >
            <Icon source={chevronRight} size={15} />
          </Button>
        </View>
        <ConversationContext
          project={props.project}
          branch={props.branch}
          session={props.session}
          state={props.state}
          titleAction={props.titleAction}
        />
      </View>
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
