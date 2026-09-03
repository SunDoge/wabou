import { mount } from "@wabou/core/renderer";
import { ColorThemeProvider, useWindow } from "@wabou/core";
import { createComponent } from "solid-js";
//#region src/layout-fixtures.ts
function normalizeFixture(entry) {
	return typeof entry === "function" ? { render: entry } : entry;
}
/**
* Give component fixtures the same theme, viewport, and bounded root without
* repeating an application shell in every test entry.
*/
function defineComponentFixtures(fixtures, options = {}) {
	return Object.fromEntries(Object.entries(fixtures).map(([id, entry]) => {
		const fixture = normalizeFixture(entry);
		return [id, {
			width: fixture.width ?? options.width,
			height: fixture.height ?? options.height,
			scaleFactor: fixture.scaleFactor ?? options.scaleFactor,
			waitMs: fixture.waitMs ?? options.waitMs,
			render: () => {
				const render = () => createComponent(fixture.render, {});
				return options.wrap?.(render) ?? render();
			}
		}];
	}));
}
/**
* Expose named component fixtures to `wabou layout --batch`.
*
* The registry is bundled once by Vite. Every native mount disposes the
* preceding Solid owner before rendering the next case, so effects and event
* handlers retain ordinary Solid cleanup semantics while QuickJS is reused.
*/
function defineLayoutFixtures(fixtures, options = {}) {
	const entries = Object.entries(fixtures);
	if (entries.length === 0) throw new Error("defineLayoutFixtures requires at least one fixture");
	const registry = /* @__PURE__ */ new Map();
	for (const [id, entry] of entries) {
		if (id.length === 0) throw new Error("layout fixture id must not be empty");
		const fixture = normalizeFixture(entry);
		if (typeof fixture.render !== "function") throw new TypeError(`layout fixture \`${id}\` must be a function`);
		registry.set(id, fixture);
	}
	let dispose;
	globalThis.__wabou_layout_fixture_ids = () => JSON.stringify([...registry.keys()]);
	globalThis.__wabou_layout_fixture_cases = () => JSON.stringify([...registry].map(([id, { width, height, scaleFactor, waitMs }]) => ({
		id,
		width,
		height,
		scaleFactor,
		waitMs
	})));
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
				}
			});
		});
	};
}
//#endregion
export { defineComponentFixtures, defineLayoutFixtures };

//# sourceMappingURL=layout-fixtures.mjs.map