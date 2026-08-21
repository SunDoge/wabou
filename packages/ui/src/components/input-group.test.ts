import { describe, expect, test } from "bun:test";
import { inputGroupAddonClass, inputGroupClass } from "./forms";

describe("InputGroup", () => {
  test("owns one explicit surface and focus state", () => {
    expect(inputGroupClass("horizontal", false, false)).toContain(
      "h-8 flex-row",
    );
    expect(inputGroupClass("vertical", true, false)).toContain(
      "h-auto flex-col",
    );
    expect(inputGroupClass("vertical", true, false)).toContain("border-focus");
    expect(inputGroupClass("horizontal", true, true)).toContain(
      "border-danger",
    );
  });

  test("maps inline and block addons without selector inference", () => {
    expect(inputGroupAddonClass("inline-start")).toContain("h-full");
    expect(inputGroupAddonClass("block-end")).toContain("w-full");
  });
});
