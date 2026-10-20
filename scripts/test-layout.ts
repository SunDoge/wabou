import { resolve } from "node:path";

export type LayoutTestApp = "gallery" | "launcher" | "meow-starter" | "reader";

export interface LayoutTestSelection {
  apps: readonly LayoutTestApp[];
  filters: readonly string[];
  /** Reuse an existing fixture bundle. Intended only for unchanged sources. */
  skipBuild: boolean;
  /** Reuse an already built renderer and package artifacts. */
  reuseRenderer: boolean;
}

const ALL_APPS: readonly LayoutTestApp[] = [
  "gallery",
  "launcher",
  "meow-starter",
  "reader",
];

function inferApps(filters: readonly string[]): readonly LayoutTestApp[] {
  if (filters.length === 0) return ALL_APPS;
  const applications = new Set<LayoutTestApp>();
  for (const filter of filters) {
    if (filter.startsWith("launcher/")) applications.add("launcher");
    else if (filter.startsWith("reader/")) applications.add("reader");
    else if (filter.startsWith("meow-starter/"))
      applications.add("meow-starter");
    else applications.add("gallery");
  }
  return ALL_APPS.filter((app) => applications.has(app));
}

function fixtureBelongsToApp(filter: string, app: LayoutTestApp): boolean {
  if (filter.startsWith("launcher/")) return app === "launcher";
  if (filter.startsWith("reader/")) return app === "reader";
  if (filter.startsWith("meow-starter/")) return app === "meow-starter";
  return app === "gallery";
}

function parseApp(value: string | undefined): LayoutTestApp {
  if (
    value === "gallery" ||
    value === "launcher" ||
    value === "meow-starter" ||
    value === "reader"
  )
    return value;
  throw new Error(
    `unknown layout app ${JSON.stringify(value)}; expected gallery, launcher, meow-starter, or reader`,
  );
}

export function parseLayoutTestArgs(
  args: readonly string[],
): LayoutTestSelection {
  let app: LayoutTestApp | undefined;
  let skipBuild = false;
  let reuseRenderer = false;
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
    if (argument === "--reuse-renderer") {
      reuseRenderer = true;
      continue;
    }
    if (argument === "--skip-build") {
      skipBuild = true;
      continue;
    }
    if (argument.startsWith("-")) {
      throw new Error(`unknown layout test option: ${argument}`);
    }
    filters.push(argument);
  }
  return {
    apps: app ? [app] : inferApps(filters),
    filters,
    skipBuild,
    reuseRenderer,
  };
}

async function runLayoutTests(selection: LayoutTestSelection): Promise<void> {
  const root = resolve(import.meta.dir, "..");
  if (!selection.reuseRenderer) {
    for (const command of [
      [process.execPath, "run", "packages:build"],
      [
        "cargo",
        "build",
        "--release",
        "-p",
        "wabou-cli",
        "--features",
        "embedded-renderer",
        "--bin",
        "wabou-render",
      ],
    ]) {
      const child = Bun.spawn(command, {
        cwd: root,
        stdin: "inherit",
        stdout: "inherit",
        stderr: "inherit",
      });
      const exitCode = await child.exited;
      if (exitCode !== 0) process.exit(exitCode);
    }
  }
  for (const app of selection.apps) {
    const filters =
      selection.apps.length === 1
        ? selection.filters
        : selection.filters.filter((filter) =>
            fixtureBelongsToApp(filter, app),
          );
    const entry = resolve(root, "apps", app, "tests/layout.ts");
    const command = [
      process.execPath,
      "--conditions=wabou-source",
      "run",
      entry,
      ...filters,
    ];
    console.log(
      `[layout] ${app}${
        filters.length > 0 ? `: ${filters.join(", ")}` : ": all fixtures"
      }`,
    );
    const child = Bun.spawn(command, {
      cwd: root,
      env: {
        ...process.env,
        WABOU_LAYOUT_COMMAND:
          process.env.WABOU_LAYOUT_COMMAND ??
          resolve(root, "target/release/wabou-render"),
        WABOU_LAYOUT_SKIP_BUILD: selection.skipBuild ? "1" : "0",
      },
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
