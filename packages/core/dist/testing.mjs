import { n as dispatchHostMessage, t as dispatchResizeObservation } from "./resize-observer-tlA51H_x.mjs";
//#region src/testing.ts
/** Publish one native file-drop event without requiring a window backend. */
function dispatchFileDropEvent(event) {
	dispatchHostMessage("wabou:file-drop", JSON.stringify(event));
}
/** Publish an arbitrary application message in component tests. */
function dispatchHostMessageForTest(topic, payload) {
	dispatchHostMessage(topic, payload);
}
//#endregion
export { dispatchFileDropEvent, dispatchHostMessageForTest, dispatchResizeObservation };

//# sourceMappingURL=testing.mjs.map