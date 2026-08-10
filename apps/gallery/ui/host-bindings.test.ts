import { expect, test } from "bun:test";
import type { Host } from "@wabou/core";
import { createBindingsDemoClient } from "./generated/host-bindings";

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
