import "./streams";
import { installMissingGlobals } from "./globals";

/** Stream wrapper over the runtime's stateful native TextDecoder. */
export class WabouTextDecoderStream {
  private readonly decoder: TextDecoder;
  private readonly transform: TransformStream<BufferSource, string>;

  constructor(label = "utf-8", options: TextDecoderOptions = {}) {
    this.decoder = new TextDecoder(label, options);
    this.transform = new TransformStream({
      transform: (chunk, controller) => {
        const text = this.decoder.decode(chunk, { stream: true });
        if (text) controller.enqueue(text);
      },
      flush: (controller) => {
        const text = this.decoder.decode();
        if (text) controller.enqueue(text);
      },
    });
  }
  get encoding() {
    return this.decoder.encoding;
  }
  get fatal() {
    return this.decoder.fatal;
  }
  get ignoreBOM() {
    return this.decoder.ignoreBOM;
  }
  get readable() {
    return this.transform.readable;
  }
  get writable() {
    return this.transform.writable;
  }
  get [Symbol.toStringTag]() {
    return "TextDecoderStream";
  }
}

/** Keep a trailing high surrogate until the next chunk or stream flush. */
export class WabouTextEncoderStream {
  private readonly transform: TransformStream<unknown, Uint8Array>;

  constructor() {
    const encoder = new TextEncoder();
    let pending = "";
    this.transform = new TransformStream({
      transform(chunk, controller) {
        if (typeof chunk === "symbol")
          throw new TypeError("Cannot encode a Symbol");
        let text = pending + String(chunk);
        pending = "";
        const last = text.charCodeAt(text.length - 1);
        if (last >= 0xd800 && last <= 0xdbff) {
          pending = text.slice(-1);
          text = text.slice(0, -1);
        }
        if (text) controller.enqueue(encoder.encode(text));
      },
      flush(controller) {
        if (pending) controller.enqueue(encoder.encode(pending));
        pending = "";
      },
    });
  }
  get encoding() {
    return "utf-8";
  }
  get readable() {
    return this.transform.readable;
  }
  get writable() {
    return this.transform.writable;
  }
  get [Symbol.toStringTag]() {
    return "TextEncoderStream";
  }
}

/** Install the Encoding Standard transforms missing from QuickJS. */
export function installEncodingStreamsPolyfill(): void {
  installMissingGlobals({
    TextDecoderStream: WabouTextDecoderStream,
    TextEncoderStream: WabouTextEncoderStream,
  });
}
installEncodingStreamsPolyfill();
