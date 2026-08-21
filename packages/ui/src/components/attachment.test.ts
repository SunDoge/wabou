import { describe, expect, test } from "bun:test";
import { attachmentClass, attachmentMediaClass } from "./attachment";

const context = (
  state: "done" | "error",
  size: "default" | "sm" | "xs",
  orientation: "horizontal" | "vertical",
) => ({ state: () => state, size: () => size, orientation: () => orientation });

describe("Attachment", () => {
  test("maps state to explicit semantic colors", () => {
    expect(attachmentClass({ state: "error" })).toContain("border-danger");
    expect(attachmentClass({ state: "error" })).toContain("bg-danger-surface");
    expect(attachmentClass({ state: "uploading" })).toContain("border-focus");
  });

  test("maps orientation without group-data selectors", () => {
    expect(attachmentClass({ orientation: "horizontal" })).toContain(
      "flex-row",
    );
    expect(attachmentClass({ orientation: "vertical" })).toContain("flex-col");
  });

  test("gives media deterministic geometry", () => {
    expect(
      attachmentMediaClass("icon", context("done", "sm", "horizontal")),
    ).toContain("w-8");
    expect(
      attachmentMediaClass("image", context("done", "xs", "vertical")),
    ).toContain("w-full");
    expect(
      attachmentMediaClass("image", context("error", "xs", "horizontal")),
    ).toContain("opacity-60");
  });
});
