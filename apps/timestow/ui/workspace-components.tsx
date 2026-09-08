import {
  Alert,
  Button,
  Dialog,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DirectoryPicker,
  Icon,
  Text,
  View,
} from "@wabou/ui";
import folder from "lucide-static/icons/folder.svg?raw";
import folderCog from "lucide-static/icons/folder-cog.svg?raw";
import trash2 from "lucide-static/icons/trash-2.svg?raw";
import { createSignal, For as ForValue, Show } from "solid-js";

export interface BackupSourcesPanelProps {
  sources: readonly string[];
  disabled?: boolean;
  onChange(sources: string[]): void | Promise<void>;
}

/** Source editor isolated from repository I/O so its behavior is cheap to test. */
export function BackupSourcesPanel(props: BackupSourcesPanelProps) {
  const [draft, setDraft] = createSignal("");

  function add(candidate = draft()): void {
    const value = candidate.trim();
    if (!value || props.disabled || props.sources.includes(value)) return;
    props.onChange([...props.sources, value]);
    setDraft("");
  }

  return (
    <View
      role="group"
      aria-label="Backup folder selection"
      class="min-w-0 flex flex-col gap-3"
    >
      <DirectoryPicker
        value={draft()}
        onValueChange={setDraft}
        onBrowseSelect={add}
        disabled={props.disabled}
        browseLabel="Browse"
        browseAriaLabel="Choose backup folder"
        aria-label="Backup folder"
        placeholder="Add a folder"
        onKeyDown={(event) => {
          if (event.key !== "Enter") return;
          event.preventDefault();
          add();
        }}
      />
      <Show
        when={props.sources.length > 0}
        fallback={
          <View class="rounded-lg border border-dashed border-subtle bg-surface-muted px-4 py-5">
            <Text class="text-center text-sm text-muted">
              No folders selected. Add one to enable backups.
            </Text>
          </View>
        }
      >
        <View class="min-w-0 flex flex-col overflow-hidden rounded-lg border border-subtle">
          <ForValue each={props.sources}>
            {(source) => (
              <View class="min-w-0 min-h-11 flex flex-row items-center gap-2 border-b border-subtle px-3">
                <Icon source={folder} size={15} class="flex-none text-muted" />
                <Text class="min-w-0 flex-1 truncate text-sm">{source}</Text>
                <Button
                  size="icon"
                  variant="ghost"
                  disabled={props.disabled}
                  aria-label={`Remove ${source}`}
                  onClick={() =>
                    props.onChange(
                      props.sources.filter((item) => item !== source),
                    )
                  }
                >
                  <Icon source={trash2} size={14} />
                </Button>
              </View>
            )}
          </ForValue>
        </View>
      </Show>
    </View>
  );
}

export function BackupSourcesDialog(props: BackupSourcesPanelProps) {
  const [updating, setUpdating] = createSignal(false);
  const [error, setError] = createSignal<string>();

  async function updateSources(sources: string[]): Promise<void> {
    if (updating()) return;
    setUpdating(true);
    setError(undefined);
    try {
      await props.onChange(sources);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setUpdating(false);
    }
  }

  return (
    <Dialog
      aria-label="Manage backup folders"
      closeOnBackdrop={!updating()}
      closeOnEscape={!updating()}
      trigger={(trigger) => (
        <Button
          {...trigger}
          variant="outline"
          disabled={props.disabled}
          aria-label="Manage backup folders"
        >
          <Icon source={folderCog} size={15} />
          {props.sources.length === 1
            ? "1 folder"
            : `${props.sources.length} folders`}
        </Button>
      )}
    >
      <View class="min-w-0 flex flex-col gap-5">
        <DialogHeader>
          <DialogTitle>Backup folders</DialogTitle>
          <DialogDescription>
            Every selected folder is included the next time this profile is
            backed up.
          </DialogDescription>
        </DialogHeader>
        <BackupSourcesPanel
          {...props}
          disabled={props.disabled || updating()}
          onChange={(sources) => void updateSources(sources)}
        />
        <Show when={error()}>
          {(message) => (
            <Alert variant="destructive" title="Could not update folders">
              {message()}
            </Alert>
          )}
        </Show>
      </View>
    </Dialog>
  );
}
