import type { Aria2Task } from "./aria2";

export type TaskFilter = "all" | "active" | "waiting" | "complete" | "stopped";
export type TaskSort = "queue" | "name" | "size" | "progress" | "speed";

export function taskMatchesFilter(
  task: Aria2Task,
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

function progress(task: Aria2Task): number {
  return task.totalLength > 0 ? task.completedLength / task.totalLength : 0;
}

export function projectTasks(
  tasks: readonly Aria2Task[],
  filter: TaskFilter,
  query: string,
  sort: TaskSort,
): Aria2Task[] {
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
    return order || left.gid.localeCompare(right.gid);
  });
}
