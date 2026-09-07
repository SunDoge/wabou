import { expect, test } from "bun:test";
import {
  backendFeatureIsolationViolations,
  crossBackendViolations,
  formalVerificationViolations,
  sharedBoundaryViolations,
  transitionalPackagingViolations,
} from "./check-backend-boundaries";

test("Hybrid entry points cannot reactivate the GPUI runtime by default", () => {
  expect(
    backendFeatureIsolationViolations({
      packages: [
        {
          id: "facade-id",
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
        {
          id: "hybrid-id",
          name: "wabou-backend-vello-hybrid",
          dependencies: [
            {
              kind: null,
              name: "wabou-runtime",
              rename: "runtime-api",
              uses_default_features: false,
            },
          ],
        },
      ],
    }),
  ).toEqual([
    "wabou -> wabou-runtime (wabou-runtime, normal) enables default GPUI features",
  ]);
});

test("backend-neutral packages cannot import either backend", () => {
  expect(
    sharedBoundaryViolations({
      packages: [
        {
          id: "protocol-id",
          name: "wabou-protocol",
          dependencies: [
            { kind: null, name: "wabou-shell", rename: null },
            { kind: "dev", name: "wabou-shell-vello", rename: "vello-shell" },
            { kind: null, name: "vello", rename: null },
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
    "wabou-protocol -> vello (vello, normal)",
    "wabou-protocol -> vello-shell (wabou-shell-vello, dev)",
    "wabou-protocol -> wabou-shell (wabou-shell, normal)",
  ]);
});

test("backend packages cannot acquire new cross-backend dependencies", () => {
  expect(
    crossBackendViolations({
      packages: [
        {
          id: "shell-id",
          name: "wabou-shell",
          dependencies: [
            { kind: null, name: "wabou-widgets-vello", rename: null },
          ],
        },
        {
          id: "legacy-runtime-id",
          name: "wabou-backend-vello-hybrid",
          dependencies: [
            { kind: null, name: "wabou-shell", rename: "gpui-shell" },
            { kind: null, name: "wabou-runtime", rename: "runtime-api" },
          ],
        },
        {
          id: "legacy-widgets-id",
          name: "wabou-widgets-vello",
          dependencies: [{ kind: null, name: "wabou-terminal", rename: null }],
        },
      ],
    }),
  ).toEqual([
    "wabou-backend-vello-hybrid -> gpui-shell (wabou-shell, normal)",
    "wabou-shell -> wabou-widgets-vello (wabou-widgets-vello, normal)",
    "wabou-widgets-vello -> wabou-terminal (wabou-terminal, normal)",
  ]);
});

test("shared runtime permits only feature-gated GPUI host dependencies", () => {
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
            {
              kind: "dev",
              name: "gpui-ce",
              optional: false,
              rename: "gpui",
            },
            {
              kind: null,
              name: "gpui_ce_platform",
              optional: false,
              rename: "gpui-platform",
            },
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
    "wabou-runtime -> gpui-platform (gpui_ce_platform, normal)",
    "wabou-runtime -> winit (winit, normal)",
  ]);
});

test("every formal workspace crate participates in ordinary verification", () => {
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
          name: "wabou-future-widget",
          dependencies: [],
        },
        {
          id: "vello-id",
          name: "wabou-shell-vello",
          dependencies: [],
        },
      ],
      workspace_members: ["shell-id", "future-id", "vello-id"],
      workspace_default_members: ["shell-id", "vello-id"],
    }),
  ).toEqual([
    "wabou-future-widget is missing from formal default workspace members",
  ]);
});

test("legacy crates stay unpublished and outside default workspace commands", () => {
  expect(
    transitionalPackagingViolations({
      packages: [
        {
          dependencies: [],
          id: "legacy-widgets-id",
          name: "wabou-legacy-widgets",
          publish: null,
        },
        {
          dependencies: [],
          id: "legacy-runtime-id",
          name: "wabou-legacy-runtime",
          publish: [],
        },
      ],
      workspace_default_members: ["legacy-widgets-id"],
    }),
  ).toEqual([
    "wabou-legacy-widgets is a default workspace member",
    "wabou-legacy-widgets is publishable",
  ]);
});
