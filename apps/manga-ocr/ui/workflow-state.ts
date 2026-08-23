import type { ModelDownloadProgress } from "./api";

export function downloadProgressPercent(
  progress: ModelDownloadProgress,
): number {
  if (progress.totalBytes <= 0) return 0;
  return Math.round(
    Math.min(1, Math.max(0, progress.downloadedBytes / progress.totalBytes)) *
      100,
  );
}
