import { spawn } from "node:child_process";
import { readFile } from "node:fs/promises";

import { parseLayoutSnapshot, type LayoutSnapshot } from "./layout";

export interface RenderAppLayoutOptions {
  readonly app: string;
  readonly out: string;
  readonly width?: number;
  readonly height?: number;
  readonly scaleFactor?: number;
  readonly mode?: string;
  readonly skipBuild?: boolean;
  readonly waitMs?: number;
  /** Executable and any fixed prefix arguments. Defaults to `["wabou"]`. */
  readonly command?: readonly string[];
}

export function layoutCommandArgs(
  options: RenderAppLayoutOptions,
): readonly string[] {
  const args = ["layout", options.app, "--out", options.out];
  if (options.width !== undefined) args.push("--width", String(options.width));
  if (options.height !== undefined)
    args.push("--height", String(options.height));
  if (options.scaleFactor !== undefined)
    args.push("--scale-factor", String(options.scaleFactor));
  if (options.mode !== undefined) args.push("--mode", options.mode);
  if (options.skipBuild) args.push("--skip-build");
  if (options.waitMs !== undefined)
    args.push("--wait-ms", String(options.waitMs));
  return args;
}

export async function renderAppLayout(
  options: RenderAppLayoutOptions,
): Promise<LayoutSnapshot> {
  const command = options.command ?? ["wabou"];
  if (command.length === 0) throw new Error("layout command must not be empty");
  await new Promise<void>((resolve, reject) => {
    const child = spawn(
      command[0],
      [...command.slice(1), ...layoutCommandArgs(options)],
      {
        stdio: ["ignore", "inherit", "inherit"],
      },
    );
    child.once("error", reject);
    child.once("exit", (code, signal) => {
      if (code === 0) resolve();
      else
        reject(
          new Error(
            `layout command failed ${signal ? `with signal ${signal}` : `with exit status ${code}`}`,
          ),
        );
    });
  });
  return parseLayoutSnapshot(JSON.parse(await readFile(options.out, "utf8")));
}
