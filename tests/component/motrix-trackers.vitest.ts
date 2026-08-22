import { describe, expect, test } from "vitest";
import type { DownloadTask } from "../../apps/motrix/ui/downloads";
import { trackerEndpoints } from "../../apps/motrix/ui/pages/trackers";

const task = (id: string, uri?: string): DownloadTask => ({
  id,
  name: id,
  status: "active",
  totalLength: 0,
  completedLength: 0,
  downloadSpeed: 0,
  uploadSpeed: 0,
  uploadedLength: 0,
  dir: "/tmp",
  uri,
  connections: 0,
  bittorrent: true,
  retryable: true,
  archived: false,
  fileCount: 1,
  priority: "normal",
  createdAtMs: 0,
});

describe("trackerEndpoints", () => {
  test("decodes, deduplicates and counts magnet tracker endpoints", () => {
    const shared = "udp://tracker.example:6969/announce";
    const endpoints = trackerEndpoints([
      task("one", `magnet:?xt=urn:btih:ONE&tr=${encodeURIComponent(shared)}`),
      task(
        "two",
        `magnet:?xt=urn:btih:TWO&tr=${encodeURIComponent(shared)}&tr=${encodeURIComponent("https://tracker.example/announce")}`,
      ),
    ]);

    expect(endpoints).toEqual([
      {
        url: "https://tracker.example/announce",
        protocol: "https",
        tasks: 1,
      },
      { url: shared, protocol: "udp", tasks: 2 },
    ]);
  });

  test("ignores malformed, non-magnet and non-BitTorrent sources", () => {
    const http = {
      ...task("http", "https://example.com/file"),
      bittorrent: false,
    };
    expect(
      trackerEndpoints([task("bad", "magnet:?tr=%E0%A4%A"), http]),
    ).toEqual([]);
  });
});
