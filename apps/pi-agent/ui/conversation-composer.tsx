import { Button, Icon, Text, TextArea, View, WorkbenchFooter } from "@wabou/ui";
import send from "lucide-static/icons/send.svg?raw";
import { Show } from "solid-js";
import type {
  AgentCommand,
  AgentConnection,
  AgentModel,
  AgentSessionStats,
  AgentThinkingLevel,
} from "./agent-state";
import { CommandPicker } from "./command-picker";
import {
  ComposerContextFiles,
  WorkspaceContextPicker,
} from "./composer-context";
import {
  ComposerDeliveryControl,
  type ComposerDeliveryMode,
} from "./composer-delivery";
import { ComposerImagePicker, ComposerImages } from "./composer-images";
import {
  ExtensionUiChrome,
  type ExtensionUiStatus,
  type ExtensionUiWidget,
} from "./extension-ui";
import { i18n, m } from "./i18n";
import { ModelControls } from "./model-controls";
import { SessionUsage } from "./session-usage";
import { workspaceName } from "./sidebar";

export interface ConversationComposerProps {
  connection: AgentConnection;
  cwd: string;
  draft: string;
  images: readonly string[];
  contextFiles: readonly string[];
  deliveryMode: ComposerDeliveryMode;
  models: readonly AgentModel[];
  modelProvider?: string;
  modelId?: string;
  thinking?: AgentThinkingLevel;
  thinkingLevels: readonly AgentThinkingLevel[];
  commands: readonly AgentCommand[];
  stats?: AgentSessionStats;
  statuses: readonly ExtensionUiStatus[];
  widgets: readonly ExtensionUiWidget[];
  changeDraft(value: string): void;
  changeImages(paths: readonly string[]): void;
  changeContextFiles(paths: readonly string[]): void;
  changeDeliveryMode(value: ComposerDeliveryMode): void;
  chooseModel(provider: string, modelId: string): void;
  chooseThinking(level: AgentThinkingLevel): void;
  loadWorkspaceFiles(cwd: string): Promise<readonly string[]>;
  submit(): void;
}

/** Stable, independently testable boundary for the conversation's primary action. */
export function ConversationComposer(props: ConversationComposerProps) {
  const running = () => props.connection === "running";
  const submit = () => {
    if (props.draft.trim()) props.submit();
  };
  return (
    <WorkbenchFooter class="border-0 bg-canvas px-5 pt-3 pb-5">
      <View
        role="group"
        aria-label={i18n.message(m.prompt_placeholder, {})}
        data-wabou-owns="surface focus-ring"
        class="max-w-3xl mx-auto min-w-0 rounded-xl border border-subtle bg-input shadow-xs px-3 pt-3 pb-2 gap-2"
      >
        <ExtensionUiChrome
          statuses={props.statuses}
          widgets={props.widgets}
          placement="aboveEditor"
        />
        <ComposerImages paths={props.images} change={props.changeImages} />
        <ComposerContextFiles
          paths={props.contextFiles}
          change={props.changeContextFiles}
        />
        <TextArea
          chrome="none"
          class="h-16"
          value={props.draft}
          aria-label={i18n.message(m.prompt_placeholder, {})}
          placeholder={
            running()
              ? i18n.message(m.queue_follow_up, {})
              : i18n.message(m.prompt_placeholder, {})
          }
          onInput={(event) => props.changeDraft(event.currentTarget.value)}
          onKeyDown={(event) => {
            if (event.key === "Enter" && (event.mods & 1) === 0) {
              event.preventDefault();
              submit();
            }
          }}
        />
        <ExtensionUiChrome
          statuses={[]}
          widgets={props.widgets}
          placement="belowEditor"
        />
        <View class="min-w-0 flex flex-row flex-wrap items-center justify-between gap-1.5">
          <View class="min-w-0 flex-1 flex flex-row flex-wrap items-center gap-0.5">
            <ComposerImagePicker
              paths={props.images}
              change={props.changeImages}
            />
            <WorkspaceContextPicker
              cwd={props.cwd}
              paths={props.contextFiles}
              change={props.changeContextFiles}
              loadFiles={props.loadWorkspaceFiles}
            />
            <CommandPicker
              commands={props.commands}
              choose={props.changeDraft}
            />
            <View aria-hidden="true" class="h-5 w-px mx-1 bg-subtle" />
            <Show when={!running()}>
              <ModelControls
                models={props.models}
                modelProvider={props.modelProvider}
                modelId={props.modelId}
                thinking={props.thinking}
                thinkingLevels={props.thinkingLevels}
                chooseModel={props.chooseModel}
                chooseThinking={props.chooseThinking}
              />
            </Show>
            <Show when={running()}>
              <ComposerDeliveryControl
                value={props.deliveryMode}
                change={props.changeDeliveryMode}
              />
            </Show>
          </View>
          <Button
            variant="secondary"
            size="icon"
            class="flex-none rounded-full border border-subtle"
            aria-label={
              running() ? i18n.message(m.queue, {}) : i18n.message(m.send, {})
            }
            disabled={!props.draft.trim()}
            onClick={submit}
          >
            <Icon source={send} size={14} />
          </Button>
        </View>
      </View>
      <View class="max-w-3xl mx-auto min-w-0 px-3 pt-2 flex flex-row flex-wrap items-center justify-between gap-x-3 gap-y-1">
        <Text class="min-w-40 flex-1 truncate text-xs text-muted">
          {workspaceName(props.cwd)} · {i18n.message(m.send_hint, {})}
        </Text>
        <SessionUsage stats={props.stats} />
      </View>
    </WorkbenchFooter>
  );
}
