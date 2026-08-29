import { dirname } from "node:path";
import { fileURLToPath } from "node:url";
import solid from "@solidjs/vite-plugin";
import { mergeConfig } from "vite";
//#region src/test.ts
/** Configure Vitest to compile Wabou TSX through Solid's universal renderer. */
function defineWabouTestConfig(options = {}) {
	const core = fileURLToPath(import.meta.resolve("@wabou/core"));
	const renderer = fileURLToPath(import.meta.resolve("@wabou/core/renderer"));
	const testing = fileURLToPath(import.meta.resolve("@wabou/core/testing"));
	const solidEntry = `${dirname(fileURLToPath(import.meta.resolve("solid-js/package.json")))}/dist/solid.js`;
	return mergeConfig({
		plugins: solid({ solid: {
			generate: "universal",
			moduleName: "@wabou/core/renderer"
		} }),
		resolve: {
			conditions: ["browser", "wabou-source"],
			dedupe: ["solid-js"],
			alias: [
				{
					find: /^solid-js$/,
					replacement: solidEntry
				},
				{
					find: /^@wabou\/core$/,
					replacement: core
				},
				{
					find: /^@wabou\/core\/renderer$/,
					replacement: renderer
				},
				{
					find: /^@wabou\/core\/testing$/,
					replacement: testing
				},
				{
					find: /^solid-js\/web$/,
					replacement: renderer
				}
			]
		},
		test: { environment: "node" }
	}, options.vite ?? {});
}
//#endregion
export { defineWabouTestConfig };

//# sourceMappingURL=test.mjs.map