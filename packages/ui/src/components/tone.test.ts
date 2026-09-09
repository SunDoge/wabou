import { describe, expect, test } from "bun:test";
import { componentToneClass } from "./tone";

describe("component tone contract", () => {
  test("keeps semantic soft surfaces consistent", () => {
    expect(componentToneClass("accent", "surface")).toBe(
      "border-accent bg-selected text-accent",
    );
    expect(componentToneClass("success", "surface")).toBe(
      "border-success-primary bg-success-surface text-success-primary",
    );
    expect(componentToneClass("danger", "surface")).toBe(
      "border-danger bg-danger-surface text-danger-primary",
    );
  });

  test("uses the same semantic colors for compact indicators", () => {
    expect(componentToneClass("neutral", "indicator")).toBe("bg-muted");
    expect(componentToneClass("accent", "indicator")).toBe("bg-accent");
    expect(componentToneClass("success", "indicator")).toBe(
      "bg-success-primary",
    );
    expect(componentToneClass("danger", "indicator")).toBe("bg-danger");
  });

  test("keeps solid semantic foregrounds independent from the accent", () => {
    expect(componentToneClass("accent", "solid")).toContain("text-on-accent");
    expect(componentToneClass("success", "solid")).toContain("text-on-success");
    expect(componentToneClass("danger", "solid")).toContain("text-on-danger");
  });
});
