// Small fetch surface backed by the Rust host. Keep these objects compatible
// with the standard Web API so framework-agnostic libraries can feature-detect
// and exchange Responses without depending on a browser DOM.

interface FetchInit {
  method?: string;
  headers?: HeadersInit;
  body?: string;
}

interface FetchResponseData {
  status: number;
  statusText: string;
  headers: Record<string, string>;
  body: Uint8Array;
}

type HeadersInit =
  | WabouHeaders
  | Record<string, string>
  | Iterable<readonly [string, string]>;

function normalizeHeaderName(name: string): string {
  return String(name).toLowerCase();
}

class WabouHeaders implements Iterable<[string, string]> {
  private readonly entriesByName = new Map<string, string>();

  constructor(init?: HeadersInit) {
    if (!init) return;
    if (Symbol.iterator in Object(init)) {
      for (const [name, value] of init as Iterable<readonly [string, string]>) {
        this.append(name, value);
      }
      return;
    }
    for (const [name, value] of Object.entries(init)) this.append(name, value);
  }

  append(name: string, value: string): void {
    const key = normalizeHeaderName(name);
    const current = this.entriesByName.get(key);
    this.entriesByName.set(
      key,
      current ? `${current}, ${String(value)}` : String(value),
    );
  }

  delete(name: string): void {
    this.entriesByName.delete(normalizeHeaderName(name));
  }

  get(name: string): string | null {
    return this.entriesByName.get(normalizeHeaderName(name)) ?? null;
  }

  has(name: string): boolean {
    return this.entriesByName.has(normalizeHeaderName(name));
  }

  set(name: string, value: string): void {
    this.entriesByName.set(normalizeHeaderName(name), String(value));
  }

  entries(): MapIterator<[string, string]> {
    return this.entriesByName.entries();
  }

  keys(): MapIterator<string> {
    return this.entriesByName.keys();
  }

  values(): MapIterator<string> {
    return this.entriesByName.values();
  }

  forEach(
    callback: (value: string, key: string, headers: WabouHeaders) => void,
    thisArg?: unknown,
  ): void {
    for (const [key, value] of this.entriesByName) {
      callback.call(thisArg, value, key, this);
    }
  }

  [Symbol.iterator](): MapIterator<[string, string]> {
    return this.entries();
  }

  toRecord(): Record<string, string> {
    return Object.fromEntries(this.entriesByName);
  }
}

interface ResponseInit {
  status?: number;
  statusText?: string;
  headers?: HeadersInit;
}

function encodeResponseBody(
  body: string | Uint8Array | ArrayBuffer | null,
  copy: boolean,
): Uint8Array {
  if (typeof body === "string") return new TextEncoder().encode(body);
  if (body instanceof Uint8Array) return copy ? body.slice() : body;
  if (body instanceof ArrayBuffer) return new Uint8Array(body.slice(0));
  return new Uint8Array();
}

class WabouResponse {
  readonly body: ReadableStream<Uint8Array> | null;
  readonly headers: WabouHeaders;
  readonly status: number;
  readonly statusText: string;
  readonly url: string;
  private readonly bodyBytes: Uint8Array;
  private consumed = false;

  constructor(
    body: string | Uint8Array | ArrayBuffer | null = null,
    init: ResponseInit = {},
    url = "",
    copyBody = true,
  ) {
    this.bodyBytes = encodeResponseBody(body, copyBody);
    this.status = init.status ?? 200;
    this.statusText = init.statusText ?? "";
    this.headers = new WabouHeaders(init.headers);
    this.url = url;
    if (body === null) {
      this.body = null;
    } else {
      const stream = new ReadableStream<Uint8Array>({
        start: (controller) => {
          controller.enqueue(this.bodyBytes);
          controller.close();
        },
      });
      const getReader = stream.getReader.bind(stream);
      stream.getReader = ((...args: Parameters<typeof stream.getReader>) => {
        this.consumed = true;
        return getReader(...args);
      }) as typeof stream.getReader;
      this.body = stream;
    }
  }

