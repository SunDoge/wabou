import { LayoutSnapshot } from "./layout.mjs";
//#region src/layout-node.d.ts
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
  /** Executable and any fixed prefix arguments. Defaults to `["wabou"]`. */
  readonly command?: readonly string[];
}
declare function layoutCommandArgs(options: RenderAppLayoutOptions): readonly string[];
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
  readonly checks?: readonly ("visible-overflow" | "sibling-collision")[];
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
  /** Executable and any fixed prefix arguments. Defaults to `["wabou"]`. */
  readonly command?: readonly string[];
}
interface LayoutFixtureResult {
  readonly id: string;
  readonly snapshot: LayoutSnapshot;
}
interface LayoutFixtureReport {
  readonly version: 1;
  readonly cases: readonly LayoutFixtureResult[];
}
declare function parseLayoutFixtureReport(value: unknown): LayoutFixtureReport;
declare function validateLayoutFixtureReport(report: LayoutFixtureReport, fixtures: readonly LayoutFixtureCase[]): Promise<void>;
/** Build one fixture bundle and evaluate every case in one QuickJS runtime. */
declare function renderLayoutFixtures(options: RenderLayoutFixturesOptions): Promise<LayoutFixtureReport>;
declare function renderAppLayout(options: RenderAppLayoutOptions): Promise<LayoutSnapshot>;
//#endregion
export { LayoutFixtureCase, LayoutFixtureReport, LayoutFixtureResult, RenderAppLayoutOptions, RenderLayoutFixturesOptions, layoutCommandArgs, parseLayoutFixtureReport, renderAppLayout, renderLayoutFixtures, validateLayoutFixtureReport };
//# sourceMappingURL=layout-node.d.mts.map