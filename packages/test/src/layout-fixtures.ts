import { ColorThemeProvider, useWindow } from "@wabou/core";
import { mount } from "@wabou/core/renderer";
import { createComponent, type JSX } from "solid-js";

export type LayoutFixture = () => JSX.Element;
export interface LayoutFixtureDefinition {
  readonly render: LayoutFixture;
  readonly width?: number;
  readonly height?: number;
  readonly scaleFactor?: number;
  readonly waitMs?: number;
}
export type LayoutFixtureEntry = LayoutFixture | LayoutFixtureDefinition;
export type LayoutFixtureRegistry = Readonly<
  Record<string, LayoutFixtureEntry>
>;

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
  // Called by the native layout-batch runner after the fixture bundle boots.
  // This is deliberately not part of the application host API.
  var __wabou_layout_fixture_mount: ((id: string) => void) | undefined;
  var __wabou_layout_fixture_ids: (() => string) | undefined;
  var __wabou_layout_fixture_cases: (() => string) | undefined;
}

function normalizeFixture(entry: LayoutFixtureEntry): LayoutFixtureDefinition {
  return typeof entry === "function" ? { render: entry } : entry;
}

/**
 * Give component fixtures the same theme, viewport, and bounded root without
 * repeating an application shell in every test entry.
 */
export function defineComponentFixtures(
  fixtures: LayoutFixtureRegistry,
  options: ComponentFixtureOptions = {},
): LayoutFixtureRegistry {
  return Object.fromEntries(
    Object.entries(fixtures).map(([id, entry]) => {
      const fixture = normalizeFixture(entry);
      return [
        id,
        {
          width: fixture.width ?? options.width,
          height: fixture.height ?? options.height,
          scaleFactor: fixture.scaleFactor ?? options.scaleFactor,
          waitMs: fixture.waitMs ?? options.waitMs,
          render: () => {
            const render = () => createComponent(fixture.render, {});
            return options.wrap?.(render) ?? render();
          },
        },
      ];
    }),
  );
}

/**
 * Expose named component fixtures to `wabou layout --batch`.
 *
 * The registry is bundled once by Vite. Every native mount disposes the
 * preceding Solid owner before rendering the next case, so effects and event
 * handlers retain ordinary Solid cleanup semantics while QuickJS is reused.
 */
export function defineLayoutFixtures(
  fixtures: LayoutFixtureRegistry,
  options: LayoutFixtureOptions = {},
): void {
  const entries = Object.entries(fixtures);
  if (entries.length === 0)
    throw new Error("defineLayoutFixtures requires at least one fixture");
  const registry = new Map<string, LayoutFixtureDefinition>();
  for (const [id, entry] of entries) {
    if (id.length === 0) throw new Error("layout fixture id must not be empty");
    const fixture = normalizeFixture(entry);
    if (typeof fixture.render !== "function")
      throw new TypeError(`layout fixture \`${id}\` must be a function`);
    registry.set(id, fixture);
  }

  let dispose: (() => void) | undefined;
  globalThis.__wabou_layout_fixture_ids = () =>
    JSON.stringify([...registry.keys()]);
  globalThis.__wabou_layout_fixture_cases = () =>
    JSON.stringify(
      [...registry].map(([id, { width, height, scaleFactor, waitMs }]) => ({
        id,
        width,
        height,
        scaleFactor,
        waitMs,
      })),
    );
  globalThis.__wabou_layout_fixture_mount = (id) => {
    const fixture = registry.get(id);
    if (!fixture) throw new Error(`unknown Wabou layout fixture \`${id}\``);
    dispose?.();
    dispose = mount(() => {
      const colorTheme = options.colorTheme;
      if (colorTheme === false) return createComponent(fixture.render, {});
      const window = useWindow();
      return createComponent(ColorThemeProvider, {
        get theme() {
          const scheme = window.colorScheme();
          return colorTheme?.(scheme) ?? scheme;
        },
        transition: false,
        get children() {
          return createComponent(fixture.render, {});
        },
      });
    });
  };
}
