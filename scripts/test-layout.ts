import { resolve } from "node:path";

export type LayoutTestApp = "gallery" | "pi-agent";

export interface LayoutTestSelection {
  apps: readonly LayoutTestApp[];
  filters: readonly string[];
}

const ALL_APPS: readonly LayoutTestApp[] = ["gallery", "pi-agent"];

function parseApp(value: string | undefined): LayoutTestApp {
  if (value === "gallery" || value === "pi-agent") return value;
  throw new Error(
    `unknown layout app ${JSON.stringify(value)}; expected gallery or pi-agent`,
  );
}

export function parseLayoutTestArgs(args: readonly string[]): LayoutTestSelection {
  let app: LayoutTestApp | undefined;
  const filters: string[] = [];
  for (let index = 0; index < args.length; index += 1) {
    const argument = args[index];
    if (argument === "--app") {
      app = parseApp(args[index + 1]);
      index += 1;
      continue;
    }
    if (argument.startsWith("--app=")) {
      app = parseApp(argument.slice("--app=".length));
      continue;
    }
    if (argument.startsWith("-")) {
      throw new Error(`unknown layout test option: ${argument}`);
    }
    filters.push(argument);
  }
  return { apps: app ? [app] : ALL_APPS, filters };
}

async function runLayoutTests(selection: LayoutTestSelection): Promise<void> {
  const root = resolve(import.meta.dir, "..");
  for (const app of selection.apps) {
    const entry = resolve(root, "apps", app, "tests/layout.ts");
    const command = [
      process.execPath,
      "--conditions=wabou-source",
      "run",
      entry,
      ...selection.filters,
    ];
    console.log(
      `[layout] ${app}${
        selection.filters.length > 0
          ? `: ${selection.filters.join(", ")}`
          : ": all fixtures"
      }`,
    );
    const child = Bun.spawn(command, {
      cwd: root,
      env: { ...process.env, WABOU_LAYOUT_SKIP_BUILD: "1" },
      stdin: "inherit",
      stdout: "inherit",
      stderr: "inherit",
    });
    const exitCode = await child.exited;
    if (exitCode !== 0) process.exit(exitCode);
  }
}

if (import.meta.main) {
  await runLayoutTests(parseLayoutTestArgs(process.argv.slice(2)));
}
