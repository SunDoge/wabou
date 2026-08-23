// Dependency-free globals required before other prelude modules can evaluate.

import "../host";

const runtime = globalThis as Record<string, any>;

// QuickJS defaults to ten captured frames. Reactive feedback loops easily
// consume those inside Solid before the application component is reached,
// leaving even a valid source map with no business-code location to report.
if (typeof runtime.Error === "function") {
  runtime.Error.stackTraceLimit = 100;
}

class HostTextEncoder {
  readonly encoding = "utf-8";

  encode(value = ""): Uint8Array {
    return __wabou_utf8_encode(value);
  }

  encodeInto(
    value: string,
    destination: Uint8Array,
  ): { read: number; written: number } {
    let read = 0;
    let written = 0;
    for (const character of value) {
      const encoded = this.encode(character);
      if (written + encoded.length > destination.length) break;
      destination.set(encoded, written);
      read += character.length;
      written += encoded.length;
    }
    return { read, written };
  }
}

class HostTextDecoder {
  readonly encoding = "utf-8";
  readonly fatal: boolean;
  readonly ignoreBOM: boolean;
  private pending = new Uint8Array();
  private atStart = true;

  constructor(
    _label = "utf-8",
    options: { fatal?: boolean; ignoreBOM?: boolean } = {},
  ) {
    this.fatal = options.fatal ?? false;
    this.ignoreBOM = options.ignoreBOM ?? false;
  }

  decode(
    value: BufferSource = new Uint8Array(),
    options: { stream?: boolean } = {},
  ): string {
    const input = value instanceof ArrayBuffer
      ? new Uint8Array(value)
      : new Uint8Array(value.buffer, value.byteOffset, value.byteLength);
    const bytes = new Uint8Array(this.pending.length + input.length);
    bytes.set(this.pending);
    bytes.set(input, this.pending.length);
    let completeLength = bytes.length;
    if (options.stream && bytes.length > 0) {
      let lead = bytes.length - 1;
      while (lead > 0 && (bytes[lead] & 0xc0) === 0x80 && bytes.length - lead <= 3) {
        lead -= 1;
      }
      const first = bytes[lead];
      const expected = first >= 0xf0 && first <= 0xf4
        ? 4
        : first >= 0xe0 && first <= 0xef
          ? 3
          : first >= 0xc2 && first <= 0xdf
            ? 2
            : 1;
      if (expected > bytes.length - lead) completeLength = lead;
    }
    const complete = bytes.slice(0, completeLength);
    this.pending = options.stream ? bytes.slice(completeLength) : new Uint8Array();
    let decoded = __wabou_utf8_decode(complete);
    if (this.fatal) {
      const encoded = __wabou_utf8_encode(decoded);
      const valid = encoded.length === complete.length &&
        encoded.every((byte, index) => byte === complete[index]);
      if (!valid) throw new TypeError("The encoded data was not valid UTF-8");
    }
    if (this.atStart && decoded.length > 0) {
      this.atStart = false;
      if (!this.ignoreBOM && decoded.startsWith("\ufeff")) decoded = decoded.slice(1);
    }
    return decoded;
  }
}

runtime.TextEncoder ??= HostTextEncoder;
runtime.TextDecoder ??= HostTextDecoder;

function formatConsoleValue(value: unknown): string {
  if (typeof value === "string") return value;
  if (value instanceof Error) return value.stack ?? value.message;
  try {
    return JSON.stringify(value);
  } catch {
    return String(value);
  }
}

runtime.console ??= {};
for (const level of ["log", "info", "warn", "error", "debug"] as const) {
  runtime.console[level] ??= (...values: unknown[]) => {
    let message = values.map(formatConsoleValue).join(" ");
    if (
      level === "warn" &&
      message.includes("[STRICT_READ_UNTRACKED]") &&
      typeof Error === "function"
    ) {
      const stack = new Error().stack;
      if (stack) message = `${message}\n${stack}`;
    }
    __wabou_log(level, message);
  };
}
runtime.console.assert ??= (condition: unknown, ...values: unknown[]) => {
  if (!condition) {
    runtime.console.error("Assertion failed", ...values);
  }
};

// `window` is the global object in browsers. Keeping identity here avoids
// libraries observing two diverging sets of globals.
runtime.window ??= runtime;
runtime.self ??= runtime;
runtime.scrollX ??= 0;
runtime.scrollY ??= 0;
runtime.scrollTo ??= () => {};
runtime.addEventListener ??= () => {};
runtime.removeEventListener ??= () => {};
runtime.document ??= {
  addEventListener() {},
  removeEventListener() {},
  getElementById() {
    return null;
  },
  baseURI: "http://localhost/",
};
