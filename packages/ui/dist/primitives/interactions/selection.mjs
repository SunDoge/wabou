import { createEffect, createMemo, createSignal, untrack } from "solid-js";
import { match } from "ts-pattern";
//#region src/primitives/interactions/selection.ts
function toggleSelection(current, item, mode, allowEmpty = false) {
	return match(mode).with("single", () => current === item && allowEmpty ? void 0 : item).with("multiple", () => {
		const values = Array.isArray(current) ? current : [];
		return values.includes(item) ? values.filter((value) => value !== item) : [...values, item];
	}).exhaustive();
}
function isSelected(selection, item) {
	return Array.isArray(selection) ? selection.includes(item) : selection === item;
}
/**
* Selection state owned by stable keys while values remain host-owned.
* Selected items always resolve to the latest objects from `items`; keys that
* disappear from the source are removed instead of becoming ghost selections.
*/
function createKeyedSelection(options) {
	const initialAvailable = new Set(untrack(options.items).map(options.key));
	const initial = normalizeKeys(options.initialKeys ?? [], initialAvailable, options.mode);
	const [keys, setKeys] = createSignal(initial);
	createEffect(() => new Set(options.items().map(options.key)), (available) => {
		setKeys((current) => {
			const next = normalizeKeys(current, available, options.mode);
			return setsEqual(current, next) ? current : next;
		});
	});
	const selectedItems = createMemo(() => {
		const selected = keys();
		return options.items().filter((item) => selected.has(options.key(item)));
	});
	const set = (nextKeys) => {
		const next = normalizeKeys(nextKeys, new Set(options.items().map(options.key)), options.mode);
		setKeys((current) => setsEqual(current, next) ? current : next);
	};
	const select = (key) => {
		if (options.mode === "single") {
			set([key]);
			return;
		}
		setKeys((current) => {
			if (current.has(key)) return current;
			if (!new Set(options.items().map(options.key)).has(key)) return current;
			return /* @__PURE__ */ new Set([...current, key]);
		});
	};
	const deselect = (key) => {
		setKeys((current) => {
			if (!current.has(key)) return current;
			const next = new Set(current);
			next.delete(key);
			return next;
		});
	};
	return {
		keys,
		items: selectedItems,
		item: () => selectedItems()[0],
		isSelected: (key) => keys().has(key),
		select,
		deselect,
		toggle: (key) => keys().has(key) ? deselect(key) : select(key),
		set,
		clear: () => setKeys((current) => current.size === 0 ? current : /* @__PURE__ */ new Set())
	};
}
function normalizeKeys(keys, available, mode) {
	const next = /* @__PURE__ */ new Set();
	for (const key of keys) {
		if (!available.has(key)) continue;
		next.add(key);
		if (mode === "single") break;
	}
	return next;
}
function setsEqual(left, right) {
	if (left.size !== right.size) return false;
	for (const key of left) if (!right.has(key)) return false;
	return true;
}
//#endregion
export { createKeyedSelection, isSelected, toggleSelection };

//# sourceMappingURL=selection.mjs.map