import { describe, expect, test } from "bun:test";
import { normalizeOtpValue } from "./input-otp";

describe("InputOTP", () => {
  test("filters and truncates pasted text before it reaches visual slots", () => {
    expect(normalizeOtpValue("12a 34-567", 6)).toBe("123456");
  });

  test("accepts an application-defined character contract", () => {
    expect(normalizeOtpValue("ab-12-CD", 4, /^[A-Z]$/)).toBe("CD");
  });

  test("rejects invalid slot counts", () => {
    expect(() => normalizeOtpValue("1", 0)).toThrow(RangeError);
  });
});
