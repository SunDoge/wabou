import { JSX } from "solid-js";
//#region src/layout-fixtures.d.ts
type LayoutFixture = () => JSX.Element;
type LayoutFixtureRegistry = Readonly<Record<string, LayoutFixture>>;
declare global {
  var __wabou_layout_fixture_mount: ((id: string) => void) | undefined;
  var __wabou_layout_fixture_ids: (() => string) | undefined;
}
/**
 * Expose named component fixtures to `wabou layout --batch`.
 *
 * The registry is bundled once by Vite. Every native mount disposes the
 * preceding Solid owner before rendering the next case, so effects and event
 * handlers retain ordinary Solid cleanup semantics while QuickJS is reused.
 */
declare function defineLayoutFixtures(fixtures: LayoutFixtureRegistry): void;
//#endregion
export { LayoutFixture, LayoutFixtureRegistry, defineLayoutFixtures };
//# sourceMappingURL=layout-fixtures.d.mts.map