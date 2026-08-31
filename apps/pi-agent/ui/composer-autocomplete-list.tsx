import { CommandList, Icon } from "@wabou/ui";
import fileCode from "lucide-static/icons/file-code-2.svg?raw";
import terminal from "lucide-static/icons/terminal.svg?raw";
import { createMemo } from "solid-js";
import type { ComposerAutocompleteRow } from "./composer-autocomplete";
import { i18n, m } from "./i18n";

export interface ComposerAutocompleteListProps {
  label: string;
  rows: readonly ComposerAutocompleteRow[];
  highlighted?: string;
  loading?: boolean;
  error?: unknown;
  retry?(): void;
  highlight(id: string): void;
  choose(row: ComposerAutocompleteRow): void;
}

/** Keyboard-oriented completion list shared by the live composer and layout fixtures. */
export function ComposerAutocompleteList(props: ComposerAutocompleteListProps) {
  const items = createMemo(() =>
    props.rows.map((row) => ({
      ...row,
      description:
        row.kind === "command"
          ? row.description
          : i18n.message(m.workspace_context, {}),
    })),
  );
  return (
    <CommandList
      aria-label={props.label}
      items={items()}
      highlighted={props.highlighted}
      loading={props.loading}
      loadingText={i18n.message(m.loading_files, {})}
      error={props.error}
      errorText={i18n.message(m.workspace_files_load_failed, {})}
      retryLabel={i18n.message(m.retry, {})}
      onRetry={props.retry}
      class="max-h-64 overflow-y-auto gap-0.5"
      itemClass="min-h-10"
      onHighlightChange={props.highlight}
      onAction={(id) => {
        const row = props.rows.find((candidate) => candidate.id === id);
        if (row) props.choose(row);
      }}
      renderLeading={(item) => (
        <Icon
          source={item.id.startsWith("command:") ? terminal : fileCode}
          size={14}
          class="flex-none"
        />
      )}
    />
  );
}
