// Pure view-model helpers shared with UI tests.
export interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface RpcEnvelope<T> {
  ok: boolean;
  value?: T;
  error?: string;
}

export function decode<T>(raw: string): T {
  const envelope = JSON.parse(raw) as RpcEnvelope<T>;
  if (!envelope.ok)
    throw new Error(envelope.error ?? "DevTools request failed");
  return envelope.value as T;
}

export function overlayStyle(
  rect: Rect,
  viewportWidth: number,
  viewportHeight: number,
): Record<string, string> | undefined {
  if (viewportWidth <= 0 || viewportHeight <= 0) return undefined;
  return {
    left: `${(rect.x / viewportWidth) * 100}%`,
    top: `${(rect.y / viewportHeight) * 100}%`,
    width: `${(rect.width / viewportWidth) * 100}%`,
    height: `${(rect.height / viewportHeight) * 100}%`,
  };
}
