/** Native host shims must not replace the surrounding page's browser APIs. */
export function hasBrowserDom(): boolean {
  return typeof document !== "undefined" && document.nodeType === 9;
}
