import {
  Attachment,
  AttachmentAction,
  AttachmentActions,
  AttachmentContent,
  AttachmentDescription,
  AttachmentGroup,
  AttachmentMedia,
  AttachmentTitle,
  Button,
  Command,
  filterCommandItems,
  Icon,
  Popover,
  Text,
  View,
} from "@wabou/ui";
import fileCode from "lucide-static/icons/file-code-2.svg?raw";
import filesIcon from "lucide-static/icons/files.svg?raw";
import x from "lucide-static/icons/x.svg?raw";
import { createMemo, createSignal, For as ForValue, Show } from "solid-js";
import { i18n, m } from "./i18n";

export interface ComposerContextProps {
  paths: readonly string[];
  change(paths: readonly string[]): void;
}

export function ComposerContextFiles(props: ComposerContextProps) {
  return (
    <Show when={props.paths.length > 0}>
      <AttachmentGroup
        role="group"
        aria-label={i18n.message(m.context_files, {})}
      >
        <ForValue each={props.paths}>
          {(path) => (
            <Attachment size="sm" class="max-w-72">
              <AttachmentMedia>
                <Icon source={fileCode} size={14} />
              </AttachmentMedia>
              <AttachmentContent>
                <AttachmentTitle>{path}</AttachmentTitle>
                <AttachmentDescription>
                  {i18n.message(m.workspace_context, {})}
                </AttachmentDescription>
              </AttachmentContent>
              <AttachmentActions>
                <AttachmentAction
                  size="icon"
                  aria-label={i18n.message(m.remove_attachment, { name: path })}
                  onClick={() =>
                    props.change(props.paths.filter((item) => item !== path))
                  }
                >
                  <Icon source={x} size={12} />
                </AttachmentAction>
              </AttachmentActions>
            </Attachment>
          )}
        </ForValue>
      </AttachmentGroup>
    </Show>
  );
}

export function WorkspaceContextPicker(
  props: ComposerContextProps & {
    cwd: string;
    loadFiles(cwd: string): Promise<readonly string[]>;
  },
) {
  const [open, setOpen] = createSignal(false);
  const [query, setQuery] = createSignal("");
  const [workspace, setWorkspace] = createSignal("");
  const [files, setFiles] = createSignal<readonly string[]>([]);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal("");
  const allItems = createMemo(() =>
    files()
      .filter((path) => !props.paths.includes(path))
      .map((path) => ({ id: path, label: path })),
  );
  const visibleItems = createMemo(() =>
    filterCommandItems(allItems(), query()).slice(0, 60),
  );
  const load = async () => {
    if (!props.cwd || workspace() === props.cwd) return;
    setLoading(true);
    setError("");
    try {
      setFiles(await props.loadFiles(props.cwd));
      setWorkspace(props.cwd);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setLoading(false);
    }
  };

  return (
    <Popover
      aria-label={i18n.message(m.add_context_file, {})}
      placement="top-start"
      open={open()}
      onOpenChange={(next) => {
        setOpen(next);
        if (next) void load();
      }}
      contentClass="w-96 max-h-96"
      trigger={(trigger) => (
        <Button
          {...trigger}
          variant="ghost"
          size="icon"
          aria-label={i18n.message(m.add_context_file, {})}
          disabled={!props.cwd || props.paths.length >= 8}
        >
          <Icon source={filesIcon} size={14} />
        </Button>
      )}
    >
      <View class="px-1 pb-2">
        <Text class="font-semibold">{i18n.message(m.context_files, {})}</Text>
        <Text class="text-xs text-muted">
          {i18n.message(m.context_files_detail, {})}
        </Text>
      </View>
      <Show
        when={!loading() && !error()}
        fallback={
          <Text role="status" class="px-3 py-4 text-sm text-muted">
            {loading() ? i18n.message(m.loading_files, {}) : error()}
          </Text>
        }
      >
        <Command
          aria-label={i18n.message(m.search_workspace_files, {})}
          items={visibleItems()}
          query={query()}
          onQueryChange={setQuery}
          placeholder={i18n.message(m.search_workspace_files, {})}
          emptyText={i18n.message(m.no_files_found, {})}
          listClass="max-h-60 overflow-y-auto"
          onAction={(path) => props.change([...props.paths, path])}
          onDismiss={() => setOpen(false)}
        />
      </Show>
    </Popover>
  );
}
