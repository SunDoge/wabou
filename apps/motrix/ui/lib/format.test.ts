import { describe, expect, test } from "bun:test";
import { formatDateTime } from "./format";

describe("Motrix formatting", () => {
  test("does not expose invalid task timestamps to the UI", () => {
    expect(formatDateTime(0)).toBe("Unknown");
    expect(formatDateTime(Number.NaN)).toBe("Unknown");
    expect(formatDateTime(Number.POSITIVE_INFINITY)).toBe("Unknown");
  });

  test("formats a valid engine timestamp", () => {
    expect(formatDateTime(Date.UTC(2026, 7, 20, 8, 30))).not.toBe("Unknown");
  });
});
