import { expect, test } from "bun:test";
import {
  backendFeatureIsolationViolations,
  formalVerificationViolations,
  sharedBoundaryViolations,
} from "./check-backend-boundaries";

test("backend entry points cannot reactivate runtime backend defaults", () => {
  expect(
    backendFeatureIsolationViolations({
      packages: [
        {
          id: "runtime-id",
          name: "wabou",
          dependencies: [
            {
              kind: null,
              name: "wabou-runtime",
              rename: null,
              uses_default_features: true,
            },
          ],
        },
      ],
    }),
  ).toEqual([
    "wabou -> wabou-runtime (wabou-runtime, normal) enables default backend features",
  ]);
});

test("renderer-neutral packages cannot import the native renderer", () => {
  expect(
    sharedBoundaryViolations({
      packages: [
        {
          id: "protocol-id",
          name: "wabou-protocol",
          dependencies: [
            { kind: null, name: "wabou-shell", rename: null },
            { kind: "dev", name: "wabou-shell", rename: "native-shell" },
            { kind: null, name: "vello", rename: null },
            { kind: null, name: "vello_gpu", rename: "vello-gpu" },
          ],
        },
        {
          id: "app-id",
          name: "example-app",
          dependencies: [{ kind: null, name: "winit", rename: null }],
        },
      ],
    }),
  ).toEqual([
    "wabou-protocol -> native-shell (wabou-shell, dev)",
    "wabou-protocol -> vello (vello, normal)",
    "wabou-protocol -> vello-gpu (vello_gpu, normal)",
    "wabou-protocol -> wabou-shell (wabou-shell, normal)",
  ]);
});

test("shared runtime rejects native host dependencies", () => {
  expect(
    sharedBoundaryViolations({
      packages: [
        {
          id: "runtime-id",
          name: "wabou-runtime",
          dependencies: [
            {
              kind: null,
              name: "wabou-shell",
              optional: true,
              rename: null,
            },
            { kind: "dev", name: "vello", optional: false, rename: null },
            {
              kind: null,
              name: "winit",
              optional: true,
              rename: null,
            },
          ],
        },
      ],
    }),
  ).toEqual([
    "wabou-runtime -> vello (vello, dev)",
    "wabou-runtime -> wabou-shell (wabou-shell, normal)",
    "wabou-runtime -> winit (winit, normal)",
  ]);
});

test("the active framework spine participates in ordinary verification", () => {
  expect(
    formalVerificationViolations({
      packages: [
        {
          id: "shell-id",
          name: "wabou-shell",
          dependencies: [],
        },
        {
          id: "future-id",
          name: "wabou-runtime",
          dependencies: [],
        },
      ],
      workspace_members: ["shell-id", "future-id"],
      workspace_default_members: ["shell-id"],
    }),
  ).toEqual(["wabou-runtime is missing from formal default workspace members"]);
});
