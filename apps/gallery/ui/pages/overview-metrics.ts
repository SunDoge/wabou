import type { FrameStats } from "@wabou/ui";

export interface FrameStage {
  label: "JS" | "Host" | "Scene" | "Present";
  value: number;
  width: number;
}

/** Total time represented by the native frame diagnostics. */
export function frameDuration(stats: FrameStats): number {
  // build_frame_ms spans the complete FrameSource::build_frame call, including
  // the separately sampled JavaScript tick. Do not count that nested sample
  // twice when presenting the end-to-end frame duration.
  return stats.build_frame_ms + stats.scene_ms + stats.present_ms;
}

/**
 * Convert native timings into bounded chart stages. Widths share one scale so
 * their relative cost remains visible while a fast frame still has headroom.
 */
export function frameStages(stats: FrameStats): FrameStage[] {
  const hostBuild = Math.max(0, stats.build_frame_ms - stats.js_tick_ms);
  const values = [
    ["JS", stats.js_tick_ms],
    ["Host", hostBuild],
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
