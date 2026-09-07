import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
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
import trash2 from "lucide-static/icons/trash-2.svg?raw";
import { createEffect, createSignal, For as ForValue, Show } from "solid-js";
import type { SnapshotEntry } from "./api";
export { formatTimestamp as formatSnapshotTime } from "./format";

export function SnapshotDetails(props: {
  snapshot: SnapshotEntry;
  onSave?: (changes: {
    label: string;
    description: string;
    tags: string[];
    deleteProtected: boolean;
  }) => Promise<void> | void;
  onDelete?: () => Promise<void> | void;
}) {
  const [label, setLabel] = createSignal("");
  const [description, setDescription] = createSignal("");
  const [tags, setTags] = createSignal("");
  const [deleteProtected, setDeleteProtected] = createSignal(false);
  const [saving, setSaving] = createSignal(false);
  const [error, setError] = createSignal<string>();
  const [deleteOpen, setDeleteOpen] = createSignal(false);
  const [deleting, setDeleting] = createSignal(false);
  const [deleteError, setDeleteError] = createSignal<string>();

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

  async function deleteSnapshot(): Promise<void> {
    if (!props.onDelete || deleting() || props.snapshot.deleteProtected) return;
    setDeleting(true);
    setDeleteError(undefined);
    try {
      await props.onDelete();
      setDeleteOpen(false);
    } catch (cause) {
      setDeleteError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setDeleting(false);
    }
  }

  return (
    <>
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
              <Detail
                label="New files"
                value={String(props.snapshot.filesNew)}
              />
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
                  <Text class="text-xs font-medium text-muted">
                    Description
                  </Text>
                  <TextArea
                    aria-label="Snapshot description"
                    class="min-h-20"
                    placeholder="What is important about this snapshot?"
                    value={description()}
                    onInput={(event) =>
                      setDescription(event.currentTarget.value)
                    }
                  />
                </View>
                <Checkbox
                  label="Protect this snapshot from deletion"
                  checked={deleteProtected()}
                  onCheckedChange={setDeleteProtected}
                />
                <Show when={props.onDelete && props.snapshot.deleteProtected}>
                  <Text class="text-xs text-muted">
                    Turn off protection and save before deleting this snapshot.
                  </Text>
                </Show>
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
                <ForValue each={props.snapshot.paths}>
                  {(path) => (
                    <Text class="whitespace-normal text-sm text-secondary">
                      {path}
                    </Text>
                  )}
                </ForValue>
              </DialogScrollBody>
            </View>
            <View class="flex flex-col gap-1">
              <Text class="text-xs font-medium text-muted">Snapshot ID</Text>
              <Text class="whitespace-normal text-xs text-secondary">
                {props.snapshot.id}
              </Text>
            </View>
            <DialogFooter>
              <Show when={props.onDelete}>
                <Button
                  variant="ghost"
                  class="mr-auto"
                  disabled={props.snapshot.deleteProtected}
                  aria-label={
                    props.snapshot.deleteProtected
                      ? "Protected snapshot cannot be deleted"
                      : "Delete snapshot"
                  }
                  onClick={() => {
                    dialog.close();
                    setDeleteError(undefined);
                    setDeleteOpen(true);
                  }}
                >
                  <Icon source={trash2} size={14} /> Delete
                </Button>
              </Show>
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
      <AlertDialog
        aria-label="Delete snapshot"
        open={deleteOpen()}
        onOpenChange={(open) => {
          if (!deleting()) setDeleteOpen(open);
        }}
      >
        <AlertDialogHeader>
          <AlertDialogTitle>Delete this snapshot?</AlertDialogTitle>
          <AlertDialogDescription>
            Timestow will remove this point in time from the encrypted
            repository. Stored data is reclaimed separately when the repository
            is pruned. This action cannot be undone.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <Show when={deleteError()}>
          {(message) => (
            <Text role="alert" class="text-sm text-danger-primary">
              {message()}
            </Text>
          )}
        </Show>
        <AlertDialogFooter>
          <AlertDialogCancel disabled={deleting()}>Cancel</AlertDialogCancel>
          <AlertDialogAction
            variant="destructive"
            loading={deleting()}
            loadingLabel="Deleting…"
            onClick={(event) => {
              event.preventDefault();
              void deleteSnapshot();
            }}
          >
            Delete snapshot
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialog>
    </>
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
