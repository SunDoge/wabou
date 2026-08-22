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

test("generated snapshot validation preserves report evidence", async () => {
  const calls: unknown[][] = [];
  const client = createDevtoolsClient({
    devtools: {
      __wabouCapabilityVersion: 1,
      validateSnapshot: (...args: unknown[]) => {
        calls.push(args);
        return JSON.stringify({
          ok: true,
          value: {
            revision: 42,
            valid: false,
            errorCount: 1,
            warningCount: 0,
            truncated: false,
            issues: [
              {
                level: "error",
                code: "missing-parent",
                message: "node references a missing parent",
                nodeId: { lo: 7, hi: 1 },
              },
            ],
          },
        });
      },
    },
  } as unknown as Host);

  await expect(client.validateSnapshot()).resolves.toMatchObject({
    revision: 42,
    valid: false,
    issues: [{ code: "missing-parent", nodeId: { lo: 7, hi: 1 } }],
  });
  expect(calls).toEqual([[]]);
});

test("generated point inspection preserves logical coordinates", async () => {
  const calls: unknown[][] = [];
  const client = createDevtoolsClient({
    devtools: {
      __wabouCapabilityVersion: 1,
      inspectAtPoint: (...args: unknown[]) => {
        calls.push(args);
        return JSON.stringify({
          ok: true,
          value: { x: 320, y: 180, node: null, ancestors: [] },
        });
      },
    },
  } as unknown as Host);

  await expect(
    client.inspectAtPoint({ x: 320, y: 180 }),
  ).resolves.toMatchObject({ x: 320, y: 180, node: null });
  expect(calls).toEqual([[JSON.stringify({ x: 320, y: 180 })]]);
});

test("generated atomic capture preserves an absent point", async () => {
  const calls: unknown[][] = [];
  const client = createDevtoolsClient({
    devtools: {
      __wabouCapabilityVersion: 1,
      captureCase: (...args: unknown[]) => {
        calls.push(args);
        return JSON.stringify({
          ok: true,
          value: {
            screenshotPath: "/tmp/frame.png",
            snapshot: { status: {}, nodes: [] },
            frames: [],
            point: null,
          },
        });
      },
    },
  } as unknown as Host);

  await expect(client.captureCase({ x: null, y: null })).resolves.toMatchObject(
    { screenshotPath: "/tmp/frame.png", point: null },
  );
  expect(calls).toEqual([[JSON.stringify({ x: null, y: null })]]);
});
