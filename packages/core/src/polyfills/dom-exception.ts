import { installMissingGlobal } from "./globals";

class WabouDOMException extends Error {
  readonly code = 0;

  constructor(message = "", name = "Error") {
    super(message);
    this.name = name;
  }
}

/** Install the exception type shared by browser-compatible host APIs. */
export function installDOMExceptionPolyfill(): void {
  installMissingGlobal("DOMException", WabouDOMException);
}

installDOMExceptionPolyfill();
