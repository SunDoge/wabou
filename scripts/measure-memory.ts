import { execFileSync, spawn } from "node:child_process";
import { readFileSync } from "node:fs";
import { hostname, release } from "node:os";
import { createInterface } from "node:readline";
import { parseArgs } from "node:util";

type Memory = Record<string, number>;

export function parseLinuxMemory(status: string, rollup: string): Memory {
  const field = (text: string, key: string) => {
    const match = text.match(new RegExp(`^${key}:\\s+(\\d+) kB$`, "m"));
    if (!match) throw new Error(`missing Linux memory field: ${key}`);
    return Number(match[1]) * 1024;
  };
  return {
    rssBytes: field(status, "VmRSS"),
    rssHighWaterBytes: field(status, "VmHWM"),
    pssBytes: field(rollup, "Pss"),
  };
}

export function parseMacMemory(summary: string): Memory {
  const field = (label: string) => {
    const match = summary.match(
      new RegExp(`^${label}:\\s*([\\d.]+)\\s*([BKMGTP])(?:i?B)?\\s*$`, "mi"),
    );
    if (!match) throw new Error(`missing vmmap field: ${label}`);
    return Math.round(
      Number(match[1]) * 1024 ** "BKMGTP".indexOf(match[2].toUpperCase()),
    );
  };
  return {
    physicalFootprintBytes: field("Physical footprint"),
    physicalFootprintPeakBytes: field("Physical footprint \\(peak\\)"),
  };
}

function sample(pid: number): Memory {
  if (process.platform === "linux") {
    return parseLinuxMemory(
      readFileSync(`/proc/${pid}/status`, "utf8"),
      readFileSync(`/proc/${pid}/smaps_rollup`, "utf8"),
    );
  }
  if (process.platform === "darwin") {
    return parseMacMemory(
      execFileSync("vmmap", ["-summary", String(pid)], {
        encoding: "utf8",
        timeout: 15_000,
        maxBuffer: 4 * 1024 * 1024,
      }),
    );
  }
  throw new Error("memory sampling supports Linux and macOS only");
}

