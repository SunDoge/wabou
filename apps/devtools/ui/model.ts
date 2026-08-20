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

export interface OverlayLayers {
  layout: boolean;
  clips: boolean;
  hitTarget: boolean;
}

export type OverlayLayer = keyof OverlayLayers;

export const EMPTY_OVERLAY_LAYERS: OverlayLayers = Object.freeze({
  layout: false,
  clips: false,
  hitTarget: false,
});

export function toggleOverlayLayer(
  layers: OverlayLayers,
  layer: OverlayLayer,
): OverlayLayers {
  return { ...layers, [layer]: !layers[layer] };
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
