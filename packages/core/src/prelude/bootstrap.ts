// Dependency-free globals required before other prelude modules can evaluate.

import "../host";

const runtime = globalThis as Record<string, any>;

// QuickJS defaults to ten captured frames. Reactive feedback loops easily
// consume those inside Solid before the application component is reached,
// leaving even a valid source map with no business-code location to report.
if (typeof runtime.Error === "function") {
  runtime.Error.stackTraceLimit = 100;
}

declare function __wabou_utf8_encode_into(
  value: string,
  output: Uint8Array,
): { read: number; written: number };
function toUSVString(value: unknown): string {
  return String(value).replace(
    /[\uD800-\uDBFF](?![\uDC00-\uDFFF])|(?<![\uD800-\uDBFF])[\uDC00-\uDFFF]/g,
    "\ufffd",
  );
}

class HostTextEncoder {
  readonly encoding = "utf-8";

  encode(value = ""): Uint8Array {
    return __wabou_utf8_encode(toUSVString(value));
  }

  encodeInto(
    value: string,
    destination: Uint8Array,
  ): { read: number; written: number } {
    return __wabou_utf8_encode_into(toUSVString(value), destination);
  }
}

declare const NativeTextDecoderState: new (
  label: string,
  fatal: boolean,
  ignoreBOM: boolean,
) => {
  readonly encoding: string;
  decode(bytes: Uint8Array, stream: boolean): string;
};
class HostTextDecoder {
  readonly fatal: boolean;
  readonly ignoreBOM: boolean;
  private readonly state: InstanceType<typeof NativeTextDecoderState>;
  get encoding() {
    return this.state.encoding;
  }
  constructor(
    label = "utf-8",
    options: { fatal?: boolean; ignoreBOM?: boolean } = {},
  ) {
    this.fatal = Boolean(options.fatal);
    this.ignoreBOM = Boolean(options.ignoreBOM);
    this.state = new NativeTextDecoderState(
      String(label),
      this.fatal,
      this.ignoreBOM,
    );
  }
  decode(
    value: BufferSource = new Uint8Array(),
    options: { stream?: boolean } = {},
  ): string {
    const bytes =
      value instanceof ArrayBuffer
        ? new Uint8Array(value)
        : new Uint8Array(value.buffer, value.byteOffset, value.byteLength);
    return this.state.decode(bytes, Boolean(options.stream));
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

// `self` is the environment-neutral global alias shared by browsers and Web
// Workers. DOM-independent libraries such as TanStack Router use it after an
// explicit client-runtime check, without requiring `window` or `document`.
runtime.self ??= runtime;
