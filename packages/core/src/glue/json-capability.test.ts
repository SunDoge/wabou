import { expect, test } from "bun:test";
import { bindJsonCapability, JsonCapabilityError } from "./json-capability";

test("JSON capability clients encode requests and decode successful envelopes", async () => {
  const requests: string[] = [];
  const capability = {
    __wabouCapabilityVersion: 2,
    double(raw: string) {
      requests.push(raw);
      const request = JSON.parse(raw) as { value: number };
      return JSON.stringify({ ok: true, value: request.value * 2 });
    },
    status() {
      return Promise.resolve(JSON.stringify({ ok: true, value: "ready" }));
    },
  };
  const call = bindJsonCapability(capability, { name: "math", version: 2 });
  expect(await call<number>("double", { value: 21 })).toBe(42);
  expect(await call<string>("status")).toBe("ready");
  expect(requests).toEqual(['{"value":21}']);
});

test("JSON capability clients preserve native error codes and reject drift", async () => {
  const capability = {
    __wabouCapabilityVersion: 1,
    fail() {
      return JSON.stringify({
        ok: false,
        error: { code: "handler_failure", message: "not today" },
      });
    },
    malformed() {
      return "not JSON";
    },
  };
  const call = bindJsonCapability(capability, { name: "demo", version: 1 });
  await expect(call("fail")).rejects.toMatchObject({
    name: "JsonCapabilityError",
    code: "handler_failure",
    message: "not today",
  });
  await expect(call("malformed")).rejects.toBeInstanceOf(JsonCapabilityError);
  await expect(call("missing")).rejects.toMatchObject({
    code: "method_unavailable",
  });
  await expect(
    bindJsonCapability(capability, { name: "demo", version: 2 })("fail"),
  ).rejects.toMatchObject({ code: "capability_unavailable" });
});
