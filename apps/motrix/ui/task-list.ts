import type { DownloadTask } from "./downloads";

export type TaskFilter = "all" | "active" | "waiting" | "complete" | "stopped";
export type TaskSort = "queue" | "name" | "size" | "progress" | "speed";

export type RestartTaskAction = "retry" | "reseed";
export type PrimaryTaskAction = "pause" | "resume" | "stopSeeding";

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
  if (sort === "queue") return projected;
  return projected.sort((left, right) => {
    const order =
      sort === "name"
        ? left.name.localeCompare(right.name)
        : sort === "size"
          ? right.totalLength - left.totalLength
          : sort === "progress"
            ? progress(right) - progress(left)
            : right.downloadSpeed - left.downloadSpeed;
    return order || left.id.localeCompare(right.id);
  });
}
