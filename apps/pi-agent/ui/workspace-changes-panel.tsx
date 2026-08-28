import { Button, DiffViewer, Icon, ScrollArea, Text, View } from "@wabou/ui";
import x from "lucide-static/icons/x.svg?raw";
import { createEffect, createSignal, Show } from "solid-js";
import type { WorkspaceChanges } from "./api";
import { i18n, m } from "./i18n";

export interface WorkspaceChangesPanelProps {
  cwd: string;
  revision?: number;
  load(cwd: string): Promise<WorkspaceChanges>;
  close(): void;
}

export function WorkspaceChangesPanel(props: WorkspaceChangesPanelProps) {
  const [changes, setChanges] = createSignal<WorkspaceChanges>();
  const [error, setError] = createSignal("");
  let revision = 0;

  createEffect(
    () => `${props.cwd}\0${props.revision ?? 0}`,
    () => {
      const cwd = props.cwd;
      const request = ++revision;
      setChanges(undefined);
      setError("");
      void props
        .load(cwd)
        .then((value) => {
          if (request === revision) setChanges(value);
        })
        .catch((reason) => {
          if (request === revision) setError(String(reason));
        });
    },
  );

  return (
    <View
      role="region"
      aria-label={i18n.message(m.code_changes, {})}
      class="w-96 flex-none min-w-0 min-h-0 border-l border-subtle bg-surface flex flex-col shadow-sm"
    >
      <View class="h-14 flex-none px-4 flex flex-row items-center justify-between gap-3 border-b border-subtle">
        <View class="min-w-0 flex flex-col">
          <Text class="font-semibold">{i18n.message(m.code_changes, {})}</Text>
          <Text class="max-w-72 truncate text-xs text-muted">{props.cwd}</Text>
        </View>
        <Button
          variant="ghost"
          size="icon"
          aria-label={i18n.message(m.close_code_changes, {})}
          onClick={props.close}
        >
          <Icon source={x} size={15} />
        </Button>
      </View>
      <ScrollArea class="flex-1 min-h-0" contentClass="p-3">
        <Show
          when={!error()}
          fallback={
            <Text role="alert" class="text-sm text-danger-primary">
              {error()}
            </Text>
          }
        >
          <Show
            when={changes()}
            fallback={
              <Text role="status" class="p-3 text-sm text-muted">
                {i18n.message(m.loading_changes, {})}
              </Text>
            }
          >
            {(value) => (
              <DiffViewer
                files={value().files}
                labels={{
                  filesChanged: (count) =>
                    count === 1
                      ? i18n.message(m.one_file_changed, {})
                      : i18n.message(m.files_changed, { count }),
                  additions: (count) => i18n.message(m.additions, { count }),
                  deletions: (count) => i18n.message(m.deletions, { count }),
                  empty: i18n.message(m.no_code_changes, {}),
                  technicalDetails: i18n.message(m.technical_diff, {}),
                }}
              />
            )}
          </Show>
        </Show>
      </ScrollArea>
    </View>
  );
}
