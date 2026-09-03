import type { FileDropEvent } from "./glue/file-drop";
import { dispatchHostMessage } from "./glue/host-messages";

/** Test-only adapters that publish deterministic host observations. */
export { dispatchResizeObservation } from "./glue/resize-observer";

/** Publish one native file-drop event without requiring a window backend. */
export function dispatchFileDropEvent(event: FileDropEvent): void {
  dispatchHostMessage("wabou:file-drop", JSON.stringify(event));
}

/** Publish an arbitrary application message in component tests. */
export function dispatchHostMessageForTest(
  topic: string,
  payload: unknown,
): void {
  dispatchHostMessage(topic, payload);
}
