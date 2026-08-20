import { describe, expect, test } from "bun:test";
import { parseCurlDownload } from "./curl";

describe("parseCurlDownload", () => {
  test("rejects plain URLs and malformed shell quoting", () => {
    expect(parseCurlDownload("https://example.com/file")).toBeUndefined();
    expect(parseCurlDownload("curl 'https://example.com")).toBeUndefined();
  });

  test("extracts browser copy-as-curl download options", () => {
    expect(
      parseCurlDownload(`curl 'https://example.com/archive.zip' \\
        -H 'Accept: application/zip' \\
        -A 'Motrix Test' -e https://origin.example/ \\
        --cookie='session=abc' --proxy http://127.0.0.1:8080 \\
        --output archive.zip`),
    ).toEqual({
      urls: ["https://example.com/archive.zip"],
      headers: [
        "Accept: application/zip",
        "User-Agent: Motrix Test",
        "Referer: https://origin.example/",
        "Cookie: session=abc",
      ],
      proxy: "http://127.0.0.1:8080",
      output: "archive.zip",
    });
  });

  test("supports multiple positional and explicit URLs", () => {
    expect(
      parseCurlDownload(
        "curl --silent https://one.example/a --url=https://two.example/b",
      )?.urls,
    ).toEqual(["https://one.example/a", "https://two.example/b"]);
  });

  test("does not silently turn upload or POST commands into GET downloads", () => {
    expect(
      parseCurlDownload("curl --data-raw 'a=1' https://example.com/form"),
    ).toBeUndefined();
    expect(
      parseCurlDownload("curl -X POST https://example.com/form"),
    ).toBeUndefined();
  });
});
