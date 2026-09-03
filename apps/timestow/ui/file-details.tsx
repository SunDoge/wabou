import {
  Alert,
  Badge,
  Button,
  Dialog,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DirectoryPicker,
  Icon,
  Text,
  View,
} from "@wabou/ui";
import download from "lucide-static/icons/download.svg?raw";
import eye from "lucide-static/icons/eye.svg?raw";
import file from "lucide-static/icons/file.svg?raw";
import folder from "lucide-static/icons/folder.svg?raw";
import { createSignal, Show } from "solid-js";
import { type FileEntry, type RestorePlanSummary, useRusticApi } from "./api";

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 ** 2) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 ** 3) return `${(bytes / 1024 ** 2).toFixed(1)} MB`;
  return `${(bytes / 1024 ** 3).toFixed(1)} GB`;
}

export function FileDetails(props: {
  profileId: string;
  snapshotId: string;
  entry?: FileEntry;
}) {
  const api = useRusticApi();
  const [previewing, setPreviewing] = createSignal(false);
  const [previewPath, setPreviewPath] = createSignal<string>();
  const [previewError, setPreviewError] = createSignal<string>();

  async function preview() {
    const entry = props.entry;
    if (!entry || previewing()) return;
    setPreviewing(true);
    setPreviewError(undefined);
    try {
      const result = await api.previewPath({
        profileId: props.profileId,
        snapshotId: props.snapshotId,
        path: entry.path,
      });
      setPreviewPath(result.destination);
      await api.openPath({ path: result.destination });
    } catch (cause) {
      setPreviewError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setPreviewing(false);
    }
  }

  return (
    <View class="w-72 min-h-0 flex-none flex flex-col gap-4 border-l border-subtle bg-surface-muted p-4">
      <Show
        when={props.entry}
        fallback={
          <View class="flex flex-col gap-1 py-6">
            <Text class="font-medium">File details</Text>
            <Text class="text-sm text-muted">
              Select an item to inspect it or extract a copy.
            </Text>
          </View>
        }
      >
        {(entry) => (
          <>
            <View class="flex flex-row items-center gap-3">
              <View class="size-10 flex-none flex items-center justify-center rounded-lg bg-selected text-accent">
                <Icon
                  source={entry().kind === "directory" ? folder : file}
                  size={18}
                />
              </View>
              <View class="min-w-0 flex-1 flex flex-col gap-0.5">
                <Text class="truncate font-semibold">{entry().name}</Text>
                <Badge variant="secondary" class="self-start">
                  {entry().kind}
                </Badge>
              </View>
            </View>
            <Detail label="Path" value={entry().path} />
            <Detail
              label="Size"
              value={
                entry().kind === "directory" ? "—" : formatBytes(entry().size)
              }
            />
            <Detail label="Modified" value={entry().modified ?? "Unknown"} />
            <View class="flex flex-col gap-2 pt-1">
              <Button
                variant="outline"
                loading={previewing()}
                loadingLabel="Preparing preview…"
                onClick={() => void preview()}
              >
                <Icon source={eye} size={14} /> Preview temporary copy
              </Button>
              <ExtractDialog
                profileId={props.profileId}
                snapshotId={props.snapshotId}
                entry={entry()}
              />
            </View>
            <Show when={previewPath()}>
              {(path) => (
                <Alert title="Preview ready" class="p-3">
                  <Text class="whitespace-normal text-xs text-muted">
                    {path()}
                  </Text>
                </Alert>
              )}
            </Show>
            <Show when={previewError()}>
              {(message) => (
                <Alert variant="destructive" title="Preview failed" class="p-3">
                  {message()}
                </Alert>
              )}
            </Show>
          </>
        )}
      </Show>
    </View>
  );
}

function Detail(props: { label: string; value: string }) {
  return (
    <View class="flex flex-col gap-1">
      <Text class="text-xs font-medium tracking-wide text-muted">
        {props.label}
      </Text>
      <Text class="whitespace-normal text-sm">{props.value}</Text>
    </View>
  );
}

