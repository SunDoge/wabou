import { expect, test } from "bun:test";
import {
  crossBackendViolations,
  formalVerificationViolations,
  sharedBoundaryViolations,
  transitionalPackagingViolations,
} from "./check-backend-boundaries";

test("backend-neutral packages cannot import either backend", () => {
  expect(
    sharedBoundaryViolations({
      packages: [
        {
          id: "protocol-id",
          name: "wabou-protocol",
          dependencies: [
            { kind: null, name: "wabou-shell", rename: null },
            { kind: "dev", name: "wabou-legacy-shell", rename: "legacy-shell" },
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
    "wabou-protocol -> legacy-shell (wabou-legacy-shell, dev)",
    "wabou-protocol -> vello (vello, normal)",
    "wabou-protocol -> wabou-shell (wabou-shell, normal)",
  ]);
});

test("backend packages cannot acquire new cross-backend dependencies", () => {
  expect(
    crossBackendViolations({
      packages: [
        {
          id: "runtime-id",
          name: "wabou-runtime",
          dependencies: [
            { kind: null, name: "wabou-legacy-widgets", rename: null },
          ],
        },
        {
          id: "legacy-runtime-id",
          name: "wabou-legacy-runtime",
          dependencies: [
            { kind: null, name: "wabou-shell", rename: "gpui-shell" },
            { kind: null, name: "wabou-runtime", rename: "runtime-api" },
          ],
        },
        {
          id: "legacy-widgets-id",
          name: "wabou-legacy-widgets",
          dependencies: [{ kind: null, name: "wabou-terminal", rename: null }],
        },
      ],
    }),
  ).toEqual([
    "wabou-legacy-widgets -> wabou-terminal (wabou-terminal, normal)",
    "wabou-runtime -> wabou-legacy-widgets (wabou-legacy-widgets, normal)",
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
          id: "legacy-id",
          name: "wabou-legacy-shell",
          dependencies: [],
        },
      ],
      workspace_members: ["shell-id", "future-id", "legacy-id"],
      workspace_default_members: ["shell-id"],
    }),
  ).toEqual([
    "wabou-future-widget is missing from formal default workspace members",
  ]);
});

test("transitional backend crates stay unpublished and outside default workspace commands", () => {
  expect(
    transitionalPackagingViolations({
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
