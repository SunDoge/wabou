import {
  Badge,
  Button,
  Checkbox,
  Dialog,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogScrollBody,
  DialogTitle,
  Icon,
  Input,
  Text,
  TextArea,
  View,
} from "@wabou/ui";
import info from "lucide-static/icons/info.svg?raw";
import { createEffect, createSignal, For, Show } from "solid-js";
import type { SnapshotEntry } from "./api";

export function formatSnapshotTime(value: string): string {
  const match = value.match(/^(\d{4})-(\d{2})-(\d{2})[T ](\d{2}):(\d{2})/);
  return match
    ? `${match[1]}-${match[2]}-${match[3]} ${match[4]}:${match[5]}`
    : value;
}

export function SnapshotDetails(props: {
  snapshot: SnapshotEntry;
  onSave?: (changes: {
    label: string;
    description: string;
    tags: string[];
    deleteProtected: boolean;
  }) => Promise<void> | void;
}) {
  const [label, setLabel] = createSignal("");
  const [description, setDescription] = createSignal("");
  const [tags, setTags] = createSignal("");
  const [deleteProtected, setDeleteProtected] = createSignal(false);
  const [saving, setSaving] = createSignal(false);
  const [error, setError] = createSignal<string>();

  createEffect(
    () => ({
      id: props.snapshot.id,
      label: props.snapshot.label,
      description: props.snapshot.description ?? "",
      tags: props.snapshot.tags.join(", "),
      deleteProtected: props.snapshot.deleteProtected,
    }),
    (snapshot) => {
      setLabel(snapshot.label);
      setDescription(snapshot.description);
      setTags(snapshot.tags);
      setDeleteProtected(snapshot.deleteProtected);
      setError(undefined);
    },
  );

  async function save(close: () => void): Promise<void> {
    if (!props.onSave || saving()) return;
    setSaving(true);
    setError(undefined);
    try {
      await props.onSave({
        label: label(),
        description: description(),
        tags: Array.from(
          new Set(
            tags()
              .split(",")
              .map((tag) => tag.trim())
              .filter(Boolean),
          ),
        ),
        deleteProtected: deleteProtected(),
      });
      close();
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setSaving(false);
    }
  }

  return (
    <Dialog
      aria-label="Snapshot details"
      contentClass="max-h-[720px]"
      trigger={(trigger) => (
        <Button {...trigger} size="sm" variant="ghost">
          <Icon source={info} size={14} /> Details
        </Button>
      )}
    >
      {(dialog) => (
        <View class="min-w-0 flex flex-col gap-4">
          <DialogHeader>
            <DialogTitle>Snapshot details</DialogTitle>
            <DialogDescription>
              Metadata recorded when this backup snapshot was created.
            </DialogDescription>
          </DialogHeader>
          <View class="grid grid-cols-2 gap-4 rounded-lg border border-subtle bg-surface-muted p-4">
            <Detail label="Created" value={props.snapshot.time} />
            <Detail
              label="Hostname"
              value={props.snapshot.hostname || "Unknown"}
            />
            <Detail label="New files" value={String(props.snapshot.filesNew)} />
            <Detail
              label="Changed files"
              value={String(props.snapshot.filesChanged)}
            />
          </View>
          <Show when={props.onSave}>
            <View class="flex flex-col gap-3">
              <View class="flex flex-col gap-1.5">
                <Text class="text-xs font-medium text-muted">Label</Text>
                <Input
                  aria-label="Snapshot label"
                  placeholder="Optional snapshot label"
                  value={label()}
                  onInput={(event) => setLabel(event.currentTarget.value)}
                />
              </View>
              <View class="flex flex-col gap-1.5">
                <Text class="text-xs font-medium text-muted">Tags</Text>
                <Input
                  aria-label="Snapshot tags"
                  placeholder="photos, archive, important"
                  value={tags()}
                  onInput={(event) => setTags(event.currentTarget.value)}
                />
              </View>
              <View class="flex flex-col gap-1.5">
                <Text class="text-xs font-medium text-muted">Description</Text>
                <TextArea
                  aria-label="Snapshot description"
                  class="min-h-20"
                  placeholder="What is important about this snapshot?"
                  value={description()}
                  onInput={(event) => setDescription(event.currentTarget.value)}
                />
              </View>
              <Checkbox
                label="Protect this snapshot from deletion"
                checked={deleteProtected()}
                onCheckedChange={setDeleteProtected}
              />
            </View>
          </Show>
          <View class="min-h-0 flex flex-1 flex-col gap-2">
            <View class="flex flex-row items-center justify-between">
              <Text class="text-sm font-medium">Source paths</Text>
              <Badge variant="secondary">{props.snapshot.paths.length}</Badge>
            </View>
            <DialogScrollBody
              class="max-h-48 rounded-lg border border-subtle"
              contentClass="flex flex-col gap-1.5 p-3"
            >
              <For each={props.snapshot.paths}>
                {(path) => (
                  <Text class="whitespace-normal text-sm text-secondary">
                    {path}
                  </Text>
                )}
              </For>
            </DialogScrollBody>
          </View>
          <View class="flex flex-col gap-1">
            <Text class="text-xs font-medium text-muted">Snapshot ID</Text>
            <Text class="whitespace-normal text-xs text-secondary">
              {props.snapshot.id}
            </Text>
          </View>
          <DialogFooter>
            <Button variant="outline" onClick={dialog.close}>
              {props.onSave ? "Cancel" : "Done"}
            </Button>
            <Show when={props.onSave}>
              <Button
                loading={saving()}
                loadingLabel="Saving…"
                onClick={() => void save(dialog.close)}
              >
                Save changes
              </Button>
            </Show>
          </DialogFooter>
          <Show when={error()}>
            {(message) => (
              <Text role="alert" class="text-sm text-danger-primary">
                {message()}
              </Text>
            )}
          </Show>
        </View>
      )}
    </Dialog>
  );
}

function Detail(props: { label: string; value: string }) {
  return (
    <View class="min-w-0 flex flex-col gap-1">
      <Text class="text-xs font-medium text-muted">{props.label}</Text>
      <Text class="whitespace-normal font-medium">{props.value}</Text>
    </View>
  );
}
