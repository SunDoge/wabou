//#region src/components/menu-state.ts
/** Resolve one keyboard move without coupling menu state to rendering. */
function moveMenuHighlight(items, current, move) {
	const enabled = items.filter((item) => !item.disabled);
	if (enabled.length === 0) return void 0;
	if (move === "first") return enabled[0].id;
	if (move === "last") return enabled.at(-1)?.id;
	const index = enabled.findIndex((item) => item.id === current);
	if (move === "next") return enabled[(index + 1) % enabled.length].id;
	return enabled[(index <= 0 ? enabled.length : index) - 1].id;
}
//#endregion
export { moveMenuHighlight };

//# sourceMappingURL=menu-state.mjs.map