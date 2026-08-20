import { mkdir, readFile } from "node:fs/promises";
import { basename, dirname, relative, resolve } from "node:path";

const root = resolve(import.meta.dir, "..");

interface CaptureViewport {
  width: number;
  height: number;
  scaleFactor: number;
  waitMs: number;
}

interface CaptureConfig {
  defaults: CaptureViewport;
  overrides: Record<string, Partial<CaptureViewport>>;
}

export interface CaptureCase extends CaptureViewport {
  application: string;
  scenario: string;
  output: string;
}

export function captureCommand(
  capture: CaptureCase,
  skipBuild: boolean,
): string[] {
  const args = [
    "cargo",
    "run",
    "-p",
    "wabou-cli",
    "--",
    "render",
    capture.application,
    "--with-host",
    "--scenario",
    capture.scenario,
    "--out",
    capture.output,
    "--width",
    String(capture.width),
    "--height",
    String(capture.height),
    "--scale-factor",
    String(capture.scaleFactor),
    "--wait-ms",
    String(capture.waitMs),
  ];
  if (skipBuild) args.push("--skip-build");
  return args;
}

const fallbackViewport: CaptureViewport = {
  width: 1440,
  height: 900,
  scaleFactor: 1,
  waitMs: 250,
};
const viewportKeys = new Set(["width", "height", "scaleFactor", "waitMs"]);

function finiteNumber(
  value: unknown,
  name: string,
  minimum: number,
  maximum: number,
  integer = false,
): number {
  if (
    typeof value !== "number" ||
    !Number.isFinite(value) ||
    value < minimum ||
    value > maximum ||
    (integer && !Number.isInteger(value))
  ) {
    throw new Error(`${name} must be between ${minimum} and ${maximum}`);
  }
  return value;
}

function parseViewport(
  value: unknown,
  name: string,
  partial: boolean,
): Partial<CaptureViewport> {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`${name} must be an object`);
  }
  const record = value as Record<string, unknown>;
  for (const key of Object.keys(record)) {
    if (!viewportKeys.has(key))
      throw new Error(`${name}.${key} is unsupported`);
  }
  const viewport: Partial<CaptureViewport> = {};
  if (record.width !== undefined)
    viewport.width = finiteNumber(
      record.width,
      `${name}.width`,
      1,
      16_384,
      true,
    );
  if (record.height !== undefined)
    viewport.height = finiteNumber(
      record.height,
      `${name}.height`,
      1,
      16_384,
      true,
    );
  if (record.scaleFactor !== undefined)
    viewport.scaleFactor = finiteNumber(
      record.scaleFactor,
      `${name}.scaleFactor`,
      0.25,
      8,
    );
  if (record.waitMs !== undefined)
    viewport.waitMs = finiteNumber(
      record.waitMs,
      `${name}.waitMs`,
      0,
      60_000,
      true,
    );
  if (!partial) {
    return { ...fallbackViewport, ...viewport };
  }
  return viewport;
}

async function loadConfig(captureDirectory: string): Promise<CaptureConfig> {
  const path = resolve(captureDirectory, "config.json");
  if (!(await Bun.file(path).exists())) {
    return { defaults: fallbackViewport, overrides: {} };
  }
  const value: unknown = JSON.parse(await readFile(path, "utf8"));
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`${path} must contain an object`);
  }
  const record = value as Record<string, unknown>;
  for (const key of Object.keys(record)) {
    if (key !== "defaults" && key !== "overrides") {
      throw new Error(`${path} contains unsupported key ${key}`);
    }
  }
  const defaults = {
    ...fallbackViewport,
    ...parseViewport(record.defaults ?? {}, `${path}.defaults`, true),
  };
  const rawOverrides = record.overrides ?? {};
  if (
    rawOverrides === null ||
    typeof rawOverrides !== "object" ||
    Array.isArray(rawOverrides)
  ) {
    throw new Error(`${path}.overrides must be an object`);
  }
  const overrides = Object.fromEntries(
    Object.entries(rawOverrides as Record<string, unknown>).map(
      ([scenario, viewport]) => [
        scenario,
        parseViewport(viewport, `${path}.overrides.${scenario}`, true),
      ],
    ),
  );
  return { defaults, overrides };
}

export async function discoverCaptureCases(
  workspaceRoot = root,
): Promise<CaptureCase[]> {
  const glob = new Bun.Glob("apps/*/captures/**/*.ts");
  const sources: string[] = [];
  for await (const source of glob.scan({
    cwd: workspaceRoot,
    onlyFiles: true,
  })) {
    sources.push(source.replaceAll("\\", "/"));
  }
  sources.sort();

  const configs = new Map<string, Promise<CaptureConfig>>();
  const cases: CaptureCase[] = [];
  for (const scenario of sources) {
    const parts = scenario.split("/");
    const application = parts.slice(0, 2).join("/");
    const captureDirectory = resolve(workspaceRoot, application, "captures");
    const relativeScenario = parts.slice(3).join("/");
    let config = configs.get(application);
    if (!config) {
      config = loadConfig(captureDirectory);
      configs.set(application, config);
    }
    const resolved = await config;
    const override = resolved.overrides[relativeScenario] ?? {};
    cases.push({
      application,
      scenario,
      output: `target/wabou-captures/${basename(application)}/${relativeScenario.replace(/\.ts$/u, ".png")}`,
      ...resolved.defaults,
      ...override,
    });
  }

  for (const [application, configPromise] of configs) {
    const config = await configPromise;
    const known = new Set(
      cases
        .filter((capture) => capture.application === application)
        .map((capture) => relativeScenarioPath(capture.scenario)),
    );
    for (const override of Object.keys(config.overrides)) {
      if (!known.has(override)) {
        throw new Error(
          `${application}/captures/config.json overrides missing scenario ${override}`,
        );
      }
    }
  }
  return cases;
}

function relativeScenarioPath(scenario: string): string {
  return scenario.split("/captures/")[1] ?? scenario;
}

async function main(): Promise<void> {
  const captures = await discoverCaptureCases();
  if (captures.length === 0) {
    throw new Error("no apps/*/captures/**/*.ts scenarios were discovered");
  }
  if (process.argv.includes("--list")) {
    console.log(JSON.stringify(captures, null, 2));
    return;
  }

  const builtApplications = new Set<string>();
  for (const capture of captures) {
    const output = resolve(root, capture.output);
    await mkdir(dirname(output), { recursive: true });
    console.log(`[capture] rendering ${capture.scenario}`);
    const args = captureCommand(
      capture,
      builtApplications.has(capture.application),
    );
    const child = Bun.spawn(args, {
      cwd: root,
      stdin: "inherit",
      stdout: "inherit",
      stderr: "inherit",
    });
    const exitCode = await child.exited;
    if (exitCode !== 0) process.exit(exitCode);
    builtApplications.add(capture.application);
    if (!(await Bun.file(output).exists()) || Bun.file(output).size === 0) {
      throw new Error(`capture did not produce ${relative(root, output)}`);
    }
  }
  console.log(`verified ${captures.length} authored captures`);
}

if (import.meta.main) await main();