function ExtractDialog(props: {
  profileId: string;
  snapshotId: string;
  entry: FileEntry;
}) {
  const api = useRusticApi();
  const [destination, setDestination] = createSignal("");
  const [plan, setPlan] = createSignal<RestorePlanSummary>();
  const [pending, setPending] = createSignal<"plan" | "extract">();
  const [error, setError] = createSignal<string>();
  const [result, setResult] = createSignal<string>();

  function reset() {
    setDestination("");
    setPlan(undefined);
    setPending(undefined);
    setError(undefined);
    setResult(undefined);
  }

  async function review() {
    if (!destination().trim() || pending()) return;
    setPending("plan");
    setError(undefined);
    try {
      setPlan(
        await api.previewRestore({
          profileId: props.profileId,
          snapshotId: props.snapshotId,
          path: props.entry.path,
          destination: destination(),
        }),
      );
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setPending(undefined);
    }
  }

  async function extract() {
    if (!plan() || pending()) return;
    setPending("extract");
    setError(undefined);
    try {
      const restored = await api.restorePath({
        profileId: props.profileId,
        snapshotId: props.snapshotId,
        path: props.entry.path,
        destination: destination(),
      });
      setResult(restored.destination);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setPending(undefined);
    }
  }

  return (
    <Dialog
      aria-label={`Extract ${props.entry.name}`}
      closeOnBackdrop={!pending()}
      closeOnEscape={!pending()}
      onOpenChange={(open) => {
        if (open) reset();
      }}
      trigger={(trigger) => (
        <Button {...trigger}>
          <Icon source={download} size={14} /> Extract…
        </Button>
      )}
    >
      {(dialog) => (
        <View class="flex flex-col gap-4">
          <DialogHeader>
            <DialogTitle>Extract {props.entry.name}</DialogTitle>
            <DialogDescription>
              Choose a destination, review the restore plan, then extract a copy
              without changing the snapshot.
            </DialogDescription>
          </DialogHeader>
          <DirectoryPicker
            aria-label="Extraction destination"
            placeholder="Choose a destination folder"
            value={destination()}
            disabled={Boolean(pending()) || Boolean(result())}
            onValueChange={(value) => {
              setDestination(value);
              setPlan(undefined);
            }}
            onBrowseError={(cause) =>
              setError(cause instanceof Error ? cause.message : String(cause))
            }
          />
          <Show when={plan()}>
            {(current) => (
              <View class="grid grid-cols-2 gap-3 rounded-lg border border-subtle bg-surface-muted p-3">
                <PlanMetric
                  label="Data"
                  value={formatBytes(current().restoreSize)}
                />
                <PlanMetric
                  label="Files"
                  value={String(
                    current().filesToRestore + current().filesToModify,
                  )}
                />
                <PlanMetric
                  label="Folders"
                  value={String(
                    current().directoriesToRestore +
                      current().directoriesToModify,
                  )}
                />
                <PlanMetric
                  label="Unchanged"
                  value={String(current().filesUnchanged)}
                />
              </View>
            )}
          </Show>
          <Show when={result()}>
            {(path) => (
              <Alert title="Extraction complete">
                <Text class="whitespace-normal text-sm">{path()}</Text>
              </Alert>
            )}
          </Show>
          <Show when={error()}>
            {(message) => (
              <Alert variant="destructive" title="Extraction failed">
                {message()}
              </Alert>
            )}
          </Show>
          <DialogFooter>
            <Button
              variant="outline"
              disabled={Boolean(pending())}
              onClick={dialog.close}
            >
              {result() ? "Done" : "Cancel"}
            </Button>
            <Show
              when={plan()}
              fallback={
                <Button
                  disabled={!destination().trim() || Boolean(pending())}
                  loading={pending() === "plan"}
                  loadingLabel="Reviewing…"
                  onClick={() => void review()}
                >
                  Review extraction
                </Button>
              }
            >
              <Button
                disabled={Boolean(pending()) || Boolean(result())}
                loading={pending() === "extract"}
                loadingLabel="Extracting…"
                onClick={() => void extract()}
              >
                Extract
              </Button>
            </Show>
          </DialogFooter>
        </View>
      )}
    </Dialog>
  );
}

function PlanMetric(props: { label: string; value: string }) {
  return (
    <View class="flex flex-col gap-0.5">
      <Text class="text-xs text-muted">{props.label}</Text>
      <Text class="font-semibold">{props.value}</Text>
    </View>
  );
}
