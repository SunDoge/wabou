import { onCleanup } from "solid-js";
import { subscribe } from "./host-messages";

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

function decodeFileDrop(payload: unknown): FileDropEvent | null {
  if (typeof payload !== "string") return null;
  const value = JSON.parse(payload) as Partial<FileDropEvent>;
  if (
    value.phase !== "entered" &&
    value.phase !== "moved" &&
    value.phase !== "left" &&
    value.phase !== "dropped"
  ) {
    return null;
  }
  if (
    !Array.isArray(value.paths) ||
    !value.paths.every((path) => typeof path === "string")
  ) {
    return null;
  }
  const position = value.position;
  if (
    position !== null &&
    (typeof position !== "object" ||
      typeof position.x !== "number" ||
      typeof position.y !== "number")
  ) {
    return null;
  }
  return { phase: value.phase, paths: value.paths, position: position ?? null };
}

/** Subscribe to native file drag-and-drop events for the current window. */
export function subscribeFileDrop(handler: FileDropHandler): () => void {
  return subscribe("wabou:file-drop", (payload) => {
    const event = decodeFileDrop(payload);
    if (event) handler(event);
  });
}

/**
 * Subscribe for the lifetime of the current Solid owner.
 * Use `subscribeFileDrop` when no Solid owner is active.
 */
export function useFileDrop(handler: FileDropHandler): void {
  onCleanup(subscribeFileDrop(handler));
}
