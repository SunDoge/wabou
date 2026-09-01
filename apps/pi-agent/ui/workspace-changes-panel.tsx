import {
  Button,
  createLatestAsyncResource,
  DiffViewer,
  Icon,
  ScrollArea,
  Text,
  View,
  WorkbenchInspector,
  WorkbenchInspectorContent,
  WorkbenchInspectorState,
  WorkbenchInspectorTitlebar,
} from "@wabou/ui";
import refreshCw from "lucide-static/icons/refresh-cw.svg?raw";
import { Match, Switch } from "solid-js";
import type { WorkspaceChanges } from "./api";
import { i18n, m } from "./i18n";

export interface WorkspaceChangesPanelProps {
  cwd: string;
  revision?: number;
  load(cwd: string): Promise<WorkspaceChanges>;
  close(): void;
}

export function WorkspaceChangesPanel(props: WorkspaceChangesPanelProps) {
  const changes = createLatestAsyncResource<string, WorkspaceChanges>({
    source: () => `${props.cwd}\0${props.revision ?? 0}`,
    load: (key) => props.load(key.slice(0, key.lastIndexOf("\0"))),
  });

  return (
    <WorkbenchInspector
      projectionBoundary
      role="region"
      aria-label={i18n.message(m.code_changes, {})}
    >
      <WorkbenchInspectorTitlebar
        title={i18n.message(m.code_changes, {})}
        description={props.cwd}
        closeLabel={i18n.message(m.close_code_changes, {})}
        onClose={props.close}
      />
      <WorkbenchInspectorContent>
        <Switch>
          <Match when={changes.error()}>
            {(error) => (
              <WorkbenchInspectorState
                state="error"
                title={i18n.message(m.changes_load_failed, {})}
                description={String(error())}
                renderAction={() => (
                  <Button
                    variant="outline"
                    aria-label={i18n.message(m.retry, {})}
                    onClick={() => void changes.refresh()}
                  >
                    <Icon source={refreshCw} size={14} />
                    {i18n.message(m.retry, {})}
                  </Button>
                )}
              />
            )}
          </Match>
          <Match when={changes.value()}>
            {(value) => (
              <ScrollArea class="flex-1 min-h-0" contentClass="p-3">
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
              </ScrollArea>
            )}
          </Match>
          <Match when={true}>
            <WorkbenchInspectorState
              state="loading"
              title={i18n.message(m.loading_changes, {})}
            />
          </Match>
        </Switch>
      </WorkbenchInspectorContent>
    </WorkbenchInspector>
  );
}
