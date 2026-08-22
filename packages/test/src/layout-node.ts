import { spawn } from "node:child_process";
import { mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

import {
  assertNoLayoutDiagnostics,
  type LayoutSnapshot,
  parseLayoutSnapshot,
  siblingCollisionDiagnostics,
  styleDiagnostics,
  textCollisionDiagnostics,
  visibleOverflowDiagnostics,
} from "./layout";

export type LayoutGeometryCheck =
  | "visible-overflow"
  | "sibling-collision"
  | "text-collision";

export interface RenderAppLayoutOptions {
  readonly app: string;
  readonly out: string;
  readonly batch?: string;
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
  if (options.batch !== undefined) args.push("--batch", options.batch);
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

export interface LayoutFixtureCase {
  readonly id: string;
  readonly width?: number;
  readonly height?: number;
  readonly scaleFactor?: number;
  /** Additional time for timers, promises, or finite motion to settle. */
  readonly waitMs?: number;
  /** Style parser rejections fail a fixture by default. */
  readonly allowStyleDiagnostics?: boolean;
  /** Geometry checks are opt-in because overlays may intentionally overlap. */
  readonly checks?: readonly LayoutGeometryCheck[];
  /** Run application-specific geometry or reactive-state assertions. */
  readonly assert?: (snapshot: LayoutSnapshot) => void | Promise<void>;
}

export interface RenderLayoutFixturesOptions {
  readonly app: string;
  /** Use `"all"` to run every fixture registered by the compiled entry. */
  readonly cases: readonly LayoutFixtureCase[] | "all";
  readonly mode?: string;
  readonly skipBuild?: boolean;
  readonly waitMs?: number;
  /** Checks applied to every auto-discovered fixture. */
  readonly checks?: readonly LayoutGeometryCheck[];
  /** Per-fixture exceptions or assertions without repeating the full case list. */
  readonly overrides?: Readonly<Record<string, Omit<LayoutFixtureCase, "id">>>;
  /** Executable and any fixed prefix arguments. Defaults to `["wabou"]`. */
  readonly command?: readonly string[];
}

export interface LayoutFixtureResult {
  readonly id: string;
  readonly durationMs: number;
  readonly snapshot: LayoutSnapshot;
}

export interface LayoutFixtureReport {
  readonly version: 1;
  readonly totalDurationMs: number;
  readonly cases: readonly LayoutFixtureResult[];
}

export function parseLayoutFixtureReport(value: unknown): LayoutFixtureReport {
  if (typeof value !== "object" || value === null)
    throw new Error("Wabou layout fixture report must be an object");
  const raw = value as {
    version?: unknown;
    totalDurationMs?: unknown;
    cases?: unknown;
  };
  if (
    raw.version !== 1 ||
    !Number.isFinite(raw.totalDurationMs) ||
    (raw.totalDurationMs as number) < 0 ||
    !Array.isArray(raw.cases)
  )
    throw new Error("invalid Wabou layout fixture report");
  return {
    version: 1,
    totalDurationMs: raw.totalDurationMs as number,
    cases: raw.cases.map((entry, index) => {
      if (
        typeof entry !== "object" ||
        entry === null ||
        typeof (entry as { id?: unknown }).id !== "string" ||
        !Number.isFinite((entry as { durationMs?: unknown }).durationMs) ||
        ((entry as { durationMs: number }).durationMs as number) < 0 ||
        !("snapshot" in entry)
      )
        throw new Error(
          `invalid Wabou layout fixture result at index ${index}`,
        );
      return {
        id: (entry as { id: string }).id,
        durationMs: (entry as { durationMs: number }).durationMs,
        snapshot: parseLayoutSnapshot(
          (entry as { snapshot: unknown }).snapshot,
        ),
      };
    }),
  };
}

export async function validateLayoutFixtureReport(
  report: LayoutFixtureReport,
  fixtures: readonly LayoutFixtureCase[],
): Promise<void> {
  for (const result of report.cases) {
    const fixture = fixtures.find((entry) => entry.id === result.id);
    if (!fixture)
      throw new Error(
        `unexpected Wabou layout fixture result \`${result.id}\``,
      );
    try {
      if (!fixture.allowStyleDiagnostics)
        assertNoLayoutDiagnostics(styleDiagnostics(result.snapshot));
      for (const check of fixture.checks ?? []) {
        assertNoLayoutDiagnostics(
          check === "visible-overflow"
            ? visibleOverflowDiagnostics(result.snapshot)
            : check === "sibling-collision"
              ? siblingCollisionDiagnostics(result.snapshot)
              : textCollisionDiagnostics(result.snapshot),
        );
      }
      await fixture.assert?.(result.snapshot);
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      throw new Error(`layout fixture \`${result.id}\` failed: ${message}`, {
        cause: error,
      });
    }
  }
}

/** Build one fixture bundle and evaluate every case in one QuickJS runtime. */
export async function renderLayoutFixtures(
  options: RenderLayoutFixturesOptions,
): Promise<LayoutFixtureReport> {
  if (options.cases !== "all" && options.cases.length === 0)
    throw new Error("layout fixture batch must contain at least one case");
  const directory = await mkdtemp(join(tmpdir(), "wabou-layout-"));
  const manifest = join(directory, "manifest.json");
  const out = join(directory, "report.json");
  try {
    await writeFile(
      manifest,
      JSON.stringify(
        options.cases === "all"
          ? { version: 1, all: true }
          : {
              version: 1,
              cases: options.cases.map(
                ({ id, width, height, scaleFactor, waitMs }) => ({
                  id,
                  width,
                  height,
                  scaleFactor,
                  waitMs,
                }),
              ),
            },
      ),
      "utf8",
    );
    await runLayoutCommand({
      app: options.app,
      out,
      batch: manifest,
      mode: options.mode,
      skipBuild: options.skipBuild,
      waitMs: options.waitMs,
      command: options.command,
    });
    const report = parseLayoutFixtureReport(
      JSON.parse(await readFile(out, "utf8")),
    );
    const fixtures =
      options.cases === "all"
        ? report.cases.map(({ id }) => ({
            id,
            checks: options.checks,
            ...options.overrides?.[id],
          }))
        : options.cases;
    if (options.cases === "all" && options.overrides) {
      const discovered = new Set(report.cases.map(({ id }) => id));
      const unknown = Object.keys(options.overrides).filter(
        (id) => !discovered.has(id),
      );
      if (unknown.length > 0)
        throw new Error(
          `layout fixture overrides reference unknown fixtures: ${unknown.join(", ")}`,
        );
    }
    await validateLayoutFixtureReport(report, fixtures);
    return report;
  } finally {
    await rm(directory, { recursive: true, force: true });
  }
}

export async function renderAppLayout(
  options: RenderAppLayoutOptions,
): Promise<LayoutSnapshot> {
  await runLayoutCommand(options);
  return parseLayoutSnapshot(JSON.parse(await readFile(options.out, "utf8")));
}

async function runLayoutCommand(
  options: RenderAppLayoutOptions,
): Promise<void> {
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
}