  get ok(): boolean {
    return this.status >= 200 && this.status < 300;
  }

  get bodyUsed(): boolean {
    return this.consumed || this.body?.locked === true;
  }

  private async consumeBody(): Promise<Uint8Array> {
    if (this.bodyUsed) throw new TypeError("Response body has already been consumed");
    if (this.body === null) {
      this.consumed = true;
      return new Uint8Array();
    }
    const reader = this.body.getReader();
    const chunks: Uint8Array[] = [];
    let length = 0;
    try {
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        chunks.push(value);
        length += value.length;
      }
    } finally {
      reader.releaseLock();
    }
    const bytes = new Uint8Array(length);
    let offset = 0;
    for (const chunk of chunks) {
      bytes.set(chunk, offset);
      offset += chunk.length;
    }
    return bytes;
  }

  async text(): Promise<string> {
    return new TextDecoder().decode(await this.consumeBody());
  }

  async json(): Promise<unknown> {
    return JSON.parse(await this.text());
  }

  bytes(): Promise<Uint8Array> {
    return this.consumeBody();
  }

  async arrayBuffer(): Promise<ArrayBuffer> {
    return (await this.consumeBody()).slice().buffer as ArrayBuffer;
  }

  clone(): WabouResponse {
    if (this.bodyUsed) throw new TypeError("Cannot clone a consumed Response body");
    return new WabouResponse(
      this.bodyBytes,
      {
        status: this.status,
        statusText: this.statusText,
        headers: this.headers,
      },
      this.url,
    );
  }

  static json(value: unknown, init: ResponseInit = {}): WabouResponse {
    const headers = new WabouHeaders(init.headers);
    if (!headers.has("content-type")) {
      headers.set("content-type", "application/json");
    }
    return new WabouResponse(JSON.stringify(value), { ...init, headers });
  }

  static fromHost(data: FetchResponseData, url: string): WabouResponse {
    return new WabouResponse(
      data.body,
      {
        status: data.status,
        statusText: data.statusText,
        headers: data.headers,
      },
      url,
      false,
    );
  }
}

type FetchRuntime = typeof globalThis & {
  __wabou_fetch(url: string, initJson: string): Promise<FetchResponseData>;
};

/** Install the host-backed Fetch API surface. Safe to call again in tests. */
export function installFetchPolyfill(): void {
  if (!("Headers" in globalThis)) {
    Object.defineProperty(globalThis, "Headers", {
      configurable: true,
      writable: true,
      value: WabouHeaders,
    });
  }
  if (!("Response" in globalThis)) {
    Object.defineProperty(globalThis, "Response", {
      configurable: true,
      writable: true,
      value: WabouResponse,
    });
  }

  // A browser, Bun, or Node runtime may already provide its own fetch. Only
  // install Wabou's implementation when the native host bridge is present.
  if (!("__wabou_fetch" in globalThis)) return;

  globalThis.fetch = ((
    input: string | { url: string },
    init?: FetchInit,
  ): Promise<Response> => {
    const url = typeof input === "string" ? input : input.url;
    const serializedInit = init
      ? {
          ...init,
          headers:
            init.headers instanceof WabouHeaders
              ? init.headers.toRecord()
              : init.headers,
        }
      : {};
    return (globalThis as FetchRuntime)
      .__wabou_fetch(url, JSON.stringify(serializedInit))
      .then((data: FetchResponseData) => {
        const ResponseConstructor =
          globalThis.Response as unknown as typeof WabouResponse;
        if (ResponseConstructor === WabouResponse)
          return WabouResponse.fromHost(data, url) as unknown as Response;
        return new ResponseConstructor(data.body, {
          status: data.status,
          statusText: data.statusText,
          headers: data.headers,
        }) as unknown as Response;
      });
  }) as typeof globalThis.fetch;
}

installFetchPolyfill();
