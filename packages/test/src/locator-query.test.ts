import { describe, expect, test } from "bun:test";
import {
  decodeLocatorQuery,
  LocatorAmbiguousError,
  locatorQueryIsAbsent,
  locatorQueryMatchCount,
} from "./locator-query";

describe("strict native locator queries", () => {
  test("distinguishes missing, unique, and ambiguous matches", () => {
    expect(decodeLocatorQuery(null, "button named Save")).toBeNull();
    expect(
      decodeLocatorQuery<{ value: number }>(
        JSON.stringify({ matchCount: 1, snapshot: { value: 42 } }),
        "button named Save",
      ),
    ).toEqual({ value: 42 });
    expect(() =>
      decodeLocatorQuery(
        JSON.stringify({ matchCount: 2, snapshot: { value: 42 } }),
        "button named Save",
      ),
    ).toThrow(
      new LocatorAmbiguousError(
        "found 2 matches for button named Save; expected exactly one",
      ),
    );
  });

  test("rejects malformed host responses", () => {
    expect(() =>
      decodeLocatorQuery(
        JSON.stringify({ matchCount: 0, snapshot: {} }),
        "button named Save",
      ),
    ).toThrow("native locator query returned an invalid match count");
  });

  test("allows an explicit zero-based occurrence without weakening strict defaults", () => {
    const raw = JSON.stringify({
      matchCount: 2,
      snapshot: { value: "second" },
    });
    expect(
      decodeLocatorQuery<{ value: string }>(raw, "button named Default", 1),
    ).toEqual({ value: "second" });
    expect(
      decodeLocatorQuery<{ value: string }>(raw, "button named Default", 2),
    ).toBeNull();
  });

  test("defines absence for strict and indexed locators", () => {
    const twoMatches = JSON.stringify({ matchCount: 2, snapshot: null });
    expect(locatorQueryIsAbsent(null)).toBeTrue();
    expect(locatorQueryIsAbsent(undefined)).toBeTrue();
    expect(locatorQueryIsAbsent(twoMatches)).toBeFalse();
    expect(locatorQueryIsAbsent(twoMatches, 1)).toBeFalse();
    expect(locatorQueryIsAbsent(twoMatches, 2)).toBeTrue();
  });

  test("counts missing and repeated matches without weakening strict lookup", () => {
    expect(locatorQueryMatchCount(null)).toBe(0);
    expect(
      locatorQueryMatchCount(JSON.stringify({ matchCount: 3, snapshot: null })),
    ).toBe(3);
  });
});
