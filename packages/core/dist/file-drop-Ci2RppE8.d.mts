//#region src/glue/file-drop.d.ts
type FileDropPhase = "entered" | "moved" | "left" | "dropped";
interface FileDropPosition {
  x: number;
  y: number;
}
/** One native path event reported by the window system. */
interface FileDropEvent {
  phase: FileDropPhase;
  /** Native filesystem paths supplied on enter and drop events. */
  paths: string[];
  /** Logical window coordinates, or `null` when unavailable on the platform. */
  position: FileDropPosition | null;
}
type FileDropHandler = (event: FileDropEvent) => void;
/** Subscribe to native file drag-and-drop events for the current window. */
declare function subscribeFileDrop(handler: FileDropHandler): () => void;
/**
 * Subscribe for the lifetime of the current Solid owner.
 * Use `subscribeFileDrop` when no Solid owner is active.
 */
declare function useFileDrop(handler: FileDropHandler): void;
//#endregion
export { subscribeFileDrop as a, FileDropPosition as i, FileDropHandler as n, useFileDrop as o, FileDropPhase as r, FileDropEvent as t };
//# sourceMappingURL=file-drop-Ci2RppE8.d.mts.map