// Pure view-model helpers shared with UI tests.
export interface Rect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface Point {
  x: number;
  y: number;
}

export interface Size {
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

export interface OverlayPaintEvidence {
  sequence: number;
  enabled: boolean;
  layout_bounds: number;
  clip_bounds: number;
  highlights: number;
}

export interface ValidationSummary {
  revision: number;
  valid: boolean;
  errorCount: number;
  warningCount: number;
  truncated: boolean;
}

export interface LatestRequestGate {
  begin(): number;
  isCurrent(token: number): boolean;
}

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

/** Ignore responses from requests superseded by a newer user intent. */
export function createLatestRequestGate(): LatestRequestGate {
  let current = 0;
  return {
    begin: () => ++current,
    isCurrent: (token) => token === current,
  };
}

export function validationStatusLabel(
  report: ValidationSummary | undefined,
  currentRevision: number | undefined,
): string {
  if (!report) return "snapshot not validated";
  const stale =
    currentRevision !== undefined && report.revision !== currentRevision;
  const summary = report.valid
    ? report.warningCount === 0
      ? "valid"
      : `valid · ${report.warningCount} warning${report.warningCount === 1 ? "" : "s"}`
    : `${report.errorCount} error${report.errorCount === 1 ? "" : "s"} · ${report.warningCount} warning${report.warningCount === 1 ? "" : "s"}`;
  return `${stale ? "stale · " : ""}r${report.revision} · ${summary}${report.truncated ? " · truncated" : ""}`;
}

export function overlayEvidenceLabel(
  layers: (OverlayLayers & { selectedNode?: unknown }) | undefined,
  paint: OverlayPaintEvidence | undefined,
): string {
  if (!layers || !paint) return "overlay evidence unavailable";
  const requested =
    layers.layout ||
    layers.clips ||
    layers.hitTarget ||
    layers.selectedNode != null;
  if (!requested) return `overlay off · pass ${paint.sequence}`;
  if (!paint.enabled) return "overlay requested · awaiting native paint";
  return `overlay pass ${paint.sequence} · ${paint.layout_bounds} bounds · ${paint.clip_bounds} clips · ${paint.highlights} highlights`;
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

/** Map a click in a stretched screenshot back to inspected logical pixels. */
export function screenshotPoint(
  offset: Point,
  renderedSize: { width: number; height: number },
  viewportSize: { width: number; height: number },
): Point | undefined {
  if (
    renderedSize.width <= 0 ||
    renderedSize.height <= 0 ||
    viewportSize.width <= 0 ||
    viewportSize.height <= 0 ||
    !Number.isFinite(offset.x) ||
    !Number.isFinite(offset.y)
  )
    return undefined;
  const x = Math.max(0, Math.min(renderedSize.width, offset.x));
  const y = Math.max(0, Math.min(renderedSize.height, offset.y));
  return {
    x: (x / renderedSize.width) * viewportSize.width,
    y: (y / renderedSize.height) * viewportSize.height,
  };
}

/** Fit captured pixels inside the available stage without distorting them. */
export function containSize(content: Size, bounds: Size): Size | undefined {
  if (
    content.width <= 0 ||
    content.height <= 0 ||
    bounds.width <= 0 ||
    bounds.height <= 0
  )
    return undefined;
  const scale = Math.min(
    bounds.width / content.width,
    bounds.height / content.height,
  );
  return { width: content.width * scale, height: content.height * scale };
}
