import { existsSync, readFileSync } from "node:fs";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { wabouStylePlugin } from "@wabou/style-compiler";
import MagicString from "magic-string";
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
function wabouPlugins(root = process.cwd(), theme, ignoreClasses, intl, entry = "ui/index.tsx") {
	return [
		wabouIntlPlugin(root, entry, intl ?? manifestIntl(root)),
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
		plugins: wabouPlugins(root, options.theme, options.ignoreClasses, options.intl, options.entry ?? "ui/index.tsx"),
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
const INTL_DATA_ID = "virtual:wabou-intl-data";
const RESOLVED_INTL_DATA_ID = `\0${INTL_DATA_ID}`;
function wabouIntlPlugin(root, entry, options) {
	const entryPath = resolve(root, entry);
	return {
		name: "wabou-intl-data",
		enforce: "pre",
		resolveId(id) {
			return id === INTL_DATA_ID ? RESOLVED_INTL_DATA_ID : void 0;
		},
		load(id) {
			if (id !== RESOLVED_INTL_DATA_ID) return void 0;
			return intlDataModule(options);
		},
		transform(code, id) {
			if (id.split("?", 1)[0] !== entryPath) return void 0;
			const transformed = new MagicString(code);
			transformed.prepend(`import ${JSON.stringify(INTL_DATA_ID)};\n`);
			return {
				code: transformed.toString(),
				map: transformed.generateMap({ hires: true })
			};
		}
	};
}
function intlDataModule(options) {
	const locales = [...new Set(options.locales ?? ["en", "zh"])];
	if (locales.length === 0) throw new Error("Wabou intl.locales cannot be empty");
	for (const locale of locales) if (!/^[A-Za-z0-9-]+$/.test(locale)) throw new Error(`invalid Wabou Intl locale module ${JSON.stringify(locale)}`);
	return `${[
		"@formatjs/intl-getcanonicallocales/polyfill.js",
		"@formatjs/intl-locale/polyfill.js",
		"@formatjs/intl-pluralrules/polyfill.js",
		...locales.map((locale) => `@formatjs/intl-pluralrules/locale-data/${locale}.js`),
		"@formatjs/intl-numberformat/polyfill.js",
		...locales.map((locale) => `@formatjs/intl-numberformat/locale-data/${locale}.js`),
		"@formatjs/intl-datetimeformat/polyfill.js",
		...locales.map((locale) => `@formatjs/intl-datetimeformat/locale-data/${locale}.js`),
		options.timeZones === "all" ? "@formatjs/intl-datetimeformat/add-all-tz.js" : "@formatjs/intl-datetimeformat/add-golden-tz.js"
	].map((id) => `import ${JSON.stringify(id)};`).join("\n")}
Intl.DateTimeFormat.__setDefaultTimeZone?.(__wabou_system_time_zone());`;
}
function manifestIntl(root) {
	if (!existsSync(resolve(root, "wabou.toml"))) return {};
	const { manifest, path } = readManifest(root);
	const intl = manifest.intl;
	if (intl === void 0) return {};
	if (typeof intl !== "object" || intl === null || Array.isArray(intl)) throw new Error(`${path} intl must be a table`);
	const value = intl;
	if (value.locales !== void 0 && (!Array.isArray(value.locales) || value.locales.some((locale) => typeof locale !== "string"))) throw new Error(`${path} intl.locales must be an array of strings`);
	if (value["time-zones"] !== void 0 && value["time-zones"] !== "golden" && value["time-zones"] !== "all") throw new Error(`${path} intl.time-zones must be "golden" or "all"`);
	return {
		locales: value.locales,
		timeZones: value["time-zones"]
	};
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