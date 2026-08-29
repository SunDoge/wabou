import { Icon, Text, View } from "@wabou/ui";
import fileCode from "lucide-static/icons/file-code-2.svg?raw";
import terminal from "lucide-static/icons/terminal.svg?raw";
import { For, Show } from "solid-js";
import type { ComposerAutocompleteRow } from "./composer-autocomplete";
import { i18n, m } from "./i18n";

export interface ComposerAutocompleteListProps {
  label: string;
  rows: readonly ComposerAutocompleteRow[];
  highlighted?: string;
  loading?: boolean;
  highlight(id: string): void;
  choose(row: ComposerAutocompleteRow): void;
}

/** Keyboard-oriented completion list shared by the live composer and layout fixtures. */
export function ComposerAutocompleteList(props: ComposerAutocompleteListProps) {
  return (
    <View
      role="listbox"
      aria-label={props.label}
      aria-activedescendant={props.highlighted}
      class="min-w-0 max-h-64 overflow-y-auto flex flex-col gap-0.5"
    >
      <Show
        when={!props.loading}
        fallback={
          <Text role="status" class="px-3 py-3 text-sm text-muted">
            {i18n.message(m.loading_files, {})}
          </Text>
        }
      >
        <For each={props.rows}>
          {(row) => (
            <View
              id={row.id}
              role="option"
              aria-label={row.label}
              aria-selected={props.highlighted === row.id}
              class={
                props.highlighted === row.id
                  ? "min-h-10 px-3 py-1.5 rounded-lg bg-control-hover text-primary flex flex-row items-center gap-2"
                  : "min-h-10 px-3 py-1.5 rounded-lg bg-transparent text-secondary flex flex-row items-center gap-2"
              }
              onPointerMove={() => props.highlight(row.id)}
              onClick={() => props.choose(row)}
            >
              <Icon
                source={row.kind === "command" ? terminal : fileCode}
                size={14}
                class="flex-none"
              />
              <View class="min-w-0 flex-1 flex flex-col">
                <Text class="min-w-0 truncate text-sm">{row.label}</Text>
                <Text class="min-w-0 truncate text-xs text-muted">
                  {row.kind === "command"
                    ? row.description
                    : i18n.message(m.workspace_context, {})}
                </Text>
              </View>
            </View>
          )}
        </For>
      </Show>
    </View>
  );
}
