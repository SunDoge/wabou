import { Progress, Text, View } from "@wabou/ui";

export const BACKUP_PROGRESS_TOPIC = "timestow:backup-progress";

export interface BackupProgressEvent {
  profileId: string;
  state: "running" | "phaseComplete" | "completed" | "failed";
  kind: "spinner" | "counter" | "bytes";
  title: string;
  current: number;
  total?: number;
}

export function decodeBackupProgressEvent(value: unknown): BackupProgressEvent {
  if (!value || typeof value !== "object") {
    throw new TypeError("backup progress must be an object");
  }
  const event = value as Record<string, unknown>;
  if (
    typeof event.profileId !== "string" ||
    !["running", "phaseComplete", "completed", "failed"].includes(
      String(event.state),
    ) ||
    !["spinner", "counter", "bytes"].includes(String(event.kind)) ||
    typeof event.title !== "string" ||
    typeof event.current !== "number" ||
    (event.total !== null &&
      event.total !== undefined &&
      typeof event.total !== "number")
  ) {
    throw new TypeError("backup progress has invalid fields");
  }
  return {
    profileId: event.profileId,
    state: event.state as BackupProgressEvent["state"],
    kind: event.kind as BackupProgressEvent["kind"],
    title: event.title,
    current: event.current,
    total: typeof event.total === "number" ? event.total : undefined,
  };
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 ** 2) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 ** 3) return `${(bytes / 1024 ** 2).toFixed(1)} MB`;
  return `${(bytes / 1024 ** 3).toFixed(1)} GB`;
}

export function BackupProgressStatus(props: { progress: BackupProgressEvent }) {
  const total = () => props.progress.total;
  const indeterminate = () => total() === undefined || total() === 0;
  const valueLabel = () => {
    if (indeterminate()) return "Working…";
    const maximum = total() ?? 0;
    return props.progress.kind === "bytes"
      ? `${formatBytes(props.progress.current)} of ${formatBytes(maximum)}`
      : `${props.progress.current.toLocaleString()} of ${maximum.toLocaleString()}`;
  };
  return (
    <View class="min-w-0 flex flex-col gap-2" role="status">
      <View class="min-w-0 flex flex-row items-center justify-between gap-4">
        <Text class="min-w-0 flex-1 truncate text-sm text-secondary">
          {props.progress.title || "Backing up"}
        </Text>
        <Text class="flex-none text-xs font-mono text-muted">
          {valueLabel()}
        </Text>
      </View>
      <Progress
        label="Backup progress"
        size="sm"
        value={props.progress.current}
        maxValue={total() ?? 100}
        indeterminate={indeterminate()}
      />
    </View>
  );
}
