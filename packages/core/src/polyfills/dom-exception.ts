class WabouDOMException extends Error {
  readonly code = 0;

  constructor(message = "", name = "Error") {
    super(message);
    this.name = name;
  }
}

/** Install the exception type shared by browser-compatible host APIs. */
export function installDOMExceptionPolyfill(): void {
  if (!("DOMException" in globalThis)) {
    Object.defineProperty(globalThis, "DOMException", {
      configurable: true,
      writable: true,
      value: WabouDOMException,
    });
  }
}

installDOMExceptionPolyfill();
