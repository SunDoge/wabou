//#region src/polyfills/fetch.d.ts
interface FetchResponseData {
  status: number;
  statusText: string;
  headers: Record<string, string>;
  body: Uint8Array;
  url?: string;
  redirected?: boolean;
}
type HeadersInit = WabouHeaders | Record<string, string> | Iterable<readonly [string, string]>;
declare class WabouHeaders implements Iterable<[string, string]> {
  private readonly entriesByName;
  constructor(init?: HeadersInit);
  append(name: string, value: string): void;
  delete(name: string): void;
  get(name: string): string | null;
  has(name: string): boolean;
  set(name: string, value: string): void;
  entries(): MapIterator<[string, string]>;
  keys(): MapIterator<string>;
  values(): MapIterator<string>;
  forEach(callback: (value: string, key: string, headers: WabouHeaders) => void, thisArg?: unknown): void;
  [Symbol.iterator](): MapIterator<[string, string]>;
  toRecord(): Record<string, string>;
}
interface ResponseInit {
  status?: number;
  statusText?: string;
  headers?: HeadersInit;
}
export declare class WabouResponse {
  readonly body: ReadableStream<Uint8Array> | null;
  readonly headers: WabouHeaders;
  readonly status: number;
  readonly statusText: string;
  readonly url: string;
  readonly redirected: boolean;
  private bodyBytes;
  private consumed;
  constructor(body?: string | Uint8Array | ArrayBuffer | null, init?: ResponseInit, url?: string, copyBody?: boolean, redirected?: boolean);
  get ok(): boolean;
  get bodyUsed(): boolean;
  private consumeBody;
  text(): Promise<string>;
  json(): Promise<unknown>;
  bytes(): Promise<Uint8Array>;
  arrayBuffer(): Promise<ArrayBuffer>;
  clone(): WabouResponse;
  static json(value: unknown, init?: ResponseInit): WabouResponse;
  static fromHost(data: FetchResponseData, url: string): WabouResponse;
}
//#endregion
export {};
//# sourceMappingURL=web.d.mts.map