import { readFile, readdir } from "node:fs/promises";
import { dirname, join, parse, sep } from "node:path";
import { createGenerator } from "@unocss/core";
import { presetWabou, resolveWabouUtility, validateWabouUtility, wabouUtilityManifest } from "@wabou/unocss-preset";
//#region src/ir.ts
const STYLE_IR_VERSION = 6;
//#endregion
//#region src/support-matrix.ts
/**
* CSS support matrix — compiler ↔ host contract.
*
* - **supported**: compile to Style IR and apply in Rust `apply_ir`.
* - **unsupported**: compile-time error (never emit IR).
*
* Source of truth: `../css-support-matrix.json` (also `include_str!`'d by
* wabou-runtime tests so Rust cannot drift).
*/
const CSS_SUPPORT_MATRIX = {
	version: 1,
	description: "CSS support contract between @wabou/style-compiler and wabou-shell apply_ir. Every compiler-emitted property maps to Taffy or Vello; everything else is a compile error.",
	supported: {
		"display": "flex",
		"flex-direction": "column",
		"justify-content": "center",
		"align-items": "center",
		"align-content": "center",
		"align-self": "center",
		"flex-grow": "1",
		"flex-shrink": "0",
		"flex-basis": "auto",
		"flex": "1 1 auto",
		"flex-wrap": "wrap",
		"grid-template-columns": "1fr 1fr",
		"grid-template-rows": "auto",
		"grid-template-areas": "\"a b\"",
		"grid-template": "auto / 1fr 1fr",
		"width": "10px",
		"height": "10px",
		"min-width": "0px",
		"min-height": "0px",
		"max-width": "100px",
		"max-height": "100px",
		"aspect-ratio": "1",
		"position": "absolute",
		"top": "0px",
		"right": "0px",
		"bottom": "0px",
		"left": "0px",
		"gap": "8px",
		"row-gap": "8px",
		"column-gap": "8px",
		"gap-x": {
			"sample": "8px",
			"rustOnly": true
		},
		"gap-y": {
			"sample": "8px",
			"rustOnly": true
		},
		"padding": "8px",
		"padding-top": "8px",
		"padding-right": "8px",
		"padding-bottom": "8px",
		"padding-left": "8px",
		"margin": "8px",
		"margin-top": "8px",
		"margin-right": "8px",
		"margin-bottom": "8px",
		"margin-left": "8px",
		"margin-inline-end": "8px",
		"margin-inline-start": "8px",
		"border-radius": "4px",
		"background-color": "#ffffff",
		"background": "#ffffff",
		"color": "#000000",
		"font-size": "16px",
		"font-family": "sans-serif",
		"font-weight": "700",
		"line-height": "1.5",
		"overflow": "hidden",
		"overflow-x": "auto",
		"overflow-y": "scroll",
		"border-width": "1px",
		"border-color": "#000000",
		"border-top-width": "1px",
		"border-right-width": "1px",
		"border-bottom-width": "1px",
		"border-left-width": "1px",
		"box-sizing": "border-box",
		"white-space": "nowrap",
		"text-overflow": "ellipsis",
		"text-align": "center",
		"opacity": "0.5",
		"z-index": "1",
		"pointer-events": "none",
		"user-select": "none",
		"transform": "translate(1px, 2px)",
		"transform-component": {
			"sample": "translateX(1px)",
			"rustOnly": true
		},
		"transform-translate-x": {
			"sample": "translateX(1px)",
			"rustOnly": true
		},
		"transform-translate-y": {
			"sample": "translateY(1px)",
			"rustOnly": true
		},
		"transform-scale": {
			"sample": "scale(1.5)",
			"rustOnly": true
		},
		"transform-rotate": {
			"sample": "rotate(45deg)",
			"rustOnly": true
		},
		"box-shadow": "0 1px 2px #00000080"
	},
	unsupported: {
		"border-style": "Only solid borders are implemented; style keywords have no host mapping.",
		"cursor": "System cursor rendering is not implemented.",
		"text-decoration": "Text decoration painting is not implemented.",
		"text-decoration-line": "Text decoration painting is not implemented.",
		"outline": "Outline painting is not implemented.",
		"outline-offset": "Outline painting is not implemented.",
		"outline-width": "Outline painting is not implemented.",
		"outline-color": "Outline painting is not implemented.",
		"outline-style": "Outline painting is not implemented.",
		"list-style-type": "List marker painting is not implemented.",
		"transition": "CSS is stateless; drive runtime transform state from JS or Rust.",
		"animation": "CSS is stateless; drive runtime transform state from JS or Rust."
	},
	unsupportedPrefixes: {
		"transition-": "CSS is stateless; drive runtime transform state from JS or Rust.",
		"animation-": "CSS is stateless; drive runtime transform state from JS or Rust."
	}
};
function sampleOf(spec) {
	return typeof spec === "string" ? spec : spec.sample;
}
function isRustOnly(spec) {
	return typeof spec === "object" && !!spec.rustOnly;
}
/** Every property name Rust `apply_ir` must accept. */
function allHostProperties() {
	return Object.keys(CSS_SUPPORT_MATRIX.supported).sort();
}
/** Properties the compiler is allowed to emit (excludes rust-only aliases). */
function allCompilerProperties() {
	const out = [];
	for (const [name, spec] of Object.entries(CSS_SUPPORT_MATRIX.supported)) if (!isRustOnly(spec)) out.push(name);
	return out.sort();
}
function propertySample(property) {
	const supported = CSS_SUPPORT_MATRIX.supported[property];
	if (supported) return sampleOf(supported);
}
/**
* Classify a property for the compiler. Returns a human-readable reject
* message when the property must not enter Style IR.
*/
function rejectUnsupportedProperty(property) {
	if (property in CSS_SUPPORT_MATRIX.supported) return;
	if (property in CSS_SUPPORT_MATRIX.unsupported) return `unsupported CSS property ${property}: ${CSS_SUPPORT_MATRIX.unsupported[property]}`;
	for (const [prefix, reason] of Object.entries(CSS_SUPPORT_MATRIX.unsupportedPrefixes)) if (property.startsWith(prefix)) return `unsupported CSS property ${property}: ${reason}`;
	return `unsupported CSS property ${property}: not in the wabou CSS support matrix (add it as supported or unsupported)`;
}
function supportKind(property) {
	if (property in CSS_SUPPORT_MATRIX.supported) return "supported";
	if (property in CSS_SUPPORT_MATRIX.unsupported) return "unsupported";
	for (const prefix of Object.keys(CSS_SUPPORT_MATRIX.unsupportedPrefixes)) if (property.startsWith(prefix)) return "unsupported";
	return "unknown";
}
//#endregion
//#region src/vite.ts
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
export { CSS_SUPPORT_MATRIX, STYLE_IR_VERSION, allCompilerProperties, allHostProperties, assertSupportedWabouCandidates, compileColorThemes, compileWabouUtilities, extractUtilitySource, filterIgnoredClasses, findWorkspacePackages, matchesClassPattern, propertySample, rejectUnsupportedProperty, supportKind, wabouHotUpdateModules, wabouSourceDirectories, wabouStylePlugin };

//# sourceMappingURL=index.mjs.map