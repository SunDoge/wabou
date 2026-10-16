import { moveMenuHighlight } from "./menu-state.mjs";
//#region src/components/command-state.ts
function filterCommandItems(items, query) {
	const terms = query.trim().toLocaleLowerCase().split(/\s+/).filter(Boolean);
	if (terms.length === 0) return [...items];
	return items.filter((item) => {
		const haystack = [item.label, ...item.keywords ?? []].join(" ").toLocaleLowerCase();
		return terms.every((term) => haystack.includes(term));
	});
}
function reconcileCommandHighlight(items, highlighted) {
	if (items.some((item) => item.id === highlighted && !item.disabled)) return highlighted;
	return moveMenuHighlight(items, void 0, "first");
}
//#endregion
export { filterCommandItems, reconcileCommandHighlight };

//# sourceMappingURL=command-state.mjs.map