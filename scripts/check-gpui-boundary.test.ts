import { expect, test } from "bun:test";
import {
  canonicalDependencyViolations,
  formalVerificationViolations,
  gpuiBoundaryViolations,
  legacyIsolationViolations,
} from "./check-gpui-boundary";

test("formal packages cannot regain a retired rendering backend", () => {
  expect(
    gpuiBoundaryViolations({
      packages: [
        {
          id: "runtime-id",
          name: "wabou-runtime",
          dependencies: [
            { kind: null, name: "wabou-shell", rename: null },
            { kind: "dev", name: "wabou-legacy-shell", rename: "legacy-shell" },
            { kind: null, name: "vello", rename: null },
          ],
        },
        {
          id: "legacy-runtime-id",
          name: "wabou-legacy-runtime",
          dependencies: [{ kind: null, name: "winit", rename: null }],
        },
      ],
      workspace_members: ["runtime-id", "legacy-runtime-id"],
    }),
  ).toEqual([
    "wabou-runtime -> legacy-shell (wabou-legacy-shell, dev)",
    "wabou-runtime -> vello (vello, normal)",
  ]);
});

test("formal packages use the canonical shell dependency name", () => {
  expect(
    canonicalDependencyViolations({
      packages: [
        {
          id: "runtime-id",
          name: "wabou-runtime",
          dependencies: [
            { kind: null, name: "wabou-shell", rename: "gpui-shell" },
          ],
        },
        {
          id: "legacy-runtime-id",
          name: "wabou-legacy-runtime",
          dependencies: [
            { kind: null, name: "wabou-shell", rename: "gpui-shell" },
          ],
        },
      ],
      workspace_members: ["runtime-id", "legacy-runtime-id"],
    }),
  ).toEqual(["wabou-runtime renames wabou-shell to gpui-shell"]);
});

test("every new non-legacy workspace crate inherits the GPUI-only boundary", () => {
  expect(
    gpuiBoundaryViolations({
      packages: [
        {
          id: "future-id",
          name: "wabou-future-widget",
          dependencies: [{ kind: null, name: "winit", rename: null }],
        },
      ],
      workspace_members: ["future-id"],
    }),
  ).toEqual(["wabou-future-widget -> winit (winit, normal)"]);
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
