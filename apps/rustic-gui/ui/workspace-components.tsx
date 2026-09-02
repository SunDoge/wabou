import { Badge, Button, DirectoryPicker, Icon, Text, View } from "@wabou/ui";
import folder from "lucide-static/icons/folder.svg?raw";
import trash2 from "lucide-static/icons/trash-2.svg?raw";
import { createSignal, For } from "solid-js";

export interface BackupSourcesPanelProps {
  sources: readonly string[];
  disabled?: boolean;
  onChange(sources: string[]): void;
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
    <View class="flex-none p-4 flex flex-col gap-3 border-b border-subtle">
      <View class="flex flex-row items-center justify-between">
        <Text class="font-semibold">Backup folders</Text>
        <Badge variant="secondary">{props.sources.length}</Badge>
      </View>
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
      <For each={props.sources}>
        {(source) => (
          <View class="min-w-0 flex flex-row items-center gap-2 rounded-md bg-surface-muted px-2.5 py-2">
            <Icon source={folder} size={14} class="flex-none text-muted" />
            <Text class="min-w-0 flex-1 truncate text-xs">{source}</Text>
            <Button
              size="icon"
              variant="ghost"
              disabled={props.disabled}
              aria-label={`Remove ${source}`}
              onClick={() =>
                props.onChange(props.sources.filter((item) => item !== source))
              }
            >
              <Icon source={trash2} size={13} />
            </Button>
          </View>
        )}
      </For>
    </View>
  );
}
