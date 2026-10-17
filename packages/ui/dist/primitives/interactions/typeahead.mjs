//#region src/primitives/interactions/typeahead.ts
function createTypeahead(options = {}) {
	let keys = "";
	let timer;
	const collator = typeof Intl === "undefined" || typeof Intl.Collator !== "function" ? void 0 : new Intl.Collator(options.locale, {
		usage: "search",
		sensitivity: "base"
	});
	const reset = () => {
		keys = "";
		if (timer !== void 0) clearTimeout(timer);
		timer = void 0;
	};
	return {
		search(items, key, activeId) {
			if (key.length !== 1) return void 0;
			keys += key;
			if (timer !== void 0) clearTimeout(timer);
			timer = setTimeout(reset, options.timeout ?? 350);
			const query = keys.length > 1 && [...keys].every((value) => value === keys[0]) ? keys[0] : keys;
			const enabled = items.filter((item) => !item.disabled && item.textValue);
			const active = enabled.findIndex((item) => item.id === activeId);
			return [...enabled.slice(active + 1), ...enabled.slice(0, active + 1)].find((item) => {
				const prefix = item.textValue?.slice(0, query.length) ?? "";
				return collator ? collator.compare(prefix, query) === 0 : prefix.toLowerCase() === query.toLowerCase();
			});
		},
		reset
	};
}
//#endregion
export { createTypeahead };

//# sourceMappingURL=typeahead.mjs.map