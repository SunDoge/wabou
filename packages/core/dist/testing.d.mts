import { g as NodeKey } from "./protocol-DVoTIfu0.mjs";
import { t as FileDropEvent } from "./file-drop-Ci2RppE8.mjs";
//#region src/glue/resize-observer.d.ts
declare function dispatchResizeObservation(solidId: NodeKey, width: number, height: number): void;
//#endregion
//#region src/testing.d.ts
/** Publish one native file-drop event without requiring a window backend. */
declare function dispatchFileDropEvent(event: FileDropEvent): void;
//#endregion
export { dispatchFileDropEvent, dispatchResizeObservation };
//# sourceMappingURL=testing.d.mts.map