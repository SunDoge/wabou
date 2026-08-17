import { expect, test } from "bun:test";
import { installFetchPolyfill } from "./fetch";

test("installs interoperable Headers and Response globals", async () => {
  const runtime = globalThis as typeof globalThis & {
    __wabou_fetch?: (url: string, init: string) => Promise<string>;
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
      return JSON.stringify({
        status: 201,
        statusText: "Created",
        headers: { "Content-Type": "application/json" },
        body: '{"ok":true}',
      });
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
    expect(await response.json()).toEqual({ ok: true });
  } finally {
    runtime.fetch = previous.fetch;
    runtime.Headers = previous.Headers;
    runtime.Response = previous.Response;
    if (previous.hostFetch) runtime.__wabou_fetch = previous.hostFetch;
    else Reflect.deleteProperty(runtime, "__wabou_fetch");
  }
});
