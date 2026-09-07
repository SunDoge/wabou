import {
  Alert,
  Button,
  Dialog,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Icon,
  ScrollArea,
  Text,
  View,
} from "@wabou/ui";
import shieldCheck from "lucide-static/icons/shield-check.svg?raw";
import { createSignal, For as ForValue, Show } from "solid-js";
import {
  type RepositoryCheckResult,
  type RepositoryStats,
  useRusticApi,
} from "./api";
import { formatBytes } from "./format";

export interface RepositoryCheckDialogProps {
  profileId: string;
  disabled?: boolean;
  defaultOpen?: boolean;
}

function checkSummary(result: RepositoryCheckResult): {
  variant: "success" | "warning" | "error";
  title: string;
} {
  if (!result.healthy) {
    return {
      variant: "error" as const,
      title: "Repository problems found",
    };
  }
  if (result.findings.length > 0) {
    return {
      variant: "warning" as const,
      title: "Repository is usable with warnings",
    };
  }
  return {
    variant: "success" as const,
    title: "Repository structure is healthy",
  };
}

function countLabel(count: number, noun: string): string {
  return `${count} ${noun}${count === 1 ? "" : "s"}`;
}

function RepositoryMetric(props: { label: string; value: string }) {
  return (
    <View class="min-w-0 flex flex-col gap-1">
      <Text class="text-xs font-medium text-muted">{props.label}</Text>
      <Text class="truncate text-base font-semibold">{props.value}</Text>
    </View>
  );
}

function RepositoryStatistics(props: { stats: RepositoryStats }) {
  return (
    <View
      role="group"
      aria-label="Repository statistics"
      class="grid grid-cols-2 gap-x-5 gap-y-4 rounded-lg border border-subtle bg-surface-muted p-4"
    >
      <RepositoryMetric
        label="Repository size"
        value={formatBytes(props.stats.repositorySize)}
      />
      <RepositoryMetric
        label="Unique data"
        value={formatBytes(props.stats.uniqueDataSize)}
      />
      <RepositoryMetric
        label="Snapshots"
        value={props.stats.snapshotCount.toLocaleString()}
      />
      <RepositoryMetric
        label="Active packs"
        value={props.stats.packCount.toLocaleString()}
      />
    </View>
  );
}

export function RepositoryCheckResultView(props: {
  result: RepositoryCheckResult;
}) {
  const summary = () => checkSummary(props.result);
  return (
    <View class="min-w-0 flex flex-col gap-3">
      <Alert variant={summary().variant} title={summary().title}>
        {countLabel(props.result.findings.length, "finding")}
      </Alert>
      <Show when={props.result.stats}>
        {(stats) => <RepositoryStatistics stats={stats()} />}
      </Show>
      <Show when={props.result.findings.length > 0}>
        <ScrollArea
          aria-label="Repository check findings"
          class="max-h-48 rounded-lg border border-subtle bg-surface-muted"
          contentClass="p-3 flex flex-col gap-2"
        >
          <ForValue each={props.result.findings}>
            {(finding) => (
              <Text class="whitespace-normal text-xs text-secondary">
                {finding}
              </Text>
            )}
          </ForValue>
        </ScrollArea>
      </Show>
    </View>
  );
}

export function RepositoryCheckDialog(props: RepositoryCheckDialogProps) {
  const api = useRusticApi();
  const [checking, setChecking] = createSignal(false);
  const [result, setResult] = createSignal<RepositoryCheckResult>();
  const [error, setError] = createSignal<string>();

  async function check(): Promise<void> {
    if (checking()) return;
    setChecking(true);
    setError(undefined);
    try {
      setResult(await api.checkRepository({ profileId: props.profileId }));
    } catch (cause) {
      setResult(undefined);
      setError(cause instanceof Error ? cause.message : String(cause));
    } finally {
      setChecking(false);
    }
  }

  return (
    <Dialog
      aria-label="Check repository"
      defaultOpen={props.defaultOpen}
      trigger={(trigger) => (
        <Button
          {...trigger}
          aria-label="Check repository"
          variant="outline"
          disabled={props.disabled}
          onClick={(event) => {
            trigger.onClick(event);
            void check();
          }}
        >
          <Icon source={shieldCheck} size={14} /> Verify
        </Button>
      )}
    >
      {(dialog) => (
        <View class="min-w-0 flex flex-col gap-4">
          <DialogHeader>
            <DialogTitle>Check repository</DialogTitle>
            <DialogDescription>
              Verify indexes, snapshots, file trees, and pack references. This
              quick check does not read every stored data byte.
            </DialogDescription>
          </DialogHeader>

          <Show when={result()}>
            {(current) => <RepositoryCheckResultView result={current()} />}
          </Show>
          <Show when={error()}>
            {(message) => (
              <Alert variant="error" title="Repository check failed">
                {message()}
              </Alert>
            )}
          </Show>

          <DialogFooter>
            <Button variant="outline" onClick={dialog.close}>
              Close
            </Button>
            <Button
              aria-label={result() ? "Check again" : "Check now"}
              loading={checking()}
              loadingLabel="Checking…"
              onClick={() => void check()}
            >
              {result() ? "Check again" : "Check now"}
            </Button>
          </DialogFooter>
        </View>
      )}
    </Dialog>
  );
}
