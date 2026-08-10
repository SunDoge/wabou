import { expect, test } from "bun:test";
import type { Host } from "@wabou/core";
import { createDevtoolsClient } from "./generated/host-bindings";

test("generated DevTools client preserves primitive and optional arguments", async () => {
  const calls: unknown[][] = [];
  const client = createDevtoolsClient({
    devtools: {
      setOverlay: (...args: unknown[]) => {
        calls.push(args);
        return JSON.stringify({
          ok: true,
          value: {
            layout: true,
            clips: true,
            hitTarget: true,
            selectedNode: null,
          },
        });
      },
    },
  } as unknown as Host);

  await expect(client.setOverlay(true, true, true)).resolves.toMatchObject({
    layout: true,
  });
  expect(calls).toEqual([[true, true, true, undefined]]);
});
