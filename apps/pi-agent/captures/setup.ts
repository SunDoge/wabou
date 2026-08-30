import { resolve } from "node:path";

/** Build and select the deterministic Pi RPC process used by authored captures. */
export default async function preparePiAgentCaptures(): Promise<
  Record<string, string>
> {
  const root = resolve(import.meta.dir, "../../..");
  const executable = resolve(
    root,
    `target/debug/examples/pi-agent-fixture${process.platform === "win32" ? ".exe" : ""}`,
  );
  const child = Bun.spawn(
    ["cargo", "build", "-p", "pi-agent-wabou", "--example", "pi-agent-fixture"],
    {
      cwd: root,
      stdin: "inherit",
      stdout: "inherit",
      stderr: "inherit",
    },
  );
  const exitCode = await child.exited;
  if (exitCode !== 0) {
    throw new Error(`pi-agent capture fixture build failed with ${exitCode}`);
  }
  return { WABOU_PI_BIN: executable };
}
