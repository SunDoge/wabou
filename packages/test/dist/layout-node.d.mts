import { LayoutSnapshot } from "./layout.mjs";
//#region src/layout-node.d.ts
type LayoutFixtureCheck = "visible-overflow" | "sibling-collision" | "text-collision" | "visual-quality";
interface RenderAppLayoutOptions {
  readonly app: string;
  readonly out: string;
  readonly batch?: string;
  /** Mount one named application layout fixture before evaluating a probe. */
  readonly fixture?: string;
  readonly width?: number;
  readonly height?: number;
  readonly scaleFactor?: number;
  /** Native system color scheme exposed to the fixture application. */
  readonly colorScheme?: "light" | "dark";
  readonly mode?: string;
  readonly skipBuild?: boolean;
  readonly waitMs?: number;
  /** JavaScript evaluated after the initial GPUI projection checkpoint. */
  readonly probe?: string;
  /** Boot the application's Rust host so custom capabilities are available. */
  readonly withHost?: boolean;
  /** Executable and any fixed prefix arguments. Defaults to `["wabou"]`. */
  readonly command?: readonly string[];
}
declare function layoutCommandArgs(options: RenderAppLayoutOptions): readonly string[];
interface ProjectionBoundaryProbeDelta {
  readonly root: {
    readonly lo: number;
    readonly hi: number;
  };
  readonly label?: string;
  readonly structureDelta: number;
  readonly layoutDelta: number;
  readonly paintDelta: number;
  readonly materializationDelta: number;
  readonly ownedNodes: number;
}
declare function projectionBoundaryProbe(report: ProjectionProbeReport, label: string): ProjectionBoundaryProbeDelta;
interface ProjectionProbeReport {
  readonly protocolRevisionDelta: number;
  readonly boundaries: readonly ProjectionBoundaryProbeDelta[];
}
declare function probeAppProjection(options: RenderAppLayoutOptions & {
  readonly probe: string;
}): Promise<ProjectionProbeReport>;
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
  /** Scene checks are opt-in because overlays and muted decoration may be intentional. */
  readonly checks?: readonly LayoutFixtureCheck[];
  /** Run application-specific geometry or reactive-state assertions. */
  readonly assert?: (snapshot: LayoutSnapshot) => void | Promise<void>;
}
interface RenderLayoutFixturesOptions {
  readonly app: string;
  /** Use `"all"` to run every fixture registered by the compiled entry. */
  readonly cases: readonly LayoutFixtureCase[] | "all";
  readonly mode?: string;
  /** Native system color scheme exposed to every fixture in this batch. */
  readonly colorScheme?: "light" | "dark";
  readonly skipBuild?: boolean;
  readonly waitMs?: number;
  /** Checks applied to every auto-discovered fixture. */
  readonly checks?: readonly LayoutFixtureCheck[];
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
export { LayoutFixtureCase, LayoutFixtureCheck, LayoutFixtureReport, LayoutFixtureResult, ProjectionBoundaryProbeDelta, ProjectionProbeReport, RenderAppLayoutOptions, RenderLayoutFixturesOptions, layoutCommandArgs, parseLayoutFixtureReport, probeAppProjection, projectionBoundaryProbe, reactiveRuntimeDiagnostic, renderAppLayout, renderLayoutFixtures, validateLayoutFixtureReport };
//# sourceMappingURL=layout-node.d.mts.map