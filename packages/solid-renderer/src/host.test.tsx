import { describe, expect, test } from "bun:test";
import { createComponent, createRoot } from "solid-js";
import { type BuiltinHost, type Host, HostProvider, useHost } from "./host";

const fakeBuiltinHost: BuiltinHost = {
  system: { openUrl: () => true },
  fonts: { load: () => true },
  diagnostics: { frameStats: () => null },
  layout: {
    snapshot: () => ({
      revision: 0,
      viewport: { x: 0, y: 0, width: 0, height: 0 },
      nodes: [],
    }),
    measure: () => null,
    clippingRect: () => null,
    viewport: () => ({ x: 0, y: 0, width: 0, height: 0 }),
  },
};
const fakeHost = fakeBuiltinHost as Host;

describe("host context", () => {
  test("binds a host to a Solid subtree", () => {
    let received: Host | undefined;

    createRoot((dispose) => {
      createComponent(HostProvider, {
        value: fakeHost,
        get children() {
          received = useHost();
          return null;
        },
      });
      dispose();
    });

    expect(received).toBe(fakeHost);
  });
});
