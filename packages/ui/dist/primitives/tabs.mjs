import { createMemo, createSignal } from "solid-js";
//#region src/primitives/tabs.ts
/**
* Stateful tab collection with stable identity and deterministic activation.
*
* Closing the active tab selects its right-hand neighbour, or the previous
* tab when the closed tab was last. Reordering never changes the active key.
*/
function createTabs(options) {
	const initialTabs = [...options.initialTabs ?? []];
	assertUniqueKeys(initialTabs, options.key);
	const initialActiveKey = options.initialActiveKey !== void 0 && initialTabs.some((tab) => options.key(tab) === options.initialActiveKey) ? options.initialActiveKey : initialTabs[0] === void 0 ? void 0 : options.key(initialTabs[0]);
	const [tabs, setTabs] = createSignal(initialTabs);
	const [activeKey, setActiveKey] = createSignal(() => initialActiveKey);
	const focusTargets = /* @__PURE__ */ new Map();
	const commitActiveKey = (key) => {
		if (activeKey() === key) return false;
		setActiveKey(() => key);
		options.onActiveChange?.(key);
		return true;
	};
	const select = (key) => {
		if (!tabs().some((tab) => options.key(tab) === key)) return false;
		return commitActiveKey(key);
	};
	const selectAt = (index) => {
		const values = tabs();
		if (values.length === 0) return false;
		const normalized = (index % values.length + values.length) % values.length;
		return select(options.key(values[normalized]));
	};
	const activeIndex = () => {
		const current = activeKey();
		return current === void 0 ? -1 : tabs().findIndex((tab) => options.key(tab) === current);
	};
	const focus = (key) => {
		const target = focusTargets.get(key);
		if (target === void 0) return false;
		target.focus();
		return true;
	};
	const activateAt = (index, moveFocus) => {
		const values = tabs();
		if (values.length === 0) return false;
		const normalized = (index % values.length + values.length) % values.length;
		const key = options.key(values[normalized]);
		select(key);
		if (moveFocus) focus(key);
		return true;
	};
	return {
		tabs,
		activeKey,
		activeTab: createMemo(() => {
			const current = activeKey();
			return current === void 0 ? void 0 : tabs().find((tab) => options.key(tab) === current);
		}),
		select,
		selectNext: () => selectAt(activeIndex() + 1),
		selectPrevious: () => selectAt(activeIndex() <= 0 ? tabs().length - 1 : activeIndex() - 1),
		selectFirst: () => selectAt(0),
		selectLast: () => selectAt(tabs().length - 1),
		add: (tab, addOptions = {}) => {
			const key = options.key(tab);
			const current = tabs();
			if (current.some((candidate) => options.key(candidate) === key)) return false;
			const index = Math.max(0, Math.min(addOptions.index ?? current.length, current.length));
			const next = [...current];
			next.splice(index, 0, tab);
			setTabs(next);
			if (activeKey() === void 0 || addOptions.activate !== false) commitActiveKey(key);
			return true;
		},
		close: (key) => {
			const current = tabs();
			const index = current.findIndex((tab) => options.key(tab) === key);
			if (index < 0) return false;
			const next = current.filter((_, candidateIndex) => candidateIndex !== index);
			setTabs(next);
			focusTargets.delete(key);
			if (activeKey() === key) {
				const neighbour = next[Math.min(index, next.length - 1)];
				commitActiveKey(neighbour === void 0 ? void 0 : options.key(neighbour));
			}
			return true;
		},
		move: (key, requestedIndex) => {
			const current = tabs();
			const index = current.findIndex((tab) => options.key(tab) === key);
			if (index < 0) return false;
			const target = Math.max(0, Math.min(requestedIndex, current.length - 1));
			if (target === index) return false;
			const next = [...current];
			const [tab] = next.splice(index, 1);
			next.splice(target, 0, tab);
			setTabs(next);
			return true;
		},
		register: (key, node) => {
			if (tabs().some((tab) => options.key(tab) === key)) focusTargets.set(key, node);
		},
		focus,
		handleKeyDown: (key, event) => {
			const index = tabs().findIndex((tab) => options.key(tab) === key);
			if (index < 0) return false;
			const horizontal = options.orientation !== "vertical";
			const target = event.key === "Home" ? 0 : event.key === "End" ? tabs().length - 1 : horizontal && event.key === "ArrowRight" || !horizontal && event.key === "ArrowDown" ? index + 1 : horizontal && event.key === "ArrowLeft" || !horizontal && event.key === "ArrowUp" ? index - 1 : void 0;
			if (target === void 0) return false;
			event.preventDefault?.();
			return activateAt(target, true);
		}
	};
}
function assertUniqueKeys(tabs, key) {
	const keys = /* @__PURE__ */ new Set();
	for (const tab of tabs) {
		const value = key(tab);
		if (keys.has(value)) throw new Error(`Duplicate tab key: ${String(value)}`);
		keys.add(value);
	}
}
//#endregion
export { createTabs };

//# sourceMappingURL=tabs.mjs.map