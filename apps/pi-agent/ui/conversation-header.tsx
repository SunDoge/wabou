import {
  Icon,
  Toolbar,
  ToolbarButton,
  ToolbarGroup,
  ToolbarSeparator,
  ToolbarToggle,
  View,
  WorkbenchHeader,
} from "@wabou/ui";
import chevronLeft from "lucide-static/icons/chevron-left.svg?raw";
import chevronRight from "lucide-static/icons/chevron-right.svg?raw";
import ellipsis from "lucide-static/icons/ellipsis.svg?raw";
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
import { type SessionAction, SessionActions } from "./session-actions";

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
  newSessionPending?: boolean;
  compactSession(): void | Promise<void>;
  cloneSession(): void | Promise<void>;
  exportSession(): void | Promise<void>;
  sessionActionError?: (action: SessionAction, error: unknown) => void;
  abort(): void | Promise<void>;
  abortPending?: boolean;
}

/** Fixed-height conversation chrome with explicit shrink and action groups. */
export function ConversationHeader(props: ConversationHeaderProps) {
  return (
    <WorkbenchHeader class="bg-canvas border-0 justify-between">
      <View class="min-w-0 flex-1 flex flex-row items-center gap-1">
        <Toolbar
          aria-label={i18n.message(m.session_navigation, {})}
          class="border-0 bg-transparent p-0 gap-0"
        >
          <ToolbarButton
            variant="ghost"
            size="icon"
            aria-label={i18n.message(m.previous_session, {})}
            disabled={!props.canGoBack}
            onClick={props.goBack}
          >
            <Icon source={chevronLeft} size={15} />
          </ToolbarButton>
          <ToolbarButton
            variant="ghost"
            size="icon"
            aria-label={i18n.message(m.next_session, {})}
            disabled={!props.canGoForward}
            onClick={props.goForward}
          >
            <Icon source={chevronRight} size={15} />
          </ToolbarButton>
        </Toolbar>
        <ConversationContext
          project={props.project}
          branch={props.branch}
          session={props.session}
          state={props.state}
          titleAction={props.titleAction}
        />
      </View>
      <Toolbar
        aria-label="Conversation actions"
        class="border-0 bg-transparent p-0 gap-1"
      >
        <ToolbarGroup>
          <ToolbarToggle
            size="icon"
            aria-label="Toggle terminal"
            pressed={props.terminalOpen}
            disabled={!props.cwdAvailable}
            onPressedChange={props.toggleTerminal}
          >
            <Icon source={squareTerminal} size={15} />
          </ToolbarToggle>
          <ToolbarToggle
            size="icon"
            aria-label={i18n.message(m.workspace_files, {})}
            pressed={props.filesOpen}
            disabled={!props.cwdAvailable}
            onPressedChange={props.toggleFiles}
          >
            <Icon source={folder} size={15} />
          </ToolbarToggle>
          <Show when={props.repository}>
            <ToolbarToggle
              size="icon"
              aria-label={i18n.message(m.code_changes, {})}
              pressed={props.changesOpen}
              onPressedChange={props.toggleChanges}
            >
              <Icon source={gitBranch} size={15} />
            </ToolbarToggle>
          </Show>
        </ToolbarGroup>
        <ToolbarSeparator />
        <ToolbarGroup>
          <ToolbarToggle
            size="icon"
            aria-label={i18n.message(m.search_transcript, {})}
            disabled={props.state.items.length === 0}
            pressed={props.searchOpen}
            onPressedChange={props.toggleSearch}
          >
            <Icon source={search} size={15} />
          </ToolbarToggle>
          <Show when={props.state.connection === "ready"}>
            <ToolbarButton
              size="icon"
              aria-label={i18n.message(m.new_session, {})}
              disabled={props.newSessionPending}
              onClick={props.newSession}
            >
              <Icon source={filePlus} size={15} />
            </ToolbarButton>
            <SessionActions
              disabled={!props.state.sessionId}
              compact={props.compactSession}
              clone={props.cloneSession}
              exportHtml={props.exportSession}
              onActionError={props.sessionActionError}
              trigger={(trigger) => (
                <ToolbarButton
                  {...trigger}
                  size="icon"
                  aria-label={i18n.message(m.session_actions, {})}
                >
                  <Icon source={ellipsis} size={16} />
                </ToolbarButton>
              )}
            />
          </Show>
          <Show when={props.state.connection === "running"}>
            <ToolbarButton
              variant="outline"
              disabled={props.abortPending}
              loading={props.abortPending}
              onClick={props.abort}
            >
              <Icon source={square} size={12} /> {i18n.message(m.stop, {})}
            </ToolbarButton>
          </Show>
        </ToolbarGroup>
      </Toolbar>
    </WorkbenchHeader>
  );
}
