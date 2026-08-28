import { LayoutSnapshot } from "./layout.mjs";
//#region src/layout-node.d.ts
type LayoutGeometryCheck = "visible-overflow" | "sibling-collision" | "text-collision";
interface RenderAppLayoutOptions {
  readonly app: string;
  readonly out: string;
  readonly batch?: string;
  readonly width?: number;
  readonly height?: number;
  readonly scaleFactor?: number;
  readonly mode?: string;
  readonly skipBuild?: boolean;
  readonly waitMs?: number;
  /** Boot the application's Rust host so custom capabilities are available. */
  readonly withHost?: boolean;
  /** Executable and any fixed prefix arguments. Defaults to `["wabou"]`. */
  readonly command?: readonly string[];
}
declare function layoutCommandArgs(options: RenderAppLayoutOptions): readonly string[];
/** Return the first Solid runtime diagnostic that makes a layout run invalid. */
declare function reactiveRuntimeDiagnostic(output: string): string | undefined;
interface LayoutFixtureCase {
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
interface RenderLayoutFixturesOptions {
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
interface LayoutFixtureResult {
  readonly id: string;
  readonly durationMs: number;
  readonly snapshot: LayoutSnapshot;
}
interface LayoutFixtureReport {
  readonly version: 1;
  readonly totalDurationMs: number;
  readonly cases: readonly LayoutFixtureResult[];
}
declare function parseLayoutFixtureReport(value: unknown): LayoutFixtureReport;
declare function validateLayoutFixtureReport(report: LayoutFixtureReport, fixtures: readonly LayoutFixtureCase[]): Promise<void>;
/** Build one fixture bundle and evaluate every case in one QuickJS runtime. */
declare function renderLayoutFixtures(options: RenderLayoutFixturesOptions): Promise<LayoutFixtureReport>;
declare function renderAppLayout(options: RenderAppLayoutOptions): Promise<LayoutSnapshot>;
//#endregion
export { LayoutFixtureCase, LayoutFixtureReport, LayoutFixtureResult, LayoutGeometryCheck, RenderAppLayoutOptions, RenderLayoutFixturesOptions, layoutCommandArgs, parseLayoutFixtureReport, reactiveRuntimeDiagnostic, renderAppLayout, renderLayoutFixtures, validateLayoutFixtureReport };
//# sourceMappingURL=layout-node.d.mts.map