async function main() {
  const { values } = parseArgs({
    args: process.argv
      .slice(2)
      .filter((arg, index) => !(index === 0 && arg === "--")),
    options: {
      probe: { type: "string" },
      pid: { type: "string" },
      out: { type: "string" },
      label: { type: "string" },
      width: { type: "string", default: "800" },
      height: { type: "string", default: "600" },
      scale: { type: "string", default: "1" },
      seconds: { type: "string", default: "60" },
      interval: { type: "string", default: "2" },
      help: { type: "boolean" },
    },
  });
  if (values.help) {
    console.log(
      "Usage: bun run perf:memory --probe target/release/examples/memory_probe --out probe.json [--width 800 --height 600 --scale 1 --seconds 60]\n       bun run perf:memory --pid PID --label gallery-idle --out app.json [--width 800 --height 600 --scale 1 --seconds 60 --interval 2]\nDimensions are logical; application metadata is supplied by the operator, not detected. PID mode attaches without controlling or terminating the application.",
    );
    return;
  }
  if (
    !!values.probe === !!values.pid ||
    !values.out ||
    (values.pid && !values.label)
  ) {
    throw new Error(
      "provide exactly one of --probe/--pid, --out, and --label for PID mode; see --help",
    );
  }
  const positive = (
    name: "width" | "height" | "scale" | "seconds" | "interval",
  ) => {
    const number = Number(values[name]);
    if (!Number.isFinite(number) || number <= 0)
      throw new Error(`invalid --${name}`);
    return number;
  };
  const width = positive("width"),
    height = positive("height"),
    scale = positive("scale");
  const seconds = positive("seconds"),
    interval = positive("interval");
  const physicalWidth = width * scale,
    physicalHeight = height * scale;
  if (
    ![physicalWidth, physicalHeight].every(
      (n) => Number.isInteger(n) && n >= 8 && n <= 65535,
    ) ||
    !Number.isInteger(seconds)
  ) {
    throw new Error(
      "physical dimensions must be integers in 8..65535; seconds must be an integer",
    );
  }
  const pid = values.pid ? Number(values.pid) : undefined;
  if (pid !== undefined && (!Number.isSafeInteger(pid) || pid <= 0))
    throw new Error("invalid PID");
  let revision = "unknown",
    dirty = false;
  try {
    revision = execFileSync("git", ["rev-parse", "HEAD"], {
      encoding: "utf8",
    }).trim();
    dirty =
      execFileSync("git", ["status", "--porcelain"], {
        encoding: "utf8",
      }).trim().length > 0;
  } catch {
    /* Reports also work outside the checkout. */
  }
  const started = performance.now();
  const samples: {
    elapsedMs: number;
    stage: string;
    memory: Memory;
    details?: unknown;
  }[] = [];
  const report = {
    schemaVersion: 1,
    kind: values.probe ? "hybrid-memory-probe" : "application-memory",
    label: values.label ?? "two-rectangles",
    startedAt: new Date().toISOString(),
    host: {
      platform: process.platform,
      release: release(),
      arch: process.arch,
      hostname: hostname(),
    },
    checkout: { revision, dirty },
    viewport: {
      logicalWidth: width,
      logicalHeight: height,
      scaleFactor: scale,
      physicalWidth,
      physicalHeight,
    },
    source: values.probe ?? pid,
    seconds,
    intervalSeconds: values.probe ? null : interval,
    complete: false,
    error: null as string | null,
    samples,
    sampledMaxBytes: {} as Memory,
  };
  const record = (target: number, stage: string, details?: unknown) => {
    const memory = sample(target);
    samples.push({
      elapsedMs: performance.now() - started,
      stage,
      memory,
      details,
    });
    for (const [key, value] of Object.entries(memory)) {
      report.sampledMaxBytes[key] = Math.max(
        report.sampledMaxBytes[key] ?? 0,
        value,
      );
    }
    console.log(
      `${stage}: ${Object.entries(memory)
        .map(([key, value]) => `${key}=${(value / 1024 / 1024).toFixed(1)} MiB`)
        .join(", ")}`,
    );
  };
  try {
    if (values.probe) {
      const child = spawn(
        values.probe,
        [String(physicalWidth), String(physicalHeight), String(seconds)],
        {
          stdio: ["pipe", "pipe", "inherit"],
        },
      );
      // Convert spawn failures to an observed result immediately, avoiding unhandled rejections.
      const finished = new Promise<string | null>((resolve) => {
        child.once("error", (error) => resolve(error.message));
        child.once("exit", (code, signal) =>
          resolve(code === 0 ? null : `probe exited: ${code ?? signal}`),
        );
      });
      const launched = new Promise<string | null>((resolve) => {
        child.once("spawn", () => resolve(null));
        child.once("error", (error) => resolve(error.message));
      });
      const timeout = setTimeout(
        () => child.kill("SIGKILL"),
        (seconds + 120) * 1000,
      );
      try {
        const launchError = await launched;
        if (launchError) throw new Error(launchError);
        const lines = createInterface({ input: child.stdout });
        for await (const line of lines) {
          const message = JSON.parse(line) as {
            stage: string;
            details: unknown;
          };
          if (typeof message.stage !== "string" || !child.pid)
            throw new Error("invalid probe checkpoint");
          record(child.pid, message.stage, message.details);
          child.stdin.write("continue\n");
        }
        const failure = await finished;
        if (failure) throw new Error(failure);
        if (samples.at(-1)?.stage !== "pixels_verified")
          throw new Error("probe did not verify pixels");
      } finally {
        clearTimeout(timeout);
        if (child.exitCode === null && child.signalCode === null)
          child.kill("SIGKILL");
        await finished;
      }
    } else {
      const deadline = performance.now() + seconds * 1000;
      do {
        record(pid!, "application");
        const remaining = deadline - performance.now();
        if (remaining <= 0) break;
        await Bun.sleep(Math.min(interval * 1000, remaining));
      } while (performance.now() < deadline);
    }
    report.complete = true;
  } catch (error) {
    report.error = String(error);
    throw error;
  } finally {
    await Bun.write(values.out, `${JSON.stringify(report, null, 2)}\n`);
  }
}

if (import.meta.main) await main();
