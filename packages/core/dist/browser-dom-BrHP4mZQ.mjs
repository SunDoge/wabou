//#region src/browser-dom.ts
/** Native host shims must not replace the surrounding page's browser APIs. */
function hasBrowserDom() {
	return typeof document !== "undefined" && document.nodeType === 9;
}
//#endregion
export { hasBrowserDom as t };

//# sourceMappingURL=browser-dom-BrHP4mZQ.mjs.map