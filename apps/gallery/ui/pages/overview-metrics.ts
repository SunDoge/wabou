import type { FrameStats } from "@wabou/core";

export interface FrameStage {
  label: "JS" | "Build" | "Scene" | "Present";
  value: number;
  width: number;
}

/** Total time represented by the native frame diagnostics. */
export function frameDuration(stats: FrameStats): number {
  return (
    stats.js_tick_ms + stats.build_frame_ms + stats.scene_ms + stats.present_ms
  );
}

/**
 * Convert native timings into bounded chart stages. Widths share one scale so
 * their relative cost remains visible while a fast frame still has headroom.
 */
export function frameStages(stats: FrameStats): FrameStage[] {
  const values = [
    ["JS", stats.js_tick_ms],
    ["Build", stats.build_frame_ms],
    ["Scene", stats.scene_ms],
    ["Present", stats.present_ms],
  ] as const;
  const scale = Math.max(16.67, frameDuration(stats));
  return values.map(([label, value]) => ({
    label,
    value,
    width: Math.min(100, Math.max(0, (value / scale) * 100)),
  }));
}

/** Keep a bounded immutable history suitable for a retained chart source. */
export function appendFrameSample(
  samples: readonly number[],
  value: number,
  capacity = 36,
): number[] {
  if (!Number.isFinite(value) || value < 0) return [...samples];
  const next = [...samples, value];
  return next.slice(Math.max(0, next.length - capacity));
}
