// fetch polyfill. The Rust host registers `__wabou_fetch(url, initJson)` which
// returns a Promise resolving to a JSON string `{status, statusText, headers, body}`.
// This wrapper turns it into a fetch-like API returning a WabouResponse object
// with .text(), .json(), .status, .statusText, .headers, .ok, .url.

interface FetchInit {
  method?: string;
  headers?: Record<string, string>;
  body?: string;
}

interface FetchResponseData {
  status: number;
  statusText: string;
  headers: Record<string, string>;
  body: string;
}

class WabouResponse {
  private _data: FetchResponseData;
  readonly url: string;

  constructor(data: FetchResponseData, url: string) {
    this._data = data;
    this.url = url;
  }

  get status(): number {
    return this._data.status;
  }

  get statusText(): string {
    return this._data.statusText;
  }

  get ok(): boolean {
    return this._data.status >= 200 && this._data.status < 300;
  }

  get headers(): Record<string, string> {
    return this._data.headers;
  }

  text(): Promise<string> {
    return Promise.resolve(this._data.body);
  }

  json(): Promise<any> {
    return Promise.resolve(JSON.parse(this._data.body));
  }

  clone(): WabouResponse {
    return new WabouResponse(
      { ...this._data, headers: { ...this._data.headers } },
      this.url,
    );
  }
}

(globalThis as any).fetch = function (
  input: string | { url: string },
  init?: FetchInit,
): Promise<WabouResponse> {
  const url = typeof input === "string" ? input : input.url;
  const initJson = init ? JSON.stringify(init) : "{}";
  return (globalThis as any)
    .__wabou_fetch(url, initJson)
    .then((json: string) => {
      const data: FetchResponseData = JSON.parse(json);
      return new WabouResponse(data, url);
    });
};
