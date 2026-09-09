import {
  Alert,
  Badge,
  Button,
  ButtonGroup,
  Dialog,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DirectoryPicker,
  Icon,
  subscribeJsonHostMessages,
  Text,
  View,
} from "@wabou/ui";
import download from "lucide-static/icons/download.svg?raw";
import eye from "lucide-static/icons/eye.svg?raw";
import file from "lucide-static/icons/file.svg?raw";
import folder from "lucide-static/icons/folder.svg?raw";
import { createEffect, createSignal, onCleanup, Show } from "solid-js";
import {
  type FileEntry,
  type RestoreDestinationMode,
  type RestorePlanSummary,
  type RestorePreview,
  useRusticApi,
} from "./api";
import { createAsyncRequestGate } from "./async-request";
import {
  formatBytes,
  formatFileKind,
  formatOptionalDetailedTimestamp,
} from "./format";
import {
  createLocalOperationId,
  decodeOperationProgressEvent,
  OPERATION_PROGRESS_TOPIC,
  type OperationProgressEvent,
  OperationProgressStatus,
} from "./operation-progress";

export function FileDetails(props: {
  profileId: string;
  snapshotId: string;
  entry?: FileEntry;
  onOperationStart?: (profileId: string, operationId: string) => void;
  onOperationEnd?: (profileId: string, operationId: string) => void;
}) {
  const api = useRusticApi();
  const [previewing, setPreviewing] = createSignal(false);
  const [previewPath, setPreviewPath] = createSignal<string>();
  const [previewError, setPreviewError] = createSignal<string>();
  const previewRequests = createAsyncRequestGate();
  onCleanup(() => previewRequests.invalidate());

  createEffect(
    () =>
      `${props.profileId}\u0000${props.snapshotId}\u0000${props.entry?.path ?? ""}`,
    () => {
      previewRequests.invalidate();
      setPreviewing(false);
      setPreviewPath(undefined);
      setPreviewError(undefined);
    },
  );

  async function preview() {
    const entry = props.entry;
    if (!entry || previewing()) return;
    const request = previewRequests.begin();
    const profileId = props.profileId;
    const snapshotId = props.snapshotId;
    setPreviewing(true);
    setPreviewError(undefined);
    try {
      const result = await api.previewPath({
        profileId,
        snapshotId,
        path: entry.path,
      });
      if (!previewRequests.isCurrent(request)) return;
      setPreviewPath(result.destination);
      await api.openPath({ path: result.destination });
    } catch (cause) {
      if (previewRequests.isCurrent(request)) {
        setPreviewError(cause instanceof Error ? cause.message : String(cause));
      }
    } finally {
      if (previewRequests.isCurrent(request)) setPreviewing(false);
    }
  }

  return (
    <View
      role="region"
      aria-label="File details"
      class="w-full h-full min-h-0 flex flex-col gap-4 bg-surface-muted p-4"
    >
      <Show
        when={props.entry}
        fallback={
          <View class="flex flex-col gap-1 py-6">
            <Text class="font-medium">File details</Text>
            <Text class="text-sm text-muted">
              Select an item to inspect it or restore a copy.
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
                  {formatFileKind(entry().kind)}
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
            <Detail
              label="Modified"
              value={formatOptionalDetailedTimestamp(
                entry().modified,
                "Unknown",
              )}
            />
            <View class="flex flex-col gap-2 pt-1">
              <Button
                aria-label="Open preview"
                variant="outline"
                loading={previewing()}
                loadingLabel="Preparing preview…"
                onClick={() => void preview()}
              >
                <Icon source={eye} size={14} /> Open preview
              </Button>
              <RestoreDialog
                profileId={props.profileId}
                snapshotId={props.snapshotId}
                entry={entry()}
                onOperationStart={props.onOperationStart}
                onOperationEnd={props.onOperationEnd}
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

export function RestoreDialog(props: {
  profileId: string;
  snapshotId: string;
  entry: FileEntry;
  defaultOpen?: boolean;
  onOperationStart?: (profileId: string, operationId: string) => void;
  onOperationEnd?: (profileId: string, operationId: string) => void;
}) {
  const api = useRusticApi();
  const [destinationMode, setDestinationMode] =
    createSignal<RestoreDestinationMode>("original");
  const [destination, setDestination] = createSignal("");
  const [preview, setPreview] = createSignal<RestorePreview>();
  const [pending, setPending] = createSignal<"plan" | "restore">();
  const [error, setError] = createSignal<string>();
  const [result, setResult] = createSignal<string>();
  const [progress, setProgress] = createSignal<OperationProgressEvent>();
  let activeOperationId: string | undefined;
  const willOverwrite = () => {
    const plan = preview()?.plan;
    return Boolean(
      plan && (plan.filesToModify > 0 || plan.directoriesToModify > 0),
    );
  };

  const unsubscribeProgress = subscribeJsonHostMessages<OperationProgressEvent>(
    OPERATION_PROGRESS_TOPIC,
    (event) => {
      if (
        event.operation === "restore" &&
        event.operationId === activeOperationId
      ) {
        setProgress(event);
      }
    },
    {
      decode: decodeOperationProgressEvent,
      onError: (cause) =>
        console.error("[timestow] invalid operation progress", cause),
    },
  );
  onCleanup(unsubscribeProgress);

  function reset() {
    setDestinationMode("original");
    setDestination("");
    setPreview(undefined);
    setPending(undefined);
    setError(undefined);
    setResult(undefined);
    setProgress(undefined);
    activeOperationId = undefined;
  }

  async function review() {
    if ((destinationMode() === "custom" && !destination().trim()) || pending())
      return;
    setPending("plan");
    setError(undefined);
    try {
      setPreview(
        await api.previewRestore({
          profileId: props.profileId,
          snapshotId: props.snapshotId,
          path: props.entry.path,
          destinationMode: destinationMode(),
          destination: destination(),
        }),
      );
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setPending(undefined);
    }
  }

  async function restore() {
    const reviewed = preview();
    if (!reviewed || pending()) return;
    setPending("restore");
    setError(undefined);
    const operationId = createLocalOperationId("restore");
    activeOperationId = operationId;
    setProgress(undefined);
    try {
      props.onOperationStart?.(props.profileId, operationId);
      const restored = await api.restorePath({
        profileId: props.profileId,
        snapshotId: props.snapshotId,
        path: props.entry.path,
        destinationMode: destinationMode(),
        destination: destination(),
        allowOverwrite: willOverwrite(),
        operationId,
      });
      setResult(restored.destination);
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      activeOperationId = undefined;
      props.onOperationEnd?.(props.profileId, operationId);
      setPending(undefined);
    }
  }

  async function openExtractedItem(): Promise<void> {
    const path = result();
    if (!path) return;
    setError(undefined);
    try {
      await api.openPath({ path });
    } catch (cause) {
      setError(cause instanceof Error ? cause.message : String(cause));
    }
  }

  return (
    <Dialog
      aria-label={`Restore ${props.entry.name}`}
      defaultOpen={props.defaultOpen}
      closeOnBackdrop={!pending()}
      closeOnEscape={!pending()}
      onOpenChange={(open) => {
        if (open) reset();
      }}
      trigger={(trigger) => (
        <Button {...trigger} aria-label="Restore…">
          <Icon source={download} size={14} /> Restore…
        </Button>
      )}
    >
      {(dialog) => (
        <View class="flex flex-col gap-4">
          <DialogHeader>
            <DialogTitle>Restore {props.entry.name}</DialogTitle>
            <DialogDescription>
              Restore to the recorded source path, or choose another folder. The
              snapshot itself is never changed.
            </DialogDescription>
          </DialogHeader>
          <View class="flex flex-col gap-2">
            <Text class="text-sm font-medium">Restore destination</Text>
            <ButtonGroup
              size="sm"
              variant="outline"
              aria-label="Restore destination mode"
              class="self-start"
            >
              <Button
                size="sm"
                variant="outline"
                selected={destinationMode() === "original"}
                aria-pressed={destinationMode() === "original"}
                aria-label="Original location"
                disabled={Boolean(pending()) || Boolean(result())}
                onClick={() => {
                  setDestinationMode("original");
                  setPreview(undefined);
                }}
              >
                Original location
              </Button>
              <Button
                size="sm"
                variant="outline"
                selected={destinationMode() === "custom"}
                aria-pressed={destinationMode() === "custom"}
                aria-label="Choose folder"
                disabled={Boolean(pending()) || Boolean(result())}
                onClick={() => {
                  setDestinationMode("custom");
                  setPreview(undefined);
                }}
              >
                Choose folder
              </Button>
            </ButtonGroup>
            <Show
              when={destinationMode() === "custom"}
              fallback={
                <Text class="text-xs text-muted">
                  The recorded source path is used by default. Existing content
                  is never replaced without a reviewed restore plan.
                </Text>
              }
            >
              <DirectoryPicker
                aria-label="Restore destination"
                placeholder="Choose a destination folder"
                value={destination()}
                disabled={Boolean(pending()) || Boolean(result())}
                onValueChange={(value) => {
                  setDestination(value);
                  setPreview(undefined);
                }}
                onBrowseError={(cause) =>
                  setError(
                    cause instanceof Error ? cause.message : String(cause),
                  )
                }
              />
            </Show>
          </View>
          <Show when={preview()}>
            {(current) => (
              <View class="flex flex-col gap-3">
                <Detail label="Will restore to" value={current().destination} />
                <RestorePlanReview plan={current().plan} />
              </View>
            )}
          </Show>
          <Show when={pending() === "restore" && progress()}>
            <OperationProgressStatus progress={progress()!} />
          </Show>
          <Show when={result()}>
            {(path) => (
              <Alert title="Restore complete">
                <Text class="whitespace-normal text-sm">{path()}</Text>
              </Alert>
            )}
          </Show>
          <Show when={error()}>
            {(message) => (
              <Alert variant="destructive" title="Restore failed">
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
              when={!result()}
              fallback={
                <Button onClick={() => void openExtractedItem()}>
                  Open restored item
                </Button>
              }
            >
              <Show
                when={preview()}
                fallback={
                  <Button
                    disabled={
                      (destinationMode() === "custom" &&
                        !destination().trim()) ||
                      Boolean(pending())
                    }
                    loading={pending() === "plan"}
                    loadingLabel="Reviewing…"
                    aria-label="Review restore"
                    onClick={() => void review()}
                  >
                    Review restore
                  </Button>
                }
              >
                <Button
                  disabled={Boolean(pending())}
                  loading={pending() === "restore"}
                  loadingLabel="Restoring…"
                  onClick={() => void restore()}
                >
                  {willOverwrite() ? "Replace and restore" : "Restore"}
                </Button>
              </Show>
            </Show>
          </DialogFooter>
        </View>
      )}
    </Dialog>
  );
}

export function RestorePlanReview(props: { plan: RestorePlanSummary }) {
  const changed = () =>
    props.plan.filesToModify > 0 || props.plan.directoriesToModify > 0;
  return (
    <View
      role="region"
      aria-label="Restore plan"
      class="min-w-0 flex flex-col gap-3"
    >
      <View class="grid grid-cols-2 gap-3 rounded-lg border border-subtle bg-surface-muted p-3">
        <PlanMetric label="Data" value={formatBytes(props.plan.restoreSize)} />
        <PlanMetric
          label="New files"
          value={String(props.plan.filesToRestore)}
        />
        <PlanMetric
          label="Files replaced"
          value={String(props.plan.filesToModify)}
          changed={props.plan.filesToModify > 0}
        />
        <PlanMetric
          label="New folders"
          value={String(props.plan.directoriesToRestore)}
        />
        <PlanMetric
          label="Folders updated"
          value={String(props.plan.directoriesToModify)}
          changed={props.plan.directoriesToModify > 0}
        />
        <PlanMetric
          label="Unchanged"
          value={String(props.plan.filesUnchanged)}
        />
      </View>
      <Show when={changed()}>
        <Alert variant="warning" title="Existing content will change">
          Review the destination carefully. Existing files or folders in this
          restore plan will be replaced.
        </Alert>
      </Show>
    </View>
  );
}

function PlanMetric(props: {
  label: string;
  value: string;
  changed?: boolean;
}) {
  return (
    <View class="flex flex-col gap-0.5">
      <Text class="text-xs text-muted">{props.label}</Text>
      <Text
        class={
          props.changed ? "font-semibold text-danger-primary" : "font-semibold"
        }
      >
        {props.value}
      </Text>
    </View>
  );
}
