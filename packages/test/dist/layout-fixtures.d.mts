import { JSX } from "solid-js";
//#region src/layout-fixtures.d.ts
export type LayoutFixture = () => JSX.Element;
export interface LayoutFixtureDefinition {
  readonly render: LayoutFixture;
  readonly width?: number;
  readonly height?: number;
  readonly scaleFactor?: number;
  readonly waitMs?: number;
}
export type LayoutFixtureEntry = LayoutFixture | LayoutFixtureDefinition;
export type LayoutFixtureRegistry = Readonly<Record<string, LayoutFixtureEntry>>;
export interface ComponentFixtureOptions {
  readonly width?: number;
  readonly height?: number;
  readonly scaleFactor?: number;
  readonly waitMs?: number;
  /** Render a fixture inside providers or a shared bounded application shell. */
  readonly wrap?: (render: LayoutFixture) => JSX.Element;
}
export interface LayoutFixtureOptions {
  /**
   * Map the native system color scheme to an authored theme name. Fixtures
   * follow `light`/`dark` by default; pass `false` when the application owns a
   * fixed or custom theme inside its fixture wrapper.
   */
  readonly colorTheme?: false | ((scheme: "light" | "dark") => string);
}
declare global {
  var __wabou_layout_fixture_mount: ((id: string) => void) | undefined;
  var __wabou_layout_fixture_ids: (() => string) | undefined;
  var __wabou_layout_fixture_cases: (() => string) | undefined;
}
/**
 * Give component fixtures the same theme, viewport, and bounded root without
 * repeating an application shell in every test entry.
 */
export declare function defineComponentFixtures(fixtures: LayoutFixtureRegistry, options?: ComponentFixtureOptions): LayoutFixtureRegistry;
/**
 * Expose named component fixtures to `wabou layout --batch`.
 *
 * The registry is bundled once by Vite. Every native mount disposes the
 * preceding Solid owner before rendering the next case, so effects and event
 * handlers retain ordinary Solid cleanup semantics while QuickJS is reused.
 */
export declare function defineLayoutFixtures(fixtures: LayoutFixtureRegistry, options?: LayoutFixtureOptions): void;
//#endregion
//# sourceMappingURL=layout-fixtures.d.mts.map