import { presetWabou, resolveWabouUtility, validateWabouUtility, wabouUtilityManifest } from "./preset.mjs";
import { existsSync, readFileSync } from "node:fs";
import { dirname, join, parse, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { readFile, readdir } from "node:fs/promises";
import { createGenerator } from "@unocss/core";
import MagicString from "magic-string";
import { parse as parse$1 } from "smol-toml";
import { defineConfig, mergeConfig } from "vite";
import solid from "vite-plugin-solid";
//#region src/style-compiler/vite.ts
const DEFAULT_IGNORED_CLASS_PATTERNS = ["lucide", "lucide-*"];
function matchesClassPattern(candidate, pattern) {
	const escaped = pattern.replace(/[.+?^${}()|[\]\\]/g, "\\$&");
	return new RegExp(`^${escaped.replaceAll("*", ".*")}$`).test(candidate);
}
function filterIgnoredClasses(candidates, patterns = []) {
	return [...candidates].filter((candidate) => !patterns.some((pattern) => matchesClassPattern(candidate, pattern)));
}
function parseThemeColor(value, theme, token) {
	const match = value.match(/^#([0-9a-fA-F]{6}|[0-9a-fA-F]{8})$/);
	if (!match) throw new Error(`invalid color theme value for ${theme}.${token}; expected #RRGGBB or #RRGGBBAA`);
	const hex = match[1];
	const parsed = Number.parseInt(hex, 16);
	return hex.length === 6 ? (parsed << 8 | 255) >>> 0 : parsed >>> 0;
}
function compileColorThemes(options) {
	if (!options) return;
	const base = options.themes[options.default];
	if (!base) throw new Error(`default Wabou color theme \`${options.default}\` does not exist`);
	const tokens = Object.keys(base.colors).sort();
	if (!tokens.length) throw new Error("Wabou color themes require at least one token");
	for (const token of tokens) {
		if (!/^[a-z][a-z0-9-]*$/.test(token)) throw new Error(`invalid Wabou color token \`${token}\``);
		if (token in wabouUtilityManifest.colors) throw new Error(`Wabou color token \`${token}\` conflicts with a palette color`);
	}
	const themes = {};
	for (const [name, theme] of Object.entries(options.themes)) {
		const actual = Object.keys(theme.colors).sort();
		const missing = tokens.filter((token) => !(token in theme.colors));
		const unknown = actual.filter((token) => !tokens.includes(token));
		if (missing.length || unknown.length) throw new Error(`Wabou color theme \`${name}\` does not match \`${options.default}\`${missing.length ? `; missing: ${missing.join(", ")}` : ""}${unknown.length ? `; unknown: ${unknown.join(", ")}` : ""}`);
		themes[name] = {
			appearance: theme.appearance,
			colors: Object.fromEntries(tokens.map((token) => [token, parseThemeColor(theme.colors[token], name, token)]))
		};
	}
	return {
		default: options.default,
		themes
	};
}
function semanticColorDeclaration(candidate, tokens) {
	const match = candidate.match(/^(bg|text|border)-(.+)$/);
	if (!match || !tokens.has(match[2])) return;
	return {
		property: match[1] === "bg" ? "background-color" : match[1] === "text" ? "color" : "border-color",
		value: {
			type: "color",
			value: {
				kind: "token",
				name: match[2]
			}
		}
	};
}
function assertSupportedWabouCandidates(candidates, semanticTokens = /* @__PURE__ */ new Set()) {
	const unsupported = [...candidates].filter((candidate) => !semanticColorDeclaration(candidate, semanticTokens)).map((candidate) => validateWabouUtility(candidate)).filter((diagnostic) => diagnostic !== void 0);
	if (unsupported.length) throw new Error(`unsupported Wabou utilities:\n${unsupported.map(({ message }) => `  - ${message}`).join("\n")}`);
}
function compileWabouUtilities(candidates, sourceOrderStart = 0, semanticTokens = /* @__PURE__ */ new Set()) {
	return [...candidates].sort().map((candidate, index) => {
		const semantic = semanticColorDeclaration(candidate, semanticTokens);
		if (semantic) return {
			className: candidate,
			specificity: 10,
			sourceOrder: sourceOrderStart + index,
			declarations: [semantic]
		};
		const utility = resolveWabouUtility(candidate);
		if (!utility) throw new Error(`unsupported Wabou utility \`${candidate}\``);
		return {
			className: candidate,
			specificity: 10,
			sourceOrder: sourceOrderStart + index,
			declarations: utility.declarations
		};
	});
}
/**
* Keep UnoCSS candidates scoped to JSX class props.
*
* Uno's default extractor scans every token, which turns values like
* `role="tab"` or terminal command strings into accidental utilities.
*/
function extractUtilitySource(source) {
	const values = [];
	const pushValue = (value, expression = false) => {
		if (expression) value = value.replace(/(?:===|!==|==|!=)\s*(?:"[^"]*"|'[^']*'|`[^`]*`)/g, "");
		const interpolations = [...value.matchAll(/\$\{([^}]+)\}/g)];
		const selectsCompleteUtilities = (code) => /^\s*[\s\S]+?\?\s*(?:"[^"]*"|'[^']*'|`[^`]*`)\s*:\s*(?:"[^"]*"|'[^']*'|`[^`]*`)\s*$/.test(code);
		if (interpolations.some((match) => !selectsCompleteUtilities(match[1])) || expression && /(?:["'`]\s*\+|\+\s*["'`])/.test(value)) throw new Error("dynamic class construction is not supported; select complete static utilities with classList and put continuous values in typed style");
		values.push(value);
	};
	for (const match of source.matchAll(/\bclass(?:Name)?\s*=\s*(?:"([^"]*)"|'([^']*)'|`([^`]*)`)/g)) pushValue(match[1] ?? match[2] ?? match[3] ?? "");
	for (const match of source.matchAll(/\bclass(?:Name)?\s*=\s*\{/g)) {
		const start = (match.index ?? 0) + match[0].length;
		let depth = 1;
		let quote = "";
		let escaped = false;
		let end = start;
		for (; end < source.length && depth > 0; end++) {
			const character = source[end];
			if (escaped) {
				escaped = false;
				continue;
			}
			if (character === "\\") {
				escaped = true;
				continue;
			}
			if (quote) {
				if (character === quote) quote = "";
				continue;
			}
			if (character === "\"" || character === "'" || character === "`") quote = character;
			else if (character === "{") depth++;
			else if (character === "}") depth--;
		}
		pushValue(source.slice(start, depth === 0 ? end - 1 : end), true);
	}
	for (const match of source.matchAll(/\bclassList\s*=\s*\{\{([\s\S]*?)\}\}/g)) {
		const entries = match[1];
		for (const candidate of entries.matchAll(/(?:^|,)\s*(?:"([^"]+)"|'([^']+)'|([A-Za-z_][\w-]*))\s*:/g)) pushValue(candidate[1] ?? candidate[2] ?? candidate[3]);
	}
	return values.join("\n");
}
/** Conventional Wabou source roots that may contain utility classes. */
function wabouSourceDirectories(root) {
	return [
		"src",
		"ui",
		"packages"
	].map((directory) => join(root, directory));
}
async function findWorkspacePackages(root) {
	let directory = root;
	for (;;) {
		try {
			const manifest = JSON.parse(await readFile(join(directory, "package.json"), "utf8"));
			if (Array.isArray(manifest.workspaces)) return join(directory, "packages");
		} catch {}
		const parent = dirname(directory);
		if (parent === directory || directory === parse(directory).root) return;
		directory = parent;
	}
}
function wabouHotUpdateModules(modules, stylesheetModule) {
	return [.../* @__PURE__ */ new Set([...modules, stylesheetModule])];
}
function wabouStylePlugin(options) {
	let referenceGenerator;
	const sources = /* @__PURE__ */ new Map();
	const sourceRoots = /* @__PURE__ */ new Set([options.root]);
	const colorThemes = compileColorThemes(options.colorThemes);
	const semanticTokens = new Set(Object.keys(colorThemes?.themes[colorThemes.default]?.colors ?? {}));
	const ignoredClassPatterns = [.../* @__PURE__ */ new Set([...DEFAULT_IGNORED_CLASS_PATTERNS, ...options.ignoreClasses ?? []])];
	let stylesheet = {
		version: 6,
		theme: {
			spacing: wabouUtilityManifest.spacing,
			colors: wabouUtilityManifest.colors
		},
		colorThemes,
		diagnostics: [],
		ignoredClassPatterns,
		rules: []
	};
	const virtual = "virtual:wabou-stylesheet";
	const resolved = `\0${virtual}`;
	async function regenerate() {
		if (!referenceGenerator) return;
		const utilitySource = [...sources.values()].join("\n");
		const matched = filterIgnoredClasses((await referenceGenerator.generate(utilitySource, { preflights: false })).matched, ignoredClassPatterns);
		assertSupportedWabouCandidates(matched, semanticTokens);
		stylesheet = {
			version: 6,
			theme: {
				spacing: wabouUtilityManifest.spacing,
				colors: wabouUtilityManifest.colors
			},
			colorThemes,
			diagnostics: [],
			ignoredClassPatterns,
			rules: compileWabouUtilities(matched, 0, semanticTokens)
		};
	}
	function accepts(id) {
		const path = id.split("?", 1)[0];
		if (path.includes("node_modules") || /\.(?:test|spec)\.(?:tsx?|jsx?)$/.test(path) || ![...sourceRoots].some((root) => path === root || path.startsWith(`${root}${sep}`))) return false;
		if (/\.css$/.test(path)) throw new Error("CSS stylesheets are not supported; use static utility classes and typed style values");
		return /\.(tsx|ts|jsx|js)$/.test(path);
	}
	async function scan(directory) {
		let entries;
		try {
			entries = await readdir(directory, { withFileTypes: true });
		} catch {
			return;
		}
		await Promise.all(entries.map(async (entry) => {
			const path = join(directory, entry.name);
			if (entry.isDirectory()) {
				if (!["dist", "node_modules"].includes(entry.name)) await scan(path);
				return;
			}
			if (accepts(path)) {
				const contents = await readFile(path, "utf8");
				sources.set(path, extractUtilitySource(contents));
			}
		}));
	}
	return {
		name: "wabou-style-compiler",
		enforce: "pre",
		async configResolved() {
			referenceGenerator = await createGenerator({
				presets: [presetWabou()],
				rules: [[/^(?:bg|text|border)-(.+)$/, ([, token]) => semanticTokens.has(token) ? { "--wabou-semantic-color": token } : void 0]]
			});
		},
		async buildStart() {
			const workspacePackages = await findWorkspacePackages(options.root);
			if (workspacePackages) sourceRoots.add(workspacePackages);
			await Promise.all([...wabouSourceDirectories(options.root), ...workspacePackages ? [workspacePackages] : []].map(scan));
			await regenerate();
		},
		async transform(code, id) {
			if (id === resolved) return;
			if (!accepts(id)) return;
			sources.set(id, extractUtilitySource(code));
			await regenerate();
		},
		resolveId(id) {
			return id === virtual ? resolved : null;
		},
		load(id) {
			if (id !== resolved) return null;
			return [
				`const __s=${JSON.stringify(JSON.stringify(stylesheet))};`,
				`globalThis.__wabou_set_stylesheet?.(__s);`,
				`if(import.meta.hot)import.meta.hot.accept();`,
				`export default JSON.parse(__s);`
			].join("\n");
		},
		async handleHotUpdate({ file, server, modules }) {
			if (!accepts(file)) return;
			const contents = await readFile(file, "utf8");
			sources.set(file, extractUtilitySource(contents));
			await regenerate();
			const module = server.moduleGraph.getModuleById(resolved);
			if (module) {
				server.moduleGraph.invalidateModule(module);
				return wabouHotUpdateModules(modules, module);
			}
		}
	};
}
//#endregion
//#region src/index.ts
/**
* Semantic colors used by `@wabou/ui` when an application does not provide a
* theme. Keeping this at the Vite boundary means every official component is
* usable in a minimal project while applications can still replace the whole
* token contract explicitly.
*/
const defaultWabouColorThemes = {
	default: "dark",
	themes: {
		dark: {
			appearance: "dark",
			colors: {
				canvas: "#111113",
				surface: "#1b1b1f",
				"surface-muted": "#18181b",
				input: "#18181b",
				control: "#212225",
				"control-hover": "#2b2d31",
				"control-pressed": "#34363b",
				selected: "#27384d",
				primary: "#eeeeef",
				secondary: "#b4b4bb",
				muted: "#8b8d98",
				subtle: "#303136",
				strong: "#484950",
				accent: "#0090ff",
				"accent-hover": "#3b9eff",
				"accent-pressed": "#0588f0",
				"on-accent": "#ffffff",
				danger: "#ef4444",
				"danger-hover": "#dc2626",
				"danger-pressed": "#b91c1c",
				"danger-surface": "#450a0a",
				"danger-primary": "#fecaca",
				"success-surface": "#064e3b",
				"success-primary": "#a7f3d0",
				focus: "#5eb1ef"
			}
		},
		light: {
			appearance: "light",
			colors: {
				canvas: "#fcfcfd",
				surface: "#ffffff",
				"surface-muted": "#f9f9fb",
				input: "#ffffff",
				control: "#f0f0f3",
				"control-hover": "#e8e8ec",
				"control-pressed": "#e0e1e6",
				selected: "#e1f0ff",
				primary: "#1c2024",
				secondary: "#60646c",
				muted: "#8b8d98",
				subtle: "#d9d9e0",
				strong: "#b9bbc3",
				accent: "#0090ff",
				"accent-hover": "#0588f0",
				"accent-pressed": "#0d74ce",
				"on-accent": "#ffffff",
				danger: "#dc2626",
				"danger-hover": "#b91c1c",
				"danger-pressed": "#991b1b",
				"danger-surface": "#fef2f2",
				"danger-primary": "#991b1b",
				"success-surface": "#ecfdf5",
				"success-primary": "#047857",
				focus: "#0d74ce"
			}
		}
	}
};
function configureDependencyOptimizer() {
	return {
		name: "wabou-configure-deps-optimizer",
		enforce: "post",
		configResolved(config) {
			if (config.command === "serve") {
				config.optimizeDeps.noDiscovery = true;
				config.optimizeDeps.include ??= [];
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
			colorThemes: theme ?? defaultWabouColorThemes,
			ignoreClasses
		}),
		...solid({ solid: {
			generate: "universal",
			moduleName: "@wabou/core/renderer"
		} }),
		configureDependencyOptimizer()
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
	const debug = process.env.WABOU_ENV_DEBUG === "true" || environment.command === "serve";
	const sourcemap = sourceMap === "true" ? true : sourceMap === "false" ? false : manifestSourceMap(root) ?? debug;
	const renderer = fileURLToPath(import.meta.resolve("@wabou/core/renderer"));
	const defaults = {
		define: { "process.env.NODE_ENV": JSON.stringify(process.env.NODE_ENV ?? (environment.command === "serve" ? "development" : "production")) },
		plugins: wabouPlugins(root, options.theme, options.ignoreClasses, options.intl, options.entry ?? "ui/index.tsx"),
		resolve: { alias: {
			"@wabou/core/renderer": renderer,
			"solid-js/web": renderer
		} },
		optimizeDeps: {
			noDiscovery: true,
			include: ["@tanstack/router-core"]
		},
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
			minify: debug ? false : "esbuild"
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
	].map((id) => fileURLToPath(import.meta.resolve(id))).map((id) => `import ${JSON.stringify(id)};`).join("\n")}
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
			manifest: parse$1(readFileSync(path, "utf8")),
			path
		};
	} catch (error) {
		throw new Error(`cannot read Wabou build output from ${path}: ${error}`);
	}
}
//#endregion
export { defaultWabouColorThemes, defineWabouConfig, wabouPlugins };

//# sourceMappingURL=index.mjs.map