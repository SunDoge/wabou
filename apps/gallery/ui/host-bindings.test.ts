import { expect, test } from "bun:test";
import { type Host, HostProvider } from "@wabou/ui";
import { createComponent, createRoot } from "solid-js";
import {
  type BindingsDemoClient,
  type DescribePaletteRequest,
  NativeCapabilityError,
  createBindingsDemoClient,
  createBindingsDemoTestCapability,
  useBindingsDemoClient,
} from "./generated/host-bindings";

const resolve = (value: unknown): unknown =>
  typeof value === "function" ? resolve(value()) : value;

test("generated client owns the JSON capability boundary", async () => {
  let encoded = "";
  const host = {
    bindingsDemo: {
      __wabouCapabilityVersion: 1,
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

test("generated test capability owns JSON envelopes", async () => {
  const capability = createBindingsDemoTestCapability({
    describePalette: ({ name, swatchCount }) => ({
      status: "palette",
      title: `${name} palette`,
      swatches: Array.from(
        { length: swatchCount },
        (_, index) => `${name}-${index + 1}`,
      ),
    }),
  });
  const response = await createBindingsDemoClient({
    bindingsDemo: capability,
  } as unknown as Host).describePalette({ name: "Ocean", swatchCount: 2 });

  expect(response).toEqual({
    status: "palette",
    title: "Ocean palette",
    swatches: ["Ocean-1", "Ocean-2"],
  });
});

test("generated test capability classifies handler failures", async () => {
  const capability = createBindingsDemoTestCapability({
    describePalette: () => {
      throw new Error("fixture unavailable");
    },
  });
  const client = createBindingsDemoClient({
    bindingsDemo: capability,
  } as unknown as Host);

  const error = await client
    .describePalette({ name: "Ocean", swatchCount: 2 })
    .catch((reason: unknown) => reason);

  expect(error).toMatchObject({ code: "handlerFailure" });
  expect(String(error)).toContain("fixture unavailable");
});

test("generated client identifies malformed native responses", async () => {
  const client = createBindingsDemoClient({
    bindingsDemo: {
      __wabouCapabilityVersion: 1,
      describePalette: () => "not JSON",
    },
  } as unknown as Host);

  const error = await client
    .describePalette({ name: "broken", swatchCount: 1 })
    .catch((reason: unknown) => reason);

  expect(error).toBeInstanceOf(NativeCapabilityError);
  expect(error).toMatchObject({
    operation: "bindingsDemo.describePalette",
    code: "invalidResponse",
  });
  expect(String(error)).toContain(
    "bindingsDemo.describePalette: invalid JSON response",
  );
});

test("generated client rejects malformed native envelopes", async () => {
  const client = createBindingsDemoClient({
    bindingsDemo: {
      __wabouCapabilityVersion: 1,
      describePalette: () => JSON.stringify({ status: "missing envelope" }),
    },
  } as unknown as Host);

  await expect(
    client.describePalette({ name: "broken", swatchCount: 1 }),
  ).rejects.toThrow("bindingsDemo.describePalette: invalid response envelope");
});

test("generated client rejects unclassified native failures", async () => {
  const client = createBindingsDemoClient({
    bindingsDemo: {
      __wabouCapabilityVersion: 1,
      describePalette: () =>
        JSON.stringify({ ok: false, error: "legacy string failure" }),
    },
  } as unknown as Host);

  const error = await client
    .describePalette({ name: "broken", swatchCount: 1 })
    .catch((reason: unknown) => reason);

  expect(error).toBeInstanceOf(NativeCapabilityError);
  expect(error).toMatchObject({ code: "invalidResponse" });
  expect(String(error)).toContain("missing an error object");
});

test("generated hook reads the current host context", async () => {
  const host = {
    bindingsDemo: {
      __wabouCapabilityVersion: 1,
      describePalette: () =>
        JSON.stringify({
          ok: false,
          error: { code: "handlerFailure", message: "context host reached" },
        }),
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

test("generated client preserves native error classification", async () => {
  const client = createBindingsDemoClient({
    bindingsDemo: {
      __wabouCapabilityVersion: 1,
      describePalette: () =>
        JSON.stringify({
          ok: false,
          error: {
            code: "invalidRequest",
            message: "swatchCount must be positive",
          },
        }),
    },
  } as unknown as Host);

  const error = await client
    .describePalette({ name: "broken", swatchCount: 0 })
    .catch((reason: unknown) => reason);

  expect(error).toBeInstanceOf(NativeCapabilityError);
  expect(error).toMatchObject({
    operation: "bindingsDemo.describePalette",
    code: "invalidRequest",
  });
  expect(String(error)).toContain("swatchCount must be positive");
});

test("generated client rejects a stale native capability ABI", () => {
  let error: unknown;
  try {
    createBindingsDemoClient({
      bindingsDemo: {
        __wabouCapabilityVersion: 2,
        describePalette: () => "unreachable",
      },
    } as unknown as Host);
  } catch (reason: unknown) {
    error = reason;
  }

  expect(error).toBeInstanceOf(NativeCapabilityError);
  expect(error).toMatchObject({
    operation: "bindingsDemo",
    code: "incompatibleHost",
  });
  expect(String(error)).toContain("bundle=1, host=2");
});

test("generated client rejects an incomplete native capability", async () => {
  const client = createBindingsDemoClient({
    bindingsDemo: {
      __wabouCapabilityVersion: 1,
    },
  } as unknown as Host);
  const error = await client
    .describePalette({ name: "missing", swatchCount: 1 })
    .catch((reason: unknown) => reason);

  expect(error).toBeInstanceOf(NativeCapabilityError);
  expect(error).toMatchObject({
    operation: "bindingsDemo",
    code: "incompatibleHost",
  });
  expect(String(error)).toContain("missing method describePalette");
});

test("generated client classifies native invocation failures", async () => {
  const client = createBindingsDemoClient({
    bindingsDemo: {
      __wabouCapabilityVersion: 1,
      describePalette: async () => {
        throw new Error("QuickJS bridge stopped");
      },
    },
  } as unknown as Host);
  const error = await client
    .describePalette({ name: "broken", swatchCount: 1 })
    .catch((reason: unknown) => reason);

  expect(error).toBeInstanceOf(NativeCapabilityError);
  expect(error).toMatchObject({
    operation: "bindingsDemo.describePalette",
    code: "invocationFailure",
  });
  expect(String(error)).toContain(
    "native invocation failed: QuickJS bridge stopped",
  );
});

test("generated client classifies request encoding failures", async () => {
  let invoked = false;
  const client = createBindingsDemoClient({
    bindingsDemo: {
      __wabouCapabilityVersion: 1,
      describePalette: () => {
        invoked = true;
        return "unreachable";
      },
    },
  } as unknown as Host);
  const request: Record<string, unknown> = {
    name: "cyclic",
    swatchCount: 1,
  };
  request.self = request;
  const error = await client
    .describePalette(request as unknown as DescribePaletteRequest)
    .catch((reason: unknown) => reason);

  expect(invoked).toBe(false);
  expect(error).toBeInstanceOf(NativeCapabilityError);
  expect(error).toMatchObject({
    operation: "bindingsDemo.describePalette",
    code: "requestEncodingFailure",
  });
  expect(String(error)).toContain("cannot encode native request");
});
