import { expect, test } from "bun:test";
import { installFetchPolyfill } from "./fetch";

test("installs interoperable Headers and Response globals", async () => {
  const runtime = globalThis as typeof globalThis & {
    __wabou_fetch?: (
      url: string,
      init: string,
    ) => Promise<{
      status: number;
      statusText: string;
      headers: Record<string, string>;
      body: Uint8Array;
    }>;
  };
  const previous = {
    fetch: runtime.fetch,
    Headers: runtime.Headers,
    Response: runtime.Response,
    hostFetch: runtime.__wabou_fetch,
  };

  try {
    Reflect.deleteProperty(runtime, "Headers");
    Reflect.deleteProperty(runtime, "Response");
    runtime.__wabou_fetch = async (_url: string, init: string) => {
      expect(JSON.parse(init)).toEqual({
        method: "POST",
        headers: { accept: "application/json" },
        body: "request",
      });
      return {
        status: 201,
        statusText: "Created",
        headers: { "Content-Type": "application/json" },
        body: new TextEncoder().encode('{"ok":true}'),
      };
    };

    installFetchPolyfill();

    const headers = new Headers({ Accept: "text/plain" });
    headers.append("ACCEPT", "application/json");
    expect(headers.get("accept")).toBe("text/plain, application/json");
    expect([...headers]).toEqual([["accept", "text/plain, application/json"]]);

    const json = Response.json({ value: 42 }, { status: 202 });
    expect(json).toBeInstanceOf(Response);
    expect(json.status).toBe(202);
    expect(json.headers.get("content-type")).toBe("application/json");
    expect(await json.clone().json()).toEqual({ value: 42 });

    const response = await fetch("https://example.test/items", {
      method: "POST",
      headers: new Headers({ Accept: "application/json" }),
      body: "request",
    });
    expect(response).toBeInstanceOf(Response);
    expect(response.status).toBe(201);
    expect(response.ok).toBe(true);
    expect(response.headers.get("content-type")).toBe("application/json");
    const arrayBufferClone = response.clone();
    const bytesClone = response.clone();
    expect(await response.json()).toEqual({ ok: true });
    expect(response.bodyUsed).toBe(true);
    expect(() => response.clone()).toThrow("consumed");
    expect(new Uint8Array(await arrayBufferClone.arrayBuffer())).toEqual(
      new TextEncoder().encode('{"ok":true}'),
    );
    expect(
      await (bytesClone as unknown as WabouBinaryResponse).bytes(),
    ).toEqual(new TextEncoder().encode('{"ok":true}'));

    const streamed = await fetch("https://example.test/items", {
      method: "POST",
      headers: new Headers({ Accept: "application/json" }),
      body: "request",
    });
    const reader = streamed.body?.getReader();
    expect(reader).toBeDefined();
    expect(await reader?.read()).toEqual({
      done: false,
      value: new TextEncoder().encode('{"ok":true}'),
    });
    expect(await reader?.read()).toEqual({ done: true, value: undefined });
    reader?.releaseLock();
    expect(streamed.bodyUsed).toBe(true);
  } finally {
    runtime.fetch = previous.fetch;
    runtime.Headers = previous.Headers;
    runtime.Response = previous.Response;
    if (previous.hostFetch) runtime.__wabou_fetch = previous.hostFetch;
    else Reflect.deleteProperty(runtime, "__wabou_fetch");
  }
});

interface WabouBinaryResponse {
  bytes(): Promise<Uint8Array>;
}

test("fetch propagates abort reason and cancels the native request", async () => {
  const runtime = globalThis as typeof globalThis & {
    __wabou_fetch?: (...args: any[]) => Promise<any>;
    __wabou_fetch_start?: (id: number) => void;
    __wabou_fetch_cancel?: (id: number) => void;
  };
  const previous = {
    fetch: runtime.fetch,
    host: runtime.__wabou_fetch,
    start: runtime.__wabou_fetch_start,
    cancel: runtime.__wabou_fetch_cancel,
  };
  let calls = 0;
  const started: number[] = [];
  const cancelled: number[] = [];
  let finish!: (value: any) => void;
  try {
    runtime.__wabou_fetch = () => {
      calls++;
      return new Promise((resolve) => {
        finish = resolve;
      });
    };
    runtime.__wabou_fetch_start = (id) => started.push(id);
    runtime.__wabou_fetch_cancel = (id) => cancelled.push(id);
    installFetchPolyfill();
    const reason = new Error("cancel test");
    const pre = new AbortController();
    pre.abort(reason);
    await expect(
      fetch("https://example.test", { signal: pre.signal }),
    ).rejects.toBe(reason);
    expect(calls).toBe(0);
    expect(started).toEqual([]);
    const c = new AbortController();
    const pending = fetch("https://example.test", { signal: c.signal });
    c.abort(reason);
    await expect(pending).rejects.toBe(reason);
    expect(cancelled).toEqual(started);
    finish({
      status: 200,
      statusText: "OK",
      headers: {},
      body: new Uint8Array(),
    });
    await Promise.resolve();
  } finally {
    runtime.fetch = previous.fetch;
    for (const [name, value] of [
      ["__wabou_fetch", previous.host],
      ["__wabou_fetch_start", previous.start],
      ["__wabou_fetch_cancel", previous.cancel],
    ] as const) {
      if (value) Reflect.set(runtime, name, value);
      else Reflect.deleteProperty(runtime, name);
    }
  }
});

test("buffered response transfers a large host body without recopying", async () => {
  const { WabouResponse } = await import("./fetch");
  const body = new Uint8Array(8 * 1024 * 1024);
  body[0] = 17;
  body[body.length - 1] = 29;
  const response = WabouResponse.fromHost(
    { status: 200, statusText: "OK", headers: {}, body },
    "https://example.test",
  );
  const bytes = await response.bytes();
  expect(bytes).toBe(body);
  expect(bytes[0]).toBe(17);
  expect(bytes[bytes.length - 1]).toBe(29);
  await expect(response.bytes()).rejects.toBeInstanceOf(TypeError);
  const arrayResponse = WabouResponse.fromHost(
    { status: 200, statusText: "OK", headers: {}, body },
    "https://example.test",
  );
  expect(await arrayResponse.arrayBuffer()).toBe(body.buffer);
});

test("response construction and cloning isolate externally mutable input", async () => {
  const { WabouResponse } = await import("./fetch");
  const input = new Uint8Array([1, 2, 3]);
  const response = new WabouResponse(input);
  input[0] = 9;
  const clone = response.clone();
  const consumed = await response.bytes();
  consumed[1] = 8;
  expect(Array.from(await clone.bytes())).toEqual([1, 2, 3]);
  expect(consumed[0]).toBe(1);
  expect(new WabouResponse(null).clone().body).toBeNull();
});

test("arrayBuffer only exposes the bytes of a host body view", async () => {
  const { WabouResponse } = await import("./fetch");
  const backing = new Uint8Array([99, 1, 2, 88]);
  const response = WabouResponse.fromHost(
    {
      status: 200,
      statusText: "OK",
      headers: {},
      body: backing.subarray(1, 3),
    },
    "https://example.test",
  );
  const buffer = await response.arrayBuffer();
  expect(buffer).not.toBe(backing.buffer);
  expect(Array.from(new Uint8Array(buffer))).toEqual([1, 2]);
});
