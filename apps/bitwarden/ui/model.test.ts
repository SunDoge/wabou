import { describe, expect, test } from "bun:test";
import { matches, unwrap } from "./model";

describe("native response boundary", () => {
  test("unwraps successful responses", () => {
    expect(unwrap<number>('{"ok":true,"value":7}')).toBe(7);
  });

  test("does not discard native errors", () => {
    expect(() => unwrap('{"ok":false,"error":"Vault is locked."}')).toThrow(
      "Vault is locked.",
    );
  });
});

test("vault search is case insensitive", () => {
  const item = {
    id: "1",
    name: "Example Login",
    subtitle: "alice@example.com",
    kind: "login",
    favorite: false,
    hasUsername: true,
    hasPassword: true,
    hasTotp: false,
  };
  expect(matches(item, "ALICE")).toBe(true);
  expect(matches(item, "missing")).toBe(false);
});
