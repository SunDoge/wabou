import {
  Button,
  CodeBlock,
  Icon,
  Markdown,
  ScrollArea,
  SearchField,
  Text,
  View,
} from "@wabou/ui";
import file from "lucide-static/icons/file.svg?raw";
import filePlus from "lucide-static/icons/file-plus-2.svg?raw";
import x from "lucide-static/icons/x.svg?raw";
import { createEffect, createMemo, createSignal, For as ForValue, Show } from "solid-js";
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
  const [files, setFiles] = createSignal<readonly string[]>([]);
  const [query, setQuery] = createSignal("");
  const [selected, setSelected] = createSignal("");
  const [preview, setPreview] = createSignal<WorkspaceFilePreview>();
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal("");
  let loadRevision = 0;

  createEffect(
    () => props.cwd,
    (cwd) => {
      const revision = ++loadRevision;
      setLoading(true);
      setError("");
      setFiles([]);
      setSelected("");
      setPreview(undefined);
      void props
        .loadFiles(cwd)
        .then((next) => {
          if (revision === loadRevision) setFiles(next);
        })
        .catch((reason) => {
          if (revision === loadRevision) setError(String(reason));
        })
        .finally(() => {
          if (revision === loadRevision) setLoading(false);
        });
    },
  );

  const filtered = createMemo(() => {
    const needle = query().trim().toLowerCase();
    return needle
      ? files().filter((path) => path.toLowerCase().includes(needle))
      : files();
  });

  const choose = (path: string) => {
    const revision = ++loadRevision;
    setSelected(path);
    setPreview(undefined);
    setLoading(true);
    setError("");
    void props
      .readFile(props.cwd, path)
      .then((next) => {
        if (revision === loadRevision) setPreview(next);
      })
      .catch((reason) => {
        if (revision === loadRevision) setError(String(reason));
      })
      .finally(() => {
        if (revision === loadRevision) setLoading(false);
      });
  };

  return (
    <View
      role="region"
      aria-label={i18n.message(m.workspace_files, {})}
      class="w-96 flex-none min-w-0 min-h-0 border-l border-subtle bg-surface flex flex-col shadow-sm"
    >
      <View class="h-14 flex-none px-4 flex flex-row items-center justify-between gap-3 border-b border-subtle">
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
      </View>
      <View class="flex-1 min-h-0 flex flex-col gap-3 p-3">
        <SearchField
          value={query()}
          onValueChange={setQuery}
          aria-label={i18n.message(m.search_workspace_files, {})}
          placeholder={i18n.message(m.search_workspace_files, {})}
        />
        <View class="h-56 flex-none min-h-0 rounded-lg border border-subtle overflow-hidden">
          <ScrollArea class="h-full" contentClass="p-1 gap-0.5">
            <Show
              when={!loading() || files().length > 0}
              fallback={
                <Text role="status" class="p-3 text-sm text-muted">
                  {i18n.message(m.loading_files, {})}
                </Text>
              }
            >
              <ForValue
                each={filtered()}
                fallback={
                  <Text role="status" class="p-3 text-sm text-muted">
                    {i18n.message(m.no_files_found, {})}
                  </Text>
                }
              >
                {(path) => (
                  <Button
                    variant={selected() === path ? "secondary" : "ghost"}
                    class="w-full h-12 min-w-0 justify-start"
                    aria-label={path}
                    onClick={() => choose(path)}
                  >
                    <Icon source={file} size={14} class="flex-none" />
                    <View class="min-w-0 flex flex-col items-start">
                      <Text class="max-w-72 truncate text-sm">
                        {basename(path)}
                      </Text>
                      <Text class="max-w-72 truncate text-xs text-muted">
                        {path}
                      </Text>
                    </View>
                  </Button>
                )}
              </ForValue>
            </Show>
          </ScrollArea>
        </View>
        <View class="flex-1 min-h-0 flex flex-col gap-2">
          <Show when={error()}>
            <Text role="alert" class="text-sm text-danger-primary">
              {error()}
            </Text>
          </Show>
          <Show
            when={preview()}
            fallback={
              <Text class="p-3 text-sm text-muted">
                {i18n.message(m.select_file_preview, {})}
              </Text>
            }
          >
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
          </Show>
        </View>
      </View>
    </View>
  );
}
