import { expect, test } from "bun:test";
import { gpuiBoundaryViolations } from "./check-gpui-boundary";

test("formal packages cannot regain a retired rendering backend", () => {
  expect(
    gpuiBoundaryViolations({
      packages: [
        {
          name: "wabou-runtime",
          dependencies: [
            { kind: null, name: "wabou-shell", rename: "gpui-shell" },
            { kind: "dev", name: "wabou-legacy-shell", rename: "legacy-shell" },
            { kind: null, name: "vello", rename: null },
          ],
        },
        {
          name: "wabou-legacy-runtime",
          dependencies: [{ kind: null, name: "winit", rename: null }],
        },
      ],
    }),
  ).toEqual(["wabou-runtime -> vello (vello)"]);
});
