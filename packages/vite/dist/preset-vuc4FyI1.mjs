import { t as manifest_default } from "./manifest-DheYpL5v.mjs";
//#endregion
//#region src/style-compiler/support-matrix.ts
/**
* CSS support matrix — compiler ↔ host contract.
*
* - **supported**: compile to Style IR and apply in Rust `apply_ir`.
* - **unsupported**: compile-time error (never emit IR).
*
* Source of truth: `./css-support-matrix.json` (also `include_str!`'d by
* wabou-runtime tests so Rust cannot drift).
*/
const CSS_SUPPORT_MATRIX = {
	version: 2,
	description: "Formal style contract between @wabou/vite and the GPUI-CE projection. Every compiler-emitted property maps losslessly to GPUI Style or explicit Wabou node metadata; everything else is a compile error.",
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
		"flex-wrap": "wrap",
		"grid-template-columns": "1fr 1fr",
		"grid-template-rows": "auto",
		"grid-column-start": {
			"sample": "1",
			"rustOnly": true
		},
		"grid-column-end": {
			"sample": "2",
			"rustOnly": true
		},
		"grid-row-start": {
			"sample": "1",
			"rustOnly": true
		},
		"grid-row-end": {
			"sample": "2",
			"rustOnly": true
		},
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
		"border-top-left-radius": "4px",
		"border-top-right-radius": "4px",
		"border-bottom-right-radius": "4px",
		"border-bottom-left-radius": "4px",
		"background-color": "#ffffff",
		"background": "#ffffff",
		"color": "#000000",
		"font-size": "16px",
		"font-family": "sans-serif",
		"font-weight": "700",
		"font-style": "italic",
		"text-decoration-line": "underline",
		"letter-spacing": "1px",
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
		"white-space": "nowrap",
		"text-overflow": "ellipsis",
		"text-align": "center",
		"opacity": "0.5",
		"z-index": "1",
		"pointer-events": "none",
		"cursor": "pointer",
		"user-select": "text",
		"filter-blur": {
			"sample": "8px",
			"rustOnly": true
		},
		"backdrop-blur": {
			"sample": "8px",
			"rustOnly": true
		},
		"box-shadow": "0 1px 2px #00000080"
	},
	unsupported: {
		"box-sizing": "GPUI owns box layout and does not expose a selectable CSS box model.",
		"contain": "GPUI does not expose CSS containment as a style property.",
		"flex": "Use the explicit flex-grow, flex-shrink and flex-basis fields projected by GPUI.",
		"grid-auto-flow": "The formal GPUI grid subset requires explicit templates and placements.",
		"grid-template": "Use GPUI grid-template-columns and grid-template-rows explicitly.",
		"grid-template-areas": "GPUI does not expose named grid areas.",
		"justify-items": "GPUI Style does not expose justify-items.",
		"justify-self": "GPUI Style does not expose justify-self.",
		"border-style": "Only solid borders are implemented; style keywords have no host mapping.",
		"text-decoration": "Text decoration painting is not implemented.",
		"outline": "GPUI Style does not expose production outlines; use borders or a dedicated focus-ring component.",
		"outline-color": "GPUI Style does not expose production outlines.",
		"outline-offset": "GPUI Style does not expose production outlines.",
		"outline-style": "GPUI Style does not expose production outlines.",
		"outline-width": "GPUI Style does not expose production outlines.",
		"list-style-type": "List marker painting is not implemented.",
		"transition": "CSS is stateless; drive runtime transform state from JS or Rust.",
		"animation": "CSS is stateless; drive runtime transform state from JS or Rust.",
		"transform": "Use the explicit runtime transform opcode; GPUI Style has no CSS transform field.",
		"transform-component": "Use the explicit runtime transform opcode.",
		"transform-origin-x": "GPUI Style has no CSS transform-origin field.",
		"transform-origin-y": "GPUI Style has no CSS transform-origin field.",
		"transform-rotate": "Use the explicit runtime transform opcode.",
		"transform-scale": "Use the explicit runtime transform opcode.",
		"transform-translate-x": "Use the explicit runtime transform opcode.",
		"transform-translate-y": "Use the explicit runtime transform opcode."
	},
	unsupportedPrefixes: {
		"transition-": "CSS is stateless; drive runtime transform state from JS or Rust.",
		"animation-": "CSS is stateless; drive runtime transform state from JS or Rust."
	}
};
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
//#endregion
//#region src/preset/index.ts
const wabouUtilityManifest = manifest_default;
function matchDynamic(utility, resolver) {
	const negative = [
		"spacing",
		"dimension",
		"translate",
		"rotate"
	].includes(resolver) && utility.startsWith("-");
	const normalized = negative ? utility.slice(1) : utility;
	const rule = wabouUtilityManifest.dynamicRules.find((candidate) => candidate.resolver === resolver);
	for (const prefix of [...rule?.prefixes ?? []].sort((left, right) => right.name.length - left.name.length)) {
		const marker = `${prefix.name}-`;
		if (normalized.startsWith(marker)) return {
			name: prefix.name,
			token: normalized.slice(marker.length),
			properties: prefix.properties,
			negative
		};
	}
}
function parseLength(token, spacing) {
	if (token === "auto") return { unit: "auto" };
	if (token === "full") return {
		unit: "percent",
		value: 1
	};
	if (spacing && token === "px") return {
		unit: "px",
		value: 1
	};
	const scale = spacing ? wabouUtilityManifest.spacing[token] : void 0;
	if (scale !== void 0) return {
		unit: "px",
		value: scale
	};
	if (token.endsWith("%")) {
		const value = Number(token.slice(0, -1));
		if (Number.isFinite(value)) return {
			unit: "percent",
			value: value / 100
		};
	}
	const raw = token.startsWith("[") && token.endsWith("]") ? token.slice(1, -1) : void 0;
	if (!raw) return;
	if (raw.endsWith("px")) {
		const value = Number(raw.slice(0, -2));
		if (Number.isFinite(value)) return {
			unit: "px",
			value
		};
	}
	if (raw.endsWith("rem")) {
		const value = Number(raw.slice(0, -3));
		if (Number.isFinite(value)) return {
			unit: "px",
			value: value * 16
		};
	}
	if (raw.endsWith("%")) {
		const value = Number(raw.slice(0, -1));
		if (Number.isFinite(value)) return {
			unit: "percent",
			value: value / 100
		};
	}
}
function parseDimensionFraction(token) {
	const match = /^(\d+)\/([1-9]\d*)$/.exec(token);
	if (!match) return;
	const numerator = Number(match[1]);
	const denominator = Number(match[2]);
	if (!Number.isInteger(numerator) || !Number.isInteger(denominator) || numerator > 4294967295 || denominator > 4294967295) return;
	return {
		unit: "percent",
		value: Math.fround(numerator / denominator)
	};
}
function parseDimensionLength(token) {
	return parseLength(token, false) ?? parseDimensionFraction(token) ?? parseLength(token, true);
}
function negateLength(value) {
	if (value.unit === "auto") return;
	return {
		...value,
		value: -value.value
	};
}
const rustF32 = Math.fround;
const lengthDeclaration = (property, value) => ({
	property,
	value: {
		type: "length",
		value
	}
});
const transformDeclaration = (kind, value) => ({
	property: {
		translateX: "transform-translate-x",
		translateY: "transform-translate-y",
		scale: "transform-scale",
		rotate: "transform-rotate"
	}[kind] ?? "transform-component",
	value: {
		type: "list",
		values: [{
			type: "record",
			fields: {
				kind: {
					type: "keyword",
					value: kind
				},
				value
			}
		}]
	}
});
function parseCandidate(candidate) {
	const matcher = candidate;
	if (matcher.includes(":")) return {
		candidate,
		message: `Wabou variants are not supported in \`${candidate}\`; use Solid classList or typed style`
	};
	let declarations = wabouUtilityManifest.staticUtilities[matcher];
	const spacing = matchDynamic(matcher, "spacing");
	if (!declarations && spacing) {
		let value = parseLength(spacing.token, true);
		if (value?.unit === "auto" && !spacing.properties.every((property) => property.startsWith("margin-"))) return {
			candidate,
			message: `invalid Wabou spacing in \`${candidate}\`; auto is only valid for margins`
		};
		if (spacing.negative) {
			if (!spacing.properties.every((property) => property.startsWith("margin-"))) return {
				candidate,
				message: `invalid negative Wabou spacing in \`${candidate}\`; only margins may be negative`
			};
			value = value && negateLength(value);
		}
		if (!value) return {
			candidate,
			message: `invalid Wabou spacing in \`${candidate}\`; expected a scale token, px, rem, or percentage`
		};
		declarations = spacing.properties.map((property) => lengthDeclaration(property, value));
	}
	const dimension = matchDynamic(matcher, "dimension");
	if (!declarations && dimension) {
		let value = parseDimensionLength(dimension.token);
		if (dimension.negative) {
			if (!dimension.properties.every((property) => [
				"top",
				"right",
				"bottom",
				"left"
			].includes(property))) return {
				candidate,
				message: `invalid negative Wabou dimension in \`${candidate}\`; only positioned edges may be negative`
			};
			value = value && negateLength(value);
		}
		if (!value) return {
			candidate,
			message: `invalid Wabou dimension in \`${candidate}\`; expected auto, full, a fraction, a scale token, px, rem, or percentage`
		};
		declarations = dimension.properties.map((name) => lengthDeclaration(name, value));
	}
	const lengthRule = matchDynamic(matcher, "length");
	if (!declarations && lengthRule) {
		const value = parseLength(lengthRule.token, false);
		if (value) declarations = lengthRule.properties.map((property) => lengthDeclaration(property, value));
	}
	const color = matchDynamic(matcher, "color");
	if (!declarations && color) {
		const [colorName, opacityToken] = color.token.split("/", 2);
		let rgba = wabouUtilityManifest.colors[colorName];
		const arbitrary = colorName.match(/^\[#([0-9a-fA-F]{6}|[0-9a-fA-F]{8})\]$/)?.[1];
		if (rgba === void 0 && arbitrary) {
			rgba = Number.parseInt(arbitrary, 16);
			if (arbitrary.length === 6) rgba = (rgba << 8 | 255) >>> 0;
		}
		if (rgba !== void 0 && opacityToken !== void 0) {
			const opacity = Number(opacityToken);
			rgba = Number.isFinite(opacity) && opacity >= 0 && opacity <= 100 ? (rgba & 4294967040 | Math.round(opacity * 2.55)) >>> 0 : void 0;
		}
		if (rgba === void 0) return {
			candidate,
			message: `unknown Wabou theme color in \`${candidate}\``
		};
		declarations = [{
			property: color.properties[0],
			value: {
				type: "color",
				value: {
					kind: "literal",
					rgba
				}
			}
		}];
	}
	const opacityRule = matchDynamic(matcher, "opacity");
	if (!declarations && opacityRule) {
		const opacity = Number(opacityRule.token);
		if (!Number.isFinite(opacity) || opacity < 0 || opacity > 100) return {
			candidate,
			message: `invalid Wabou opacity in \`${candidate}\``
		};
		declarations = [{
			property: opacityRule.properties[0],
			value: {
				type: "number",
				value: opacity / 100
			}
		}];
	}
	const numberRule = matchDynamic(matcher, "number");
	if (!declarations && numberRule) {
		const raw = numberRule.token.match(/^\[(-?(?:\d+(?:\.\d*)?|\.\d+))\]$/)?.[1];
		const value = raw === void 0 ? NaN : Number(raw);
		if (!Number.isFinite(value)) return {
			candidate,
			message: `invalid Wabou number in \`${candidate}\`; expected an arbitrary finite number`
		};
		declarations = [{
			property: numberRule.properties[0],
			value: {
				type: "number",
				value
			}
		}];
	}
	const ratioRule = matchDynamic(matcher, "ratio");
	if (!declarations && ratioRule) {
		const raw = ratioRule.token.startsWith("[") && ratioRule.token.endsWith("]") ? ratioRule.token.slice(1, -1) : void 0;
		const parts = raw?.split("/", 2);
		const value = rustF32(parts?.length === 2 ? Number(parts[0]) / Number(parts[1]) : Number(raw));
		if (!Number.isFinite(value) || value <= 0) return {
			candidate,
			message: `invalid Wabou ratio in \`${candidate}\`; expected an arbitrary positive ratio`
		};
		declarations = [{
			property: ratioRule.properties[0],
			value: {
				type: "number",
				value
			}
		}];
	}
	const translateRule = matchDynamic(matcher, "translate");
	if (!declarations && translateRule) {
		let value = parseLength(translateRule.token, true);
		if (translateRule.negative) value = value && negateLength(value);
		if (!value) return {
			candidate,
			message: `invalid Wabou translate in \`${candidate}\``
		};
		declarations = [{
			property: translateRule.properties[0],
			value: {
				type: "list",
				values: [{
					type: "record",
					fields: {
						kind: {
							type: "keyword",
							value: translateRule.name === "translate-x" ? "translateX" : "translateY"
						},
						value: {
							type: "length",
							value
						}
					}
				}]
			}
		}];
	}
	const scaleRule = matchDynamic(matcher, "scale");
	if (!declarations && scaleRule) {
		const arbitrary = scaleRule.token.match(/^\[(-?(?:\d+(?:\.\d*)?|\.\d+))\]$/)?.[1];
		const scale = rustF32(arbitrary === void 0 ? Number(scaleRule.token) / 100 : Number(arbitrary));
		if (!Number.isFinite(scale)) return {
			candidate,
			message: `invalid Wabou scale in \`${candidate}\``
		};
		declarations = [transformDeclaration("scale", {
			type: "list",
			values: [{
				type: "number",
				value: scale
			}, {
				type: "number",
				value: scale
			}]
		})];
	}
	const rotateRule = matchDynamic(matcher, "rotate");
	if (!declarations && rotateRule) {
		const arbitrary = rotateRule.token.match(/^\[(-?(?:\d+(?:\.\d*)?|\.\d+))\]$/)?.[1];
		let degrees = Number(arbitrary ?? rotateRule.token);
		if (rotateRule.negative) degrees = -degrees;
		const radians = rustF32(degrees * Math.PI / 180);
		if (!Number.isFinite(radians)) return {
			candidate,
			message: `invalid Wabou rotation in \`${candidate}\``
		};
		declarations = [transformDeclaration("rotate", {
			type: "number",
			value: radians
		})];
	}
	if (!declarations) return {
		candidate,
		message: `unsupported Wabou utility \`${candidate}\``
	};
	return {
		candidate,
		matcher,
		declarations
	};
}
function resolveWabouUtility(candidate) {
	const result = parseCandidate(candidate);
	return "declarations" in result ? result : void 0;
}
function validateWabouUtility(candidate) {
	const result = parseCandidate(candidate);
	if ("message" in result) return result;
	const rejected = result.declarations.map((declaration) => rejectUnsupportedProperty(declaration.property)).find((message) => message !== void 0);
	return rejected ? {
		candidate,
		message: rejected
	} : void 0;
}
function cssValue(value) {
	switch (value.type) {
		case "keyword": return value.value;
		case "boolean": return String(value.value);
		case "number": return value.value;
		case "length":
			if (value.value.unit === "auto") return "auto";
			return `${value.value.unit === "percent" ? value.value.value * 100 : value.value.value}${value.value.unit === "percent" ? "%" : "px"}`;
		case "color": return `#${value.value.rgba.toString(16).padStart(8, "0")}`;
		case "list": return value.values.map((item) => {
			if (item.type !== "record") return cssValue(item);
			const kind = item.fields.kind;
			const argument = item.fields.value;
			if (kind?.type !== "keyword") return "";
			if (kind.value === "repeat") {
				const count = item.fields.count;
				const tracks = item.fields.values;
				if (count?.type !== "number" || tracks?.type !== "list") return "";
				return `repeat(${count.value}, ${tracks.values.map(cssValue).join(" ")})`;
			}
			if (!argument) return "";
			if (kind.value === "breadth") return cssValue(argument);
			if (kind.value === "flex") return `${cssValue(argument)}fr`;
			const text = argument.type === "list" ? argument.values.map(cssValue).join(", ") : cssValue(argument);
			return `${kind.value}(${text})`;
		}).join(" ");
		case "record": {
			const kind = value.fields.kind;
			const argument = value.fields.value;
			if (kind?.type !== "keyword" || !argument) return "";
			if (kind.value === "breadth") return cssValue(argument);
			if (kind.value === "flex") return `${cssValue(argument)}fr`;
			return "";
		}
	}
}
function unoRule() {
	return [/^.+$/, ([candidate]) => {
		const resolved = resolveWabouUtility(candidate);
		if (!resolved) return;
		return Object.fromEntries(resolved.declarations.map(({ property, value }) => [property.startsWith("transform-") ? "transform" : property, cssValue(value)]));
	}];
}
/** UnoCSS adapter for editor tooling over the native utility manifest. */
function presetWabou() {
	return {
		name: "@wabou/vite/preset",
		rules: [unoRule()],
		autocomplete: { templates: [
			"p-$spacing",
			"px-$spacing",
			"py-$spacing",
			"m-$spacing",
			"gap-$spacing",
			"w-$spacing",
			"h-$spacing",
			"bg-$colors",
			"text-$colors",
			"border-$colors"
		] }
	};
}
//#endregion
export { wabouUtilityManifest as i, resolveWabouUtility as n, validateWabouUtility as r, presetWabou as t };

//# sourceMappingURL=preset-vuc4FyI1.mjs.map