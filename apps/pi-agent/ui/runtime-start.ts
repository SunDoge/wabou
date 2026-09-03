export function createSingleFlight<K, V>() {
  const pending = new Map<K, Promise<V>>();
  return (key: K, task: () => Promise<V>): Promise<V> => {
    const current = pending.get(key);
    if (current) return current;
    const next = task();
    pending.set(key, next);
    const clear = () => {
      if (pending.get(key) === next) pending.delete(key);
    };
    void next.then(clear, clear);
    return next;
  };
}

/**
 * Stable launch identity for the active project route.
 *
 * A project route without an explicit session launches Pi in continue mode.
 * An explicit historical session waits until the session catalog proves that
 * it belongs to the project, avoiding accidental launches from stale routes.
 */
export function activeRuntimeRouteKey(
  agentId: string | undefined,
  sessionId: string | undefined,
  agentIds: readonly string[],
  sessions: readonly { agentId: string; sessionId: string }[],
): string {
  if (!agentId || !agentIds.includes(agentId)) return "";
  if (
    sessionId &&
    !sessions.some(
      (candidate) =>
        candidate.agentId === agentId && candidate.sessionId === sessionId,
    )
  ) {
    return "";
  }
  return `${agentId}\0${sessionId ?? ""}`;
}
