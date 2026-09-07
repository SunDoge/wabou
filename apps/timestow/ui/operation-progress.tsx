import { Progress, Text, View } from "@wabou/ui";
import { formatBytes } from "./format";

export const OPERATION_PROGRESS_TOPIC = "timestow:operation-progress";

export interface OperationProgressEvent {
  profileId: string;
  operation: "backup" | "restore";
  operationId: string;
  state: "running" | "phaseComplete" | "completed" | "failed";
  unit: "spinner" | "counter" | "bytes";
  title: string;
  current: number;
  total?: number;
}

export function decodeOperationProgressEvent(
  value: unknown,
): OperationProgressEvent {
  if (!value || typeof value !== "object") {
    throw new TypeError("operation progress must be an object");
  }
  const event = value as Record<string, unknown>;
  if (
    typeof event.profileId !== "string" ||
    !["backup", "restore"].includes(String(event.operation)) ||
    typeof event.operationId !== "string" ||
    !["running", "phaseComplete", "completed", "failed"].includes(
      String(event.state),
    ) ||
    !["spinner", "counter", "bytes"].includes(String(event.unit)) ||
    typeof event.title !== "string" ||
    typeof event.current !== "number" ||
    (event.total !== null &&
      event.total !== undefined &&
      typeof event.total !== "number")
  ) {
    throw new TypeError("operation progress has invalid fields");
  }
  return {
    profileId: event.profileId,
    operation: event.operation as OperationProgressEvent["operation"],
    operationId: event.operationId,
    state: event.state as OperationProgressEvent["state"],
    unit: event.unit as OperationProgressEvent["unit"],
    title: event.title,
    current: event.current,
    total: typeof event.total === "number" ? event.total : undefined,
  };
}

export function OperationProgressStatus(props: {
  progress: OperationProgressEvent;
}) {
  const total = () => props.progress.total;
  const indeterminate = () => total() === undefined || total() === 0;
  const valueLabel = () => {
    if (indeterminate()) return "Working…";
    const maximum = total() ?? 0;
    return props.progress.unit === "bytes"
      ? `${formatBytes(props.progress.current)} of ${formatBytes(maximum)}`
      : `${props.progress.current.toLocaleString()} of ${maximum.toLocaleString()}`;
  };
  return (
    <View class="min-w-0 flex flex-col gap-2" role="status">
      <View class="min-w-0 flex flex-row items-center justify-between gap-4">
        <Text class="min-w-0 flex-1 truncate text-sm text-secondary">
          {props.progress.title ||
            (props.progress.operation === "backup"
              ? "Backing up"
              : "Extracting")}
        </Text>
        <Text class="flex-none text-xs font-mono text-muted">
          {valueLabel()}
        </Text>
      </View>
      <Progress
        label={
          props.progress.operation === "backup"
            ? "Backup progress"
            : "Extraction progress"
        }
        size="sm"
        value={props.progress.current}
        maxValue={total() ?? 100}
        indeterminate={indeterminate()}
      />
    </View>
  );
}
