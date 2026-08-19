import { expect, test } from "bun:test";
import { type Host, HostProvider } from "@wabou/core";
import { createComponent, createRoot } from "solid-js";
import {
  type BindingsDemoClient,
  NativeCapabilityError,
  createBindingsDemoClient,
  useBindingsDemoClient,
} from "./generated/host-bindings";

const resolve = (value: unknown): unknown =>
  typeof value === "function" ? resolve(value()) : value;

test("generated client owns the JSON capability boundary", async () => {
  let encoded = "";
  const host = {
    bindingsDemo: {
      describePalette: async (request: string) => {
        encoded = request;
        return JSON.stringify({
          ok: true,
          value: {
            status: "palette",
            title: "Ocean palette",
            swatches: ["Ocean-1", "Ocean-2"],
          },
        });
      },
    },
  } as unknown as Host;

  const response = await createBindingsDemoClient(host).describePalette({
    name: "Ocean",
    swatchCount: 2,
  });

  expect(JSON.parse(encoded)).toEqual({ name: "Ocean", swatchCount: 2 });
  expect(response).toEqual({
    status: "palette",
    title: "Ocean palette",
    swatches: ["Ocean-1", "Ocean-2"],
  });
});

test("generated client identifies malformed native responses", async () => {
  const client = createBindingsDemoClient({
    bindingsDemo: {
      describePalette: () => "not JSON",
    },
  } as unknown as Host);

  const error = await client
    .describePalette({ name: "broken", swatchCount: 1 })
    .catch((reason: unknown) => reason);

  expect(error).toBeInstanceOf(NativeCapabilityError);
  expect(error).toMatchObject({ operation: "bindingsDemo.describePalette" });
  expect(String(error)).toContain(
    "bindingsDemo.describePalette: invalid JSON response",
  );
});

test("generated client rejects malformed native envelopes", async () => {
  const client = createBindingsDemoClient({
    bindingsDemo: {
      describePalette: () => JSON.stringify({ status: "missing envelope" }),
    },
  } as unknown as Host);

  await expect(
    client.describePalette({ name: "broken", swatchCount: 1 }),
  ).rejects.toThrow(
    "bindingsDemo.describePalette: invalid response envelope",
  );
});

test("generated hook reads the current host context", async () => {
  const host = {
    bindingsDemo: {
      describePalette: () =>
        JSON.stringify({ ok: false, error: "context host reached" }),
    },
  } as unknown as Host;
  let client: BindingsDemoClient | undefined;

  createRoot((dispose) => {
    resolve(
      createComponent(HostProvider, {
        value: host,
        get children() {
          client = useBindingsDemoClient();
          return null;
        },
      }),
    );
    dispose();
  });

  await expect(
    client?.describePalette({ name: "context", swatchCount: 1 }),
  ).rejects.toThrow("context host reached");
});
