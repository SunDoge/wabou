import { expect, test } from "bun:test";
import type { Host } from "@wabou/core";
import { createDevtoolsClient } from "./generated/host-bindings";

test("generated DevTools client sends one structured request", async () => {
  const calls: unknown[][] = [];
  const client = createDevtoolsClient({
    devtools: {
      __wabouCapabilityVersion: 1,
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

  await expect(
    client.setOverlay({
      layout: true,
      clips: true,
      hitTarget: true,
      selectedNode: null,
    }),
  ).resolves.toMatchObject({ layout: true });
  expect(calls).toEqual([
    [
      JSON.stringify({
        layout: true,
        clips: true,
        hitTarget: true,
        selectedNode: null,
      }),
    ],
  ]);
});

test("generated no-request methods are argument-free at the native boundary", async () => {
  const calls: unknown[][] = [];
  const client = createDevtoolsClient({
    devtools: {
      __wabouCapabilityVersion: 1,
      status: (...args: unknown[]) => {
        calls.push(args);
        return JSON.stringify({
          ok: true,
          value: {
            connected: true,
            selectedNode: null,
            overlay: {
              layout: false,
              clips: false,
              hitTarget: false,
              selectedNode: null,
            },
            overlayPaint: {
              sequence: 12,
              enabled: true,
              layout_bounds: 8,
              clip_bounds: 2,
              highlights: 1,
            },
          },
        });
      },
    },
  } as unknown as Host);

  await expect(client.status()).resolves.toMatchObject({
    connected: true,
    overlayPaint: { sequence: 12, layout_bounds: 8 },
  });
  expect(calls).toEqual([[]]);
});
