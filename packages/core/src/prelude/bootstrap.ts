// Dependency-free globals required before other prelude modules can evaluate.

import "../host";

const runtime = globalThis as Record<string, any>;

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

  constructor(
    _label = "utf-8",
    options: { fatal?: boolean; ignoreBOM?: boolean } = {},
  ) {
    this.fatal = options.fatal ?? false;
    this.ignoreBOM = options.ignoreBOM ?? false;
  }

  decode(value = new Uint8Array()): string {
    const decoded = __wabou_utf8_decode(value);
    if (this.fatal) {
      const encoded = __wabou_utf8_encode(decoded);
      const valid =
        encoded.length === value.length &&
        encoded.every((byte, index) => byte === value[index]);
      if (!valid) throw new TypeError("The encoded data was not valid UTF-8");
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
    __wabou_log(level, values.map(formatConsoleValue).join(" "));
  };
}

// `window` is the global object in browsers. Keeping identity here avoids
// libraries observing two diverging sets of globals.
runtime.window ??= runtime;
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
