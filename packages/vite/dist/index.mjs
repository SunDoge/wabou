import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { wabouStylePlugin } from "@wabou/style-compiler";
import { parse } from "smol-toml";
import { defineConfig, mergeConfig } from "vite";
import solid from "vite-plugin-solid";
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
	const outDir = process.env.WABOU_OUT_DIR ?? options.outDir ?? manifestOutDir(root);
	const sourceMap = process.env.WABOU_SOURCE_MAP;
	const sourcemap = sourceMap === "true" ? true : sourceMap === "false" ? false : manifestSourceMap(root) ?? process.env.WABOU_ENV_DEBUG === "true";
	const renderer = fileURLToPath(import.meta.resolve("@wabou/solid-renderer"));
	const defaults = {
		define: { "process.env.NODE_ENV": JSON.stringify(process.env.NODE_ENV ?? (environment.command === "serve" ? "development" : "production")) },
		plugins: wabouPlugins(root, options.theme, options.ignoreClasses),
		resolve: { alias: {
			"@wabou/solid-renderer": renderer,
			"solid-js/web": renderer
		} },
		build: {
			sourcemap,
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
	const config = mergeConfig(defaults, options.vite ?? {});
	if (process.env.WABOU_OUT_DIR !== void 0) config.build = {
		...config.build,
		outDir
	};
	if (process.env.WABOU_SOURCE_MAP !== void 0) config.build = {
		...config.build,
		sourcemap
	};
	return config;
}
function manifestOutDir(root) {
	const { manifest, path } = readManifest(root);
	const outDir = manifest.build?.["out-dir"];
	if (typeof outDir !== "string" || outDir.trim() === "") throw new Error(`${path} must declare a non-empty build.out-dir`);
	return outDir;
}
function manifestSourceMap(root) {
	if (!existsSync(resolve(root, "wabou.toml"))) return void 0;
	const { manifest, path } = readManifest(root);
	const sourceMap = manifest.build?.["source-map"];
	if (sourceMap === void 0) return void 0;
	if (typeof sourceMap !== "boolean") throw new Error(`${path} build.source-map must be true or false`);
	return sourceMap;
}
function readManifest(root) {
	const path = resolve(root, "wabou.toml");
	try {
		return {
			manifest: parse(readFileSync(path, "utf8")),
			path
		};
	} catch (error) {
		throw new Error(`cannot read Wabou build output from ${path}: ${error}`);
	}
}
//#endregion
export { defineWabouConfig, wabouPlugins };

//# sourceMappingURL=index.mjs.map