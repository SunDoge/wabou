import { mount } from "@wabou/core/renderer";
import type { JSX } from "solid-js";

export type LayoutFixture = () => JSX.Element;
export type LayoutFixtureRegistry = Readonly<Record<string, LayoutFixture>>;

declare global {
  // Called by the native layout-batch runner after the fixture bundle boots.
  // This is deliberately not part of the application host API.
  var __wabou_layout_fixture_mount:
    | ((id: string) => void)
    | undefined;
  var __wabou_layout_fixture_ids: (() => string) | undefined;
}

/**
 * Expose named component fixtures to `wabou layout --batch`.
 *
 * The registry is bundled once by Vite. Every native mount disposes the
 * preceding Solid owner before rendering the next case, so effects and event
 * handlers retain ordinary Solid cleanup semantics while QuickJS is reused.
 */
export function defineLayoutFixtures(fixtures: LayoutFixtureRegistry): void {
  const entries = Object.entries(fixtures);
  if (entries.length === 0)
    throw new Error("defineLayoutFixtures requires at least one fixture");
  const registry = new Map<string, LayoutFixture>();
  for (const [id, fixture] of entries) {
    if (id.length === 0) throw new Error("layout fixture id must not be empty");
    if (typeof fixture !== "function")
      throw new TypeError(`layout fixture \`${id}\` must be a function`);
    registry.set(id, fixture);
  }

  let dispose: (() => void) | undefined;
  globalThis.__wabou_layout_fixture_ids = () =>
    JSON.stringify([...registry.keys()]);
  globalThis.__wabou_layout_fixture_mount = (id) => {
    const fixture = registry.get(id);
    if (!fixture) throw new Error(`unknown Wabou layout fixture \`${id}\``);
    dispose?.();
    dispose = mount(fixture);
  };
}
