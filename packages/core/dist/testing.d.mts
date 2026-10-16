import { g as NodeKey } from "./protocol-Dp6QxGZP.mjs";
import { t as FileDropEvent } from "./file-drop-Ci2RppE8.mjs";
//#region src/glue/resize-observer.d.ts
export declare function dispatchResizeObservation(solidId: NodeKey, width: number, height: number): void;
//#endregion
//#region src/testing.d.ts
/** Publish one native file-drop event without requiring a window backend. */
export declare function dispatchFileDropEvent(event: FileDropEvent): void;
/** Publish an arbitrary application message in component tests. */
export declare function dispatchHostMessageForTest(topic: string, payload: unknown): void;
//#endregion
//# sourceMappingURL=testing.d.mts.map