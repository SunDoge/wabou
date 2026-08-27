import { n as dispatchHostMessage, t as dispatchResizeObservation } from "./resize-observer-bJVAKxKD.mjs";
//#region src/testing.ts
/** Publish one native file-drop event without requiring a window backend. */
function dispatchFileDropEvent(event) {
	dispatchHostMessage("wabou:file-drop", JSON.stringify(event));
}
//#endregion
export { dispatchFileDropEvent, dispatchResizeObservation };

//# sourceMappingURL=testing.mjs.map