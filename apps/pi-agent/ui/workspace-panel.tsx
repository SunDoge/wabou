import {
  Alert,
  Button,
  CodeBlock,
  createLatestAsyncResource,
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  Icon,
  Listbox,
  Markdown,
  ScrollArea,
  SearchField,
  Spinner,
  Text,
  View,
  WorkbenchInspector,
  WorkbenchInspectorContent,
  WorkbenchInspectorHeader,
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
  const [preview, setPreview] = createSignal<WorkspaceFilePreview>();
  const [previewLoading, setPreviewLoading] = createSignal(false);
  const [previewError, setPreviewError] = createSignal("");
  let previewRevision = 0;
  let currentCwd: string | undefined;

  const files = createLatestAsyncResource({
    source: () => props.cwd,
    load: (cwd) => props.loadFiles(cwd),
  });
  createEffect(
    () => props.cwd,
    (cwd) => {
      if (cwd === currentCwd) return;
      currentCwd = cwd;
      previewRevision++;
      setSelected("");
      setPreview(undefined);
      setPreviewLoading(false);
      setPreviewError("");
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
    const revision = ++previewRevision;
    setSelected(path);
    setPreview(undefined);
    setPreviewLoading(true);
    setPreviewError("");
    void props
      .readFile(props.cwd, path)
      .then((next) => {
        if (revision === previewRevision) setPreview(next);
      })
      .catch((reason) => {
        if (revision === previewRevision) setPreviewError(String(reason));
      })
      .finally(() => {
        if (revision === previewRevision) setPreviewLoading(false);
      });
  };

  return (
    <WorkbenchInspector
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
        <View class="h-56 flex-none min-h-0 rounded-lg border border-subtle overflow-hidden">
          <Show
            when={!files.loading() || (files.value()?.length ?? 0) > 0}
            fallback={
              <Text role="status" class="p-3 text-sm text-muted">
                {i18n.message(m.loading_files, {})}
              </Text>
            }
          >
            <Listbox
              aria-label={i18n.message(m.workspace_files, {})}
              options={fileOptions()}
              value={selected()}
              viewportHeight={224}
              itemHeight={48}
              listClass="p-1 gap-0.5"
              itemClass="rounded-md"
              emptyText={i18n.message(m.no_files_found, {})}
              renderLeading={() => (
                <Icon source={file} size={14} class="flex-none text-muted" />
              )}
              onAction={choose}
            />
          </Show>
        </View>
        <View class="flex-1 min-h-0 flex flex-col gap-2">
          <Switch>
            <Match when={files.error() ?? previewError()}>
              {(error) => (
                <Alert
                  variant="destructive"
                  title={i18n.message(m.file_preview_failed, {})}
                >
                  {String(error())}
                </Alert>
              )}
            </Match>
            <Match when={previewLoading()}>
              <Empty
                variant="plain"
                role="status"
                aria-label={i18n.message(m.loading_file_preview, {})}
                class="h-full p-4 gap-3"
              >
                <EmptyMedia>
                  <Spinner decorative />
                </EmptyMedia>
                <EmptyHeader>
                  <EmptyDescription maxLines={1} class="min-h-0">
                    {i18n.message(m.loading_file_preview, {})}
                  </EmptyDescription>
                </EmptyHeader>
              </Empty>
            </Match>
            <Match when={preview()}>
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
              <Empty variant="plain" class="h-full p-4 gap-3">
                <EmptyMedia variant="icon">
                  <Icon source={file} size={18} />
                </EmptyMedia>
                <EmptyHeader>
                  <EmptyDescription maxLines={2}>
                    {i18n.message(m.select_file_preview, {})}
                  </EmptyDescription>
                </EmptyHeader>
              </Empty>
            </Match>
          </Switch>
        </View>
      </WorkbenchInspectorContent>
    </WorkbenchInspector>
  );
}
