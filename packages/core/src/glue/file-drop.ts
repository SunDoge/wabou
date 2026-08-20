import { onCleanup } from "solid-js";
import { subscribeJson } from "./host-messages";

export type FileDropPhase = "entered" | "moved" | "left" | "dropped";

export interface FileDropPosition {
  x: number;
  y: number;
}

/** One native path event reported by the window system. */
export interface FileDropEvent {
  phase: FileDropPhase;
  /** Native filesystem paths supplied on enter and drop events. */
  paths: string[];
  /** Logical window coordinates, or `null` when unavailable on the platform. */
  position: FileDropPosition | null;
}

export type FileDropHandler = (event: FileDropEvent) => void;

function decodeFileDrop(value: unknown): FileDropEvent {
  if (typeof value !== "object" || value === null)
    throw new TypeError("file drop event must be an object");
  const event = value as Partial<FileDropEvent>;
  if (
    event.phase !== "entered" &&
    event.phase !== "moved" &&
    event.phase !== "left" &&
    event.phase !== "dropped"
  ) {
    throw new TypeError("file drop event has an invalid phase");
  }
  if (
    !Array.isArray(event.paths) ||
    !event.paths.every((path) => typeof path === "string")
  ) {
    throw new TypeError("file drop event paths must be strings");
  }
  const position = event.position;
  if (
    position !== null &&
    (typeof position !== "object" ||
      typeof position.x !== "number" ||
      typeof position.y !== "number")
  ) {
    throw new TypeError("file drop event position must be logical coordinates");
  }
  return { phase: event.phase, paths: event.paths, position: position ?? null };
}

/** Subscribe to native file drag-and-drop events for the current window. */
export function subscribeFileDrop(handler: FileDropHandler): () => void {
  return subscribeJson("wabou:file-drop", handler, { decode: decodeFileDrop });
}

/**
 * Subscribe for the lifetime of the current Solid owner.
 * Use `subscribeFileDrop` when no Solid owner is active.
 */
export function useFileDrop(handler: FileDropHandler): void {
  onCleanup(subscribeFileDrop(handler));
}
