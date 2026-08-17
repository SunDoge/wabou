import { fileURLToPath } from "node:url";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { wabouStylePlugin } from "@wabou/style-compiler";
import { defineConfig, mergeConfig } from "vite";
import solid from "vite-plugin-solid";
import { parse } from "smol-toml";
//#region src/index.ts
function disableSolidDependencyOptimizer() {
	return {
		name: "wabou-disable-solid-deps-optimizer",
		enforce: "post",
		configResolved(config) {
			if (config.command === "serve") {
				config.optimizeDeps.noDiscovery = true;
				config.optimizeDeps.include = [];
			}
		}
	};
}
/** Plugins required for Solid to target Wabou instead of the browser DOM. */
function wabouPlugins(root = process.cwd(), theme, ignoreClasses) {
	return [
		wabouStylePlugin({
			root,
			colorThemes: theme,
			ignoreClasses
		}),
		...solid({ solid: {
			generate: "universal",
			moduleName: "@wabou/solid-renderer"
		} }),
		disableSolidDependencyOptimizer()
	];
}
/** Define the complete conventional Vite configuration for a Wabou app. */
function defineWabouConfig(options) {
	if (typeof options === "function") return defineConfig((environment) => resolveWabouConfig(options(environment), environment));
	return defineConfig((environment) => resolveWabouConfig(options, environment));
}
function resolveWabouConfig(options, environment) {
	const root = options.root ?? process.cwd();
	const outDir = options.outDir ?? manifestOutDir(root);
	const renderer = fileURLToPath(import.meta.resolve("@wabou/solid-renderer"));
	const defaults = {
		define: { "process.env.NODE_ENV": JSON.stringify(process.env.NODE_ENV ?? (environment.command === "serve" ? "development" : "production")) },
		plugins: wabouPlugins(root, options.theme, options.ignoreClasses),
		resolve: { alias: {
			"@wabou/solid-renderer": renderer,
			"solid-js/web": renderer
		} },
		build: {
			sourcemap: true,
			lib: {
				entry: options.entry ?? "ui/index.tsx",
				formats: ["iife"],
				name: options.globalName ?? "WabouApp",
				fileName: () => "bundle.js"
			},
			rollupOptions: { output: {
				inlineDynamicImports: true,
				assetFileNames: "bundle.[ext]"
			} },
			cssCodeSplit: false,
			outDir,
			emptyOutDir: true,
			minify: false
		}
	};
	return mergeConfig(defaults, options.vite ?? {});
}
function manifestOutDir(root) {
	const path = resolve(root, "wabou.toml");
	let manifest;
	try {
		manifest = parse(readFileSync(path, "utf8"));
	} catch (error) {
		throw new Error(`cannot read Wabou build output from ${path}: ${error}`);
	}
	const outDir = manifest.build?.["out-dir"];
	if (typeof outDir !== "string" || outDir.trim() === "") throw new Error(`${path} must declare a non-empty build.out-dir`);
	return outDir;
}
//#endregion
export { defineWabouConfig, wabouPlugins };

//# sourceMappingURL=index.mjs.map