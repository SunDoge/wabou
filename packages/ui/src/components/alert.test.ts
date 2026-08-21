import { describe, expect, test } from "bun:test";
import { alertColors } from "./alert";

describe("Alert", () => {
  test("maps variants to explicit native theme tokens", () => {
    expect(alertColors("default")).toEqual({
      container: "border-subtle bg-surface",
      title: "text-primary",
      description: "text-secondary",
    });
    expect(alertColors("destructive").container).toContain("border-danger");
    expect(alertColors("destructive").description).toBe("text-danger-primary");
  });
});
