import {
  Button,
  createLatestAsyncResource,
  type Handle,
  Icon,
  moveMenuHighlight,
  Popover,
  Text,
  TextArea,
  View,
  type WabouKeyEvent,
  WorkbenchContentColumn,
  WorkbenchFooter,
} from "@wabou/ui";
import send from "lucide-static/icons/send.svg?raw";
import {
  createEffect,
  createMemo,
  createSignal,
  flush,
  Show,
  untrack,
} from "solid-js";
import type {
  AgentCommand,
  AgentConnection,
  AgentModel,
  AgentSessionStats,
  AgentThinkingLevel,
} from "./agent-state";
import { CommandPicker } from "./command-picker";
import {
  type ComposerAutocompleteRow,
  composerAutocompleteRows,
  detectComposerTrigger,
  normalizeComposerCursor,
  replaceComposerTrigger,
} from "./composer-autocomplete";
import { ComposerAutocompleteList } from "./composer-autocomplete-list";
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

export interface ConversationComposerProps {
  connection: AgentConnection;
  project: string;
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

interface ComposerSelection {
  anchor: number;
  head: number;
}

/** Stable, independently testable boundary for the conversation's primary action. */
export function ConversationComposer(props: ConversationComposerProps) {
  const initialDraft = untrack(() => props.draft);
  const [selection, setSelection] = createSignal<ComposerSelection>({
    anchor: initialDraft.length,
    head: initialDraft.length,
  });
  const [composerActive, setComposerActive] = createSignal(false);
  const [highlighted, setHighlighted] = createSignal<string>();
  const [dismissed, setDismissed] = createSignal<string>();
  let editor: Handle | undefined;
  let authoredDraft = initialDraft;
  const running = () => props.connection === "running";
  const submit = () => {
    if (props.draft.trim()) props.submit();
  };
  const changeDraft = (value: string, nextCursor = value.length) => {
    flush(() => {
      authoredDraft = value;
      props.changeDraft(value);
      setSelection({ anchor: nextCursor, head: nextCursor });
    });
  };
  createEffect(
    () => props.draft,
    (value) => {
      if (value !== authoredDraft) {
        setSelection({ anchor: value.length, head: value.length });
        setComposerActive(false);
        setDismissed(undefined);
      }
      authoredDraft = value;
    },
  );
  const trigger = createMemo(() =>
    composerActive()
      ? detectComposerTrigger(props.draft, selection().head)
      : null,
  );
  const controlledSelection = createMemo(() => ({
    anchor: normalizeComposerCursor(props.draft, selection().anchor),
    head: normalizeComposerCursor(props.draft, selection().head),
  }));
  const triggerKey = createMemo(() => {
    const value = trigger();
    return value
      ? `${value.kind}:${value.start}:${value.end}:${value.query}`
      : undefined;
  });
  const files = createLatestAsyncResource({
    source: () =>
      trigger()?.kind === "file" && props.cwd ? props.cwd : undefined,
    load: (cwd) => props.loadWorkspaceFiles(cwd),
  });
  const autocompleteRows = createMemo(() => {
    const value = trigger();
    if (!value) return [];
    return composerAutocompleteRows(
      value,
      props.commands,
      (files.value() ?? []).filter(
        (path) => !props.contextFiles.includes(path),
      ),
    );
  });
  const autocompleteOpen = createMemo(() => {
    const key = triggerKey();
    return Boolean(
      key &&
        key !== dismissed() &&
        (autocompleteRows().length > 0 ||
          (trigger()?.kind === "file" && files.loading())),
    );
  });
  createEffect(autocompleteRows, (rows) => {
    if (!rows.some((row) => row.id === untrack(highlighted))) {
      setHighlighted(rows[0]?.id);
    }
  });

  const chooseAutocomplete = (row: ComposerAutocompleteRow) => {
    const value = trigger();
    if (!value) return;
    const replacement = replaceComposerTrigger(props.draft, value, row);
    changeDraft(replacement.text, replacement.cursor);
    if (row.kind === "file" && props.contextFiles.length < 8) {
      props.changeContextFiles([...props.contextFiles, row.label]);
    }
    setDismissed(triggerKey());
    editor?.focus();
  };

  const handleComposerKey = (event: WabouKeyEvent) => {
    if (autocompleteOpen()) {
      const rows = autocompleteRows();
      const move =
        event.key === "ArrowDown"
          ? "next"
          : event.key === "ArrowUp"
            ? "previous"
            : undefined;
      if (move) {
        event.preventDefault();
        setHighlighted(moveMenuHighlight(rows, highlighted(), move));
        return;
      }
      if (event.key === "Enter") {
        const row = rows.find((candidate) => candidate.id === highlighted());
        if (row) {
          event.preventDefault();
          chooseAutocomplete(row);
          return;
        }
      }
      if (event.key === "Escape") {
        event.preventDefault();
        setDismissed(triggerKey());
        return;
      }
    }
    if (event.key === "Enter" && (event.mods & 1) === 0) {
      event.preventDefault();
      submit();
    }
  };

  return (
    <WorkbenchFooter class="border-0 bg-canvas px-5 pt-3 pb-5">
      <WorkbenchContentColumn
        role="group"
        aria-label={i18n.message(m.prompt_placeholder, {})}
        data-wabou-owns="surface focus-ring"
        class="max-w-5xl rounded-xl border border-subtle bg-input shadow-xs px-3 pt-3 pb-2 gap-2"
      >
        <View class="w-full min-w-0 flex flex-row justify-end">
          <SessionUsage stats={props.stats} />
        </View>
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
        <Popover
          contentRole="presentation"
          popupRole="listbox"
          placement="top-start"
          open={autocompleteOpen()}
          onOpenChange={(open) => {
            if (!open) setDismissed(triggerKey());
          }}
          outsidePointerStrategy="passthrough"
          restoreFocus={false}
          contentClass="w-96 max-h-72 p-1.5"
          trigger={(popoverTrigger) => (
            <TextArea
              ref={(node) => {
                editor = node;
                popoverTrigger.ref(node);
              }}
              aria-haspopup={popoverTrigger["aria-haspopup"]}
              aria-expanded={popoverTrigger["aria-expanded"]}
              aria-activedescendant={highlighted()}
              chrome="none"
              class="h-16"
              value={props.draft}
              widgetConfig={{
                selection: controlledSelection(),
              }}
              aria-label={i18n.message(m.prompt_placeholder, {})}
              placeholder={
                running()
                  ? i18n.message(m.queue_follow_up, {})
                  : i18n.message(m.prompt_placeholder, {})
              }
              onInput={(event) => {
                const value = event.currentTarget.value;
                setComposerActive(true);
                setDismissed(undefined);
                changeDraft(value);
              }}
              onTextSelectionChange={(event) => {
                setComposerActive(true);
                const head = event.head ?? 0;
                setSelection({ anchor: event.anchor ?? head, head });
                setDismissed(undefined);
              }}
              onFocus={() => setComposerActive(true)}
              onKeyDown={handleComposerKey}
            />
          )}
        >
          <ComposerAutocompleteList
            label={
              trigger()?.kind === "command"
                ? i18n.message(m.available_commands, {})
                : i18n.message(m.context_files, {})
            }
            rows={autocompleteRows()}
            highlighted={highlighted()}
            loading={files.loading() && trigger()?.kind === "file"}
            highlight={setHighlighted}
            choose={chooseAutocomplete}
          />
        </Popover>
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
            <CommandPicker commands={props.commands} choose={changeDraft} />
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
        <View class="min-w-0 border-t border-subtle pt-2 px-1">
          <Text class="w-full min-w-0 truncate text-xs text-secondary">
            {props.project} · {i18n.message(m.send_hint, {})}
          </Text>
        </View>
      </WorkbenchContentColumn>
    </WorkbenchFooter>
  );
}
