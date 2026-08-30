import { resolve } from "node:path";

export type LayoutTestApp = "gallery" | "pi-agent";

export interface LayoutTestSelection {
  apps: readonly LayoutTestApp[];
  filters: readonly string[];
}

const ALL_APPS: readonly LayoutTestApp[] = ["gallery", "pi-agent"];
const PI_AGENT_FIXTURE_PREFIXES = [
  "conversation/",
  "settings/",
  "shell/",
  "workspace/",
] as const;

function inferApps(filters: readonly string[]): readonly LayoutTestApp[] {
  if (filters.length === 0) return ALL_APPS;
  const applications = new Set<LayoutTestApp>();
  for (const filter of filters) {
    applications.add(
      PI_AGENT_FIXTURE_PREFIXES.some((prefix) => filter.startsWith(prefix))
        ? "pi-agent"
        : "gallery",
    );
  }
  return ALL_APPS.filter((app) => applications.has(app));
}

function fixtureBelongsToApp(filter: string, app: LayoutTestApp): boolean {
  const piAgentFixture = PI_AGENT_FIXTURE_PREFIXES.some((prefix) =>
    filter.startsWith(prefix),
  );
  return piAgentFixture ? app === "pi-agent" : app === "gallery";
}

function parseApp(value: string | undefined): LayoutTestApp {
  if (value === "gallery" || value === "pi-agent") return value;
  throw new Error(
    `unknown layout app ${JSON.stringify(value)}; expected gallery or pi-agent`,
  );
}

export function parseLayoutTestArgs(
  args: readonly string[],
): LayoutTestSelection {
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
  return { apps: app ? [app] : inferApps(filters), filters };
}

async function runLayoutTests(selection: LayoutTestSelection): Promise<void> {
  const root = resolve(import.meta.dir, "..");
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
