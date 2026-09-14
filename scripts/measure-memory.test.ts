import { describe, expect, test } from "bun:test";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { parseLinuxMemory, parseMacMemory } from "./measure-memory";

describe("memory metric units and identity", () => {
  test("Linux RSS, lifetime RSS high water and PSS remain distinct", () => {
    expect(
      parseLinuxMemory(
        "VmRSS:\t2048 kB\nVmHWM: 4096 kB\n",
        "Rss: 2048 kB\nPss: 1536 kB\n",
      ),
    ).toEqual({
      rssBytes: 2097152,
      rssHighWaterBytes: 4194304,
      pssBytes: 1572864,
    });
  });
  test("macOS physical footprint accepts fractional binary units", () => {
    expect(
      parseMacMemory(
        "Physical footprint: 50.5M\nPhysical footprint (peak): 1.25G\n",
      ),
    ).toEqual({
      physicalFootprintBytes: 52953088,
      physicalFootprintPeakBytes: 1342177280,
    });
  });
  test("missing data fails instead of silently reporting zero", () => {
    expect(() => parseMacMemory("permission denied")).toThrow();
    expect(() => parseLinuxMemory("VmRSS: 42 kB", "")).toThrow();
  });
});

test("failed probe writes an incomplete report and exits", async () => {
  const dir = await mkdtemp(join(tmpdir(), "wabou-memory-test-"));
  try {
    const out = join(dir, "report.json");
    const child = Bun.spawn(
      [
        process.execPath,
        "scripts/measure-memory.ts",
        "--probe",
        join(dir, "missing-probe"),
        "--out",
        out,
      ],
      {
        stdout: "ignore",
        stderr: "pipe",
      },
    );
    const stderr = new Response(child.stderr).text();
    expect(await child.exited).not.toBe(0);
    await stderr;
    const report = await Bun.file(out).json();
    expect(report.complete).toBe(false);
    expect(report.error).toContain("ENOENT");
    expect(report.samples).toEqual([]);
  } finally {
    await rm(dir, { recursive: true, force: true });
  }
});

test.skipIf(process.platform !== "linux")(
  "PID sampling observes a live process without terminating it",
  async () => {
    const dir = await mkdtemp(join(tmpdir(), "wabou-memory-pid-"));
    try {
      const out = join(dir, "report.json");
      const child = Bun.spawn(
        [
          process.execPath,
          "scripts/measure-memory.ts",
          "--pid",
          String(process.pid),
          "--label",
          "test-runner",
          "--seconds",
          "1",
          "--interval",
          "0.2",
          "--out",
          out,
        ],
        { stdout: "ignore", stderr: "pipe" },
      );
      const stderr = new Response(child.stderr).text();
      expect(await child.exited).toBe(0);
      expect(await stderr).toBe("");
      const report = await Bun.file(out).json();
      expect(report.complete).toBe(true);
      expect(report.samples.length).toBeGreaterThan(1);
      expect(report.samples[0].memory.rssBytes).toBeGreaterThan(0);
      expect(report.samples[0].memory.pssBytes).toBeGreaterThan(0);
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  },
);
