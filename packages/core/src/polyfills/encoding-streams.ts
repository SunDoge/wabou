import {
  TextDecoderStream,
  TextEncoderStream,
} from "@stardazed/streams-text-encoding";

const encodingStreamGlobals = { TextDecoderStream, TextEncoderStream };

/** Install the Encoding Standard stream transforms missing from QuickJS. */
export function installEncodingStreamsPolyfill(): void {
  for (const [name, constructor] of Object.entries(encodingStreamGlobals)) {
    if (name in globalThis) continue;
    Object.defineProperty(globalThis, name, {
      configurable: true,
      writable: true,
      value: constructor,
    });
  }
}

installEncodingStreamsPolyfill();
