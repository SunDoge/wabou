import { expect, test } from "bun:test";
import {
  gpuiBoundaryViolations,
  legacyIsolationViolations,
} from "./check-gpui-boundary";

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
  ).toEqual([
    "wabou-runtime -> legacy-shell (wabou-legacy-shell, dev)",
    "wabou-runtime -> vello (vello, normal)",
  ]);
});

test("retired crates stay unpublished and outside default workspace commands", () => {
  expect(
    legacyIsolationViolations({
      packages: [
        {
          dependencies: [],
          id: "legacy-shell-id",
          name: "wabou-legacy-shell",
          publish: null,
        },
        {
          dependencies: [],
          id: "legacy-runtime-id",
          name: "wabou-legacy-runtime",
          publish: [],
        },
      ],
      workspace_default_members: ["legacy-shell-id"],
    }),
  ).toEqual([
    "wabou-legacy-shell is a default workspace member",
    "wabou-legacy-shell is publishable",
  ]);
});
