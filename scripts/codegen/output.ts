import { mkdir, readFile, writeFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";

export const ROOT = resolve(import.meta.dir, "../..");

export async function writeGenerated(
  path: string,
  contents: string,
): Promise<boolean> {
  let current: string | undefined;
  try {
    current = await readFile(path, "utf8");
  } catch (error) {
    if (
      !(error instanceof Error) ||
      !("code" in error) ||
      error.code !== "ENOENT"
    ) {
      throw error;
    }
  }
  if (current === contents) return false;
  await mkdir(dirname(path), { recursive: true });
  await writeFile(path, contents);
  return true;
}

export async function run(command: string[], label: string): Promise<void> {
  const process = Bun.spawn(command, {
    cwd: ROOT,
    stdout: "inherit",
    stderr: "inherit",
  });
  if ((await process.exited) !== 0) throw new Error(`${label} failed`);
}
