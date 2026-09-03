import { mkdir } from "node:fs/promises";
import { dirname, resolve } from "node:path";

const root = resolve(import.meta.dir, "..");
const app = "apps/pi-agent";
const artifacts = resolve(root, "target/wabou-test/pi-agent");
const fixture = resolve(
  root,
  `target/debug/examples/pi-agent-fixture${process.platform === "win32" ? ".exe" : ""}`,
);

type Mode = "deterministic" | "capture" | "native" | "real";

async function run(command: string[], env = process.env): Promise<void> {
  console.log(`[pi-agent] ${command.join(" ")}`);
  const child = Bun.spawn(command, {
    cwd: root,
    env,
    stdin: "inherit",
    stdout: "inherit",
    stderr: "inherit",
  });
  const exitCode = await child.exited;
  if (exitCode !== 0) process.exit(exitCode);
}

async function buildFixture(): Promise<Record<string, string | undefined>> {
  await run([
    "cargo",
    "build",
    "-p",
    "pi-agent-wabou",
    "--example",
    "pi-agent-fixture",
  ]);
  return { ...process.env, WABOU_PI_BIN: fixture };
}

async function deterministic(): Promise<void> {
  const env = await buildFixture();
  await run(
    [
      "cargo",
      "run",
      "-p",
      "wabou-cli",
      "--",
      "test",
      app,
      "--artifacts",
      resolve(artifacts, "deterministic"),
      "--env",
      `WABOU_PI_BIN=${fixture}`,
      "--env",
      "WABOU_FAKE_PI_STARTUP_DELAY_MS=100",
    ],
    env,
  );
}

async function capture(): Promise<void> {
  const env = await buildFixture();
  const output = resolve(artifacts, "screenshots/conversation.png");
  const snapshot = resolve(artifacts, "semantics/conversation.json");
  await mkdir(dirname(output), { recursive: true });
  await mkdir(dirname(snapshot), { recursive: true });
  await run(
    [
      "cargo",
      "run",
      "-p",
      "wabou-cli",
      "--",
      "render",
      app,
      "--with-host",
      "--scenario",
      "apps/pi-agent/captures/conversation.behavior.ts",
      "--out",
      output,
      "--snapshot",
      snapshot,
      "--width",
      "1440",
      "--height",
      "900",
      "--wait-ms",
      "300",
    ],
    env,
  );
}

async function native(): Promise<void> {
  const env = await buildFixture();
  await run(
    [
      "cargo",
      "run",
      "-p",
      "wabou-cli",
      "--",
      "test",
      "apps/pi-agent/captures/conversation.behavior.ts",
      "--app",
      app,
      "--native",
      "--artifacts",
      resolve(artifacts, "native"),
      "--env",
      `WABOU_PI_BIN=${fixture}`,
    ],
    env,
  );
}

async function real(): Promise<void> {
  if (process.env.WABOU_RUN_REAL_PI !== "1") {
    throw new Error(
      "real Pi testing may consume provider credits; rerun with WABOU_RUN_REAL_PI=1",
    );
  }
  const env = { ...process.env };
  delete env.WABOU_PI_BIN;
  await run(
    [
      "cargo",
      "test",
      "-p",
      "pi-agent-wabou",
      "real_pi_",
      "--",
      "--ignored",
      "--nocapture",
    ],
    env,
  );
}

const mode = (process.argv[2] ?? "all") as Mode | "all";
switch (mode) {
  case "all":
    await deterministic();
    await capture();
    break;
  case "deterministic":
    await deterministic();
    break;
  case "capture":
    await capture();
    break;
  case "native":
    await native();
    break;
  case "real":
    await real();
    break;
  default:
    throw new Error(`unknown Pi Agent test mode: ${mode}`);
}
