import { match } from "ts-pattern";
import type { DownloadTask } from "./downloads";

export type TaskFilter = "all" | "active" | "waiting" | "complete" | "stopped";
export type TaskSort =
  | "newest"
  | "oldest"
  | "priority"
  | "name"
  | "size"
  | "progress"
  | "speed";

export type RestartTaskAction = "retry" | "reseed";
export type PrimaryTaskAction = "pause" | "resume" | "stopSeeding";
export type TaskStatusVariant =
  | "default"
  | "secondary"
  | "outline"
  | "success"
  | "destructive";

export interface TaskStatusPresentation {
  label: string;
  variant: TaskStatusVariant;
}

function humanizeStatus(status: string): string {
  const words = status.trim().replaceAll(/[-_]+/g, " ");
  return words ? words.charAt(0).toUpperCase() + words.slice(1) : "Unknown";
}

export function taskStatusPresentation(status: string): TaskStatusPresentation {
  return match(status)
    .with("active", () => ({
      label: "Downloading",
      variant: "default" as const,
    }))
    .with("waiting", () => ({
      label: "Waiting",
      variant: "secondary" as const,
    }))
    .with("paused", () => ({ label: "Paused", variant: "secondary" as const }))
    .with("complete", () => ({
      label: "Completed",
      variant: "success" as const,
    }))
    .with("seeding", () => ({ label: "Seeding", variant: "success" as const }))
    .with("error", () => ({ label: "Failed", variant: "destructive" as const }))
    .with("removed", () => ({ label: "Stopped", variant: "outline" as const }))
    .otherwise((value) => ({
      label: humanizeStatus(value),
      variant: "outline" as const,
    }));
}

export function taskPathActions(task: DownloadTask): {
  openFile: boolean;
  showInFolder: boolean;
} {
  return {
    openFile: task.status === "complete" && Boolean(task.filePath),
    showInFolder: Boolean(task.filePath || task.dir),
  };
}

export function primaryTaskAction(
  task: DownloadTask,
): PrimaryTaskAction | undefined {
  if (task.status === "active" || task.status === "waiting") return "pause";
  if (task.status === "paused") return "resume";
  if (task.status === "seeding") return "stopSeeding";
  return undefined;
}

export function primaryTaskActionLabel(action: PrimaryTaskAction): string {
  if (action === "stopSeeding") return "Stop seeding";
  return action === "pause" ? "Pause" : "Resume";
}

export function restartTaskAction(
  task: DownloadTask,
): RestartTaskAction | undefined {
  if (!task.retryable) return undefined;
  if (task.status === "error" || task.status === "removed") return "retry";
  if (task.status === "complete" && task.bittorrent) return "reseed";
  return undefined;
}

export function taskMatchesFilter(
  task: DownloadTask,
  filter: TaskFilter,
): boolean {
  if (filter === "all") return true;
  if (filter === "waiting")
    return task.status === "waiting" || task.status === "paused";
  if (filter === "stopped")
    return task.status === "error" || task.status === "removed";
  if (filter === "active")
    return task.status === "active" || task.status === "seeding";
  return task.status === filter;
}

function progress(task: DownloadTask): number {
  return task.totalLength > 0 ? task.completedLength / task.totalLength : 0;
}

export function projectTasks(
  tasks: readonly DownloadTask[],
  filter: TaskFilter,
  query: string,
  sort: TaskSort,
): DownloadTask[] {
  const needle = query.trim().toLocaleLowerCase();
  const projected = tasks.filter(
    (task) =>
      taskMatchesFilter(task, filter) &&
      (!needle || task.name.toLocaleLowerCase().includes(needle)),
  );
  return projected.sort((left, right) => {
    const order = match(sort)
      .with("newest", () => right.createdAtMs - left.createdAtMs)
      .with("oldest", () => left.createdAtMs - right.createdAtMs)
      .with(
        "priority",
        () =>
          taskPriorityRank(right.priority) - taskPriorityRank(left.priority) ||
          left.createdAtMs - right.createdAtMs,
      )
      .with("name", () => left.name.localeCompare(right.name))
      .with("size", () => right.totalLength - left.totalLength)
      .with("progress", () => progress(right) - progress(left))
      .with("speed", () => right.downloadSpeed - left.downloadSpeed)
      .exhaustive();
    return order || left.id.localeCompare(right.id);
  });
}

function taskPriorityRank(priority: DownloadTask["priority"]): number {
  return match(priority)
    .with("critical", () => 3)
    .with("high", () => 2)
    .with("normal", () => 1)
    .with("low", () => 0)
    .exhaustive();
}
