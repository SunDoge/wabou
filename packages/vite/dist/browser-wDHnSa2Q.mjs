import { n as resolveWabouUtility, r as validateWabouUtility } from "./preset-Ds47hqQP.mjs";
//#region src/style-compiler/browser.ts
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
	const ordered = [...candidates].sort();
	assertSupportedWabouCandidates(ordered, semanticTokens);
	return ordered.map((candidate, index) => {
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
//#endregion
export { compileWabouUtilities as n, assertSupportedWabouCandidates as t };

//# sourceMappingURL=browser-wDHnSa2Q.mjs.map