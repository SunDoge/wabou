import {
  Button,
  CodeBlock,
  createLatestAsyncResource,
  Icon,
  Listbox,
  Markdown,
  ScrollArea,
  SearchField,
  Text,
  View,
  WorkbenchInspector,
  WorkbenchInspectorContent,
  WorkbenchInspectorHeader,
  WorkbenchInspectorState,
} from "@wabou/ui";
import file from "lucide-static/icons/file.svg?raw";
import filePlus from "lucide-static/icons/file-plus-2.svg?raw";
import x from "lucide-static/icons/x.svg?raw";
import {
  createEffect,
  createMemo,
  createSignal,
  Match,
  Show,
  Switch,
} from "solid-js";
import type { WorkspaceFilePreview } from "./api";
import { i18n, m } from "./i18n";

export interface WorkspacePanelProps {
  cwd: string;
  loadFiles(cwd: string): Promise<readonly string[]>;
  readFile(cwd: string, path: string): Promise<WorkspaceFilePreview>;
  addContext(path: string): void;
  close(): void;
}

const basename = (path: string) => path.split(/[\\/]/).at(-1) ?? path;
const extension = (path: string) => path.split(".").at(-1)?.toLowerCase();

export function WorkspacePanel(props: WorkspacePanelProps) {
  const [query, setQuery] = createSignal("");
  const [selected, setSelected] = createSignal("");
  let currentCwd: string | undefined;

  const files = createLatestAsyncResource({
    source: () => props.cwd,
    load: (cwd) => props.loadFiles(cwd),
  });
  const previewKey = createMemo<readonly [string, string] | undefined>(() => {
    const path = selected();
    return path ? [props.cwd, path] : undefined;
  });
  const preview = createLatestAsyncResource({
    source: previewKey,
    load: ([cwd, path]) => props.readFile(cwd, path),
  });
  createEffect(
    () => props.cwd,
    (cwd) => {
      if (cwd === currentCwd) return;
      currentCwd = cwd;
      setSelected("");
    },
  );

  const filtered = createMemo(() => {
    const needle = query().trim().toLowerCase();
    const available = files.value() ?? [];
    return needle
      ? available.filter((path) => path.toLowerCase().includes(needle))
      : available;
  });
  const fileOptions = createMemo(() =>
    filtered().map((path) => ({
      value: path,
      label: basename(path),
      accessibilityLabel: path,
      description: path,
    })),
  );

  const choose = (path: string) => {
    setSelected(path);
  };

  return (
    <WorkbenchInspector
      projectionBoundary
      role="region"
      aria-label={i18n.message(m.workspace_files, {})}
    >
      <WorkbenchInspectorHeader>
        <View class="min-w-0 flex flex-col">
          <Text class="font-semibold">
            {i18n.message(m.workspace_files, {})}
          </Text>
          <Text class="max-w-72 truncate text-xs text-muted">{props.cwd}</Text>
        </View>
        <Button
          variant="ghost"
          size="icon"
          aria-label={i18n.message(m.close_workspace_files, {})}
          onClick={props.close}
        >
          <Icon source={x} size={15} />
        </Button>
      </WorkbenchInspectorHeader>
      <WorkbenchInspectorContent class="gap-3 p-3">
        <SearchField
          value={query()}
          onValueChange={setQuery}
          aria-label={i18n.message(m.search_workspace_files, {})}
          placeholder={i18n.message(m.search_workspace_files, {})}
        />
        <View class="h-56 flex-none min-h-0 flex flex-col rounded-lg border border-subtle overflow-hidden">
          <Switch>
            <Match when={files.loading() && (files.value()?.length ?? 0) === 0}>
              <WorkbenchInspectorState
                state="loading"
                title={i18n.message(m.loading_files, {})}
                class="p-4"
              />
            </Match>
            <Match when={files.error()}>
              {(error) => (
                <WorkbenchInspectorState
                  state="error"
                  title={i18n.message(m.workspace_files_load_failed, {})}
                  description={String(error())}
                  class="p-4"
                  action={{
                    label: i18n.message(m.retry, {}),
                    onAction: () => void files.refresh(),
                  }}
                />
              )}
            </Match>
            <Match when={true}>
              <Listbox
                fill
                aria-label={i18n.message(m.workspace_files, {})}
                options={fileOptions()}
                value={selected()}
                itemHeight={48}
                listClass="p-1 gap-0.5"
                itemClass="rounded-md"
                emptyText={i18n.message(m.no_files_found, {})}
                renderLeading={() => (
                  <Icon source={file} size={14} class="flex-none text-muted" />
                )}
                onAction={choose}
              />
            </Match>
          </Switch>
        </View>
        <View class="flex-1 min-h-0 flex flex-col gap-2">
          <Switch>
            <Match when={preview.error()}>
              {(error) => (
                <WorkbenchInspectorState
                  state="error"
                  title={i18n.message(m.file_preview_failed, {})}
                  description={String(error())}
                  class="p-4"
                  action={{
                    label: i18n.message(m.retry, {}),
                    onAction: () => void preview.refresh(),
                  }}
                />
              )}
            </Match>
            <Match when={preview.loading()}>
              <WorkbenchInspectorState
                state="loading"
                title={i18n.message(m.loading_file_preview, {})}
                class="p-4"
              />
            </Match>
            <Match when={preview.value()}>
              {(value) => (
                <>
                  <View class="flex-none flex flex-row items-center justify-between gap-2">
                    <Text class="min-w-0 flex-1 truncate text-sm font-medium">
                      {value().path}
                    </Text>
                    <Button
                      size="sm"
                      onClick={() => props.addContext(value().path)}
                    >
                      <Icon source={filePlus} size={13} />
                      {i18n.message(m.add_to_context, {})}
                    </Button>
                  </View>
                  <ScrollArea class="flex-1 min-h-0">
                    <Show
                      when={extension(value().path) === "md"}
                      fallback={
                        <CodeBlock
                          code={value().text}
                          language={extension(value().path) ?? "text"}
                          copyable={false}
                        />
                      }
                    >
                      <Markdown source={value().text} class="p-3" />
                    </Show>
                  </ScrollArea>
                </>
              )}
            </Match>
            <Match when={true}>
              <WorkbenchInspectorState
                state="empty"
                title={i18n.message(m.select_file_preview, {})}
                class="p-4"
                renderMedia={() => (
                  <View class="w-10 h-10 flex-none rounded-lg bg-control text-primary flex items-center justify-center">
                    <Icon source={file} size={18} />
                  </View>
                )}
              />
            </Match>
          </Switch>
        </View>
      </WorkbenchInspectorContent>
    </WorkbenchInspector>
  );
}
