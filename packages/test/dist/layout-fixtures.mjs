import { mount } from "@wabou/core/renderer";
//#region src/layout-fixtures.ts
/**
* Expose named component fixtures to `wabou layout --batch`.
*
* The registry is bundled once by Vite. Every native mount disposes the
* preceding Solid owner before rendering the next case, so effects and event
* handlers retain ordinary Solid cleanup semantics while QuickJS is reused.
*/
function defineLayoutFixtures(fixtures) {
	const entries = Object.entries(fixtures);
	if (entries.length === 0) throw new Error("defineLayoutFixtures requires at least one fixture");
	const registry = /* @__PURE__ */ new Map();
	for (const [id, fixture] of entries) {
		if (id.length === 0) throw new Error("layout fixture id must not be empty");
		if (typeof fixture !== "function") throw new TypeError(`layout fixture \`${id}\` must be a function`);
		registry.set(id, fixture);
	}
	let dispose;
	globalThis.__wabou_layout_fixture_ids = () => JSON.stringify([...registry.keys()]);
	globalThis.__wabou_layout_fixture_mount = (id) => {
		const fixture = registry.get(id);
		if (!fixture) throw new Error(`unknown Wabou layout fixture \`${id}\``);
		dispose?.();
		dispose = mount(fixture);
	};
}
//#endregion
export { defineLayoutFixtures };

//# sourceMappingURL=layout-fixtures.mjs.map