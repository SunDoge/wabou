//#region src/primitives/interactions/collection.ts
function createCollection(source) {
	const enabled = () => source().filter((item) => !item.disabled);
	const adjacent = (id, delta, loop) => {
		const items = enabled();
		if (items.length === 0) return void 0;
		const next = (id === void 0 ? -1 : items.findIndex((item) => item.id === id)) + delta;
		if (next >= 0 && next < items.length) return items[next];
		if (!loop) return void 0;
		return delta === 1 ? items[0] : items[items.length - 1];
	};
	return {
		items: source,
		find: (id) => source().find((item) => item.id === id),
		indexOf: (id) => source().findIndex((item) => item.id === id),
		first: () => enabled()[0],
		last: () => enabled().at(-1),
		next: (id, loop = false) => adjacent(id, 1, loop),
		previous: (id, loop = false) => adjacent(id, -1, loop)
	};
}
//#endregion
export { createCollection };

//# sourceMappingURL=collection.mjs.map