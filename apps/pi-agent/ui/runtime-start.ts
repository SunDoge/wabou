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

export function sessionRouteKey(
  agentId: string | undefined,
  sessionId: string | undefined,
  agentIds: readonly string[],
  sessions: readonly { agentId: string; sessionId: string }[],
): string {
  if (!agentId || !sessionId || !agentIds.includes(agentId)) return "";
  return sessions.some(
    (candidate) =>
      candidate.agentId === agentId && candidate.sessionId === sessionId,
  )
    ? `${agentId}\0${sessionId}`
    : "";
}
