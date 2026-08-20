import { describe, expect, test } from "bun:test";
import { downloadUriError, downloadUris } from "./download-uri";

describe("download URI input", () => {
  test("normalizes lines and ignores comments", () => {
    expect(
      downloadUris(
        " # mirror\n https://example.com/a \n\nmagnet:?xt=urn:btih:a",
      ),
    ).toEqual(["https://example.com/a", "magnet:?xt=urn:btih:a"]);
  });

  test("accepts supported schemes and identifies the invalid line", () => {
    expect(
      downloadUriError("https://example.com/a\nmagnet:?xt=urn:btih:a"),
    ).toBeUndefined();
    expect(downloadUriError("https://example.com/a\nfile:///tmp/private")).toBe(
      "Line 2 must use HTTP, HTTPS, or magnet.",
    );
    expect(downloadUriError("not a uri")).toBe(
      "Line 1 is not a valid download URI.",
    );
  });
});
