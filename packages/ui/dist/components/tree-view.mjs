import { Icon, Text, View } from "../primitives/view.mjs";
import { Button } from "../primitives/button.mjs";
import { createControllableState } from "../primitives/interactions/state.mjs";
import { createTypeahead } from "../primitives/interactions/typeahead.mjs";
import { mergeClasses } from "@wabou/core/style";
import { For, createMemo, createSignal, onCleanup, untrack } from "solid-js";
import { VirtualList, createComponent as createComponent$1, memo } from "@wabou/core/renderer";
import chevronDown from "lucide-static/icons/chevron-down.svg?raw";
import chevronRight from "lucide-static/icons/chevron-right.svg?raw";
//#region src/components/tree-view.tsx
/** Validates a nested tree once and provides deterministic visible traversal. */
function createTreeModel(nodes) {
	const byId = /* @__PURE__ */ new Map();
	const parents = /* @__PURE__ */ new Map();
	const entries = /* @__PURE__ */ new Map();
	const visit = (items, parentId) => {
		items.forEach((node, index) => {
			if (!node.id) throw new Error("tree node id must not be empty");
			if (byId.has(node.id)) throw new Error(`tree node id must be unique: ${node.id}`);
			if (!node.label) throw new Error(`tree node label must not be empty: ${node.id}`);
			byId.set(node.id, node);
			parents.set(node.id, parentId);
			entries.set(node.id, {
				node,
				parentId,
				level: parentId === null ? 1 : (entries.get(parentId)?.level ?? 0) + 1,
				position: index + 1,
				setSize: items.length
			});
			if (node.children?.length) visit(node.children, node.id);
		});
	};
	visit(nodes, null);
	const isBranch = (id) => {
		const node = byId.get(id);
		return Boolean(node?.hasChildren || node?.children?.length);
	};
	return {
		get: (id) => byId.get(id),
		parent: (id) => parents.get(id),
		firstChild: (id) => byId.get(id)?.children?.[0]?.id,
		isBranch,
		visible(expandedIds) {
			const expanded = new Set(expandedIds);
			const result = [];
			const flatten = (items) => {
				items.forEach((node) => {
					const entry = entries.get(node.id);
					if (!entry) throw new Error(`missing tree model entry: ${node.id}`);
					result.push(entry);
					if (node.children?.length && expanded.has(node.id)) flatten(node.children);
				});
			};
			flatten(nodes);
			return result;
		}
	};
}
function validateExpandedIds(model, ids) {
	const unique = /* @__PURE__ */ new Set();
	for (const id of ids) {
		if (!model.get(id)) throw new Error(`unknown expanded tree node: ${id}`);
		if (!model.isBranch(id)) throw new Error(`tree leaf cannot be expanded: ${id}`);
		unique.add(id);
	}
	return [...unique];
}
/** A single-select tree with explicit data, expansion, and native focus routing. */
function TreeView(props) {
	const initialModel = createTreeModel(untrack(() => props.items));
	const virtual = untrack(() => props.virtual);
	const model = createMemo(() => createTreeModel(props.items));
	const defaultExpanded = validateExpandedIds(initialModel, untrack(() => props.defaultExpandedIds) ?? []);
	const expandedState = createControllableState({
		value: () => props.expandedIds === void 0 ? void 0 : validateExpandedIds(model(), props.expandedIds),
		defaultValue: defaultExpanded,
		onChange: props.onExpandedChange
	});
	const selectedState = createControllableState({
		value: () => props.selectedId,
		defaultValue: untrack(() => props.defaultSelectedId) ?? null,
		onChange: props.onSelectedChange
	});
	const [activeId, setActiveId] = createSignal(void 0, { ownedWrite: true });
	const typeahead = createTypeahead();
	onCleanup(typeahead.reset);
	const handles = /* @__PURE__ */ new Map();
	let virtualController;
	let pendingFocusId;
	const expanded = () => expandedState.value();
	const visible = createMemo(() => model().visible(expanded()));
	const enabledVisible = () => visible().filter(({ node }) => !node.disabled);
	const isExpanded = (id) => expanded().includes(id);
	const isSelected = (id) => selectedState.value() === id;
	const tabStop = () => {
		const candidates = enabledVisible();
		const active = activeId();
		if (active && candidates.some(({ node }) => node.id === active)) return active;
		const selected = selectedState.value();
		if (selected && candidates.some(({ node }) => node.id === selected)) return selected;
		return candidates[0]?.node.id;
	};
	const focus = (id) => {
		if (!id || model().get(id)?.disabled) return false;
		const handle = handles.get(id);
		if (!handle && virtual) {
			const index = visible().findIndex(({ node }) => node.id === id);
			if (index < 0 || !virtualController) return false;
			pendingFocusId = id;
			setActiveId(id);
			virtualController.scrollToIndex(index);
			return true;
		}
		setActiveId(id);
		handle?.focus();
		return true;
	};
	const setExpanded = (id, next) => {
		if (!model().isBranch(id)) return false;
		const current = expanded();
		if (current.includes(id) === next) return false;
		return expandedState.set(next ? [...current, id] : current.filter((candidate) => candidate !== id));
	};
	const select = (node) => {
		if (node.disabled) return;
		selectedState.set(node.id);
	};
	const activate = (node) => {
		select(node);
		if (model().isBranch(node.id)) setExpanded(node.id, !isExpanded(node.id));
	};
	const moveLinear = (id, key) => {
		const candidates = enabledVisible();
		const index = candidates.findIndex(({ node }) => node.id === id);
		const target = key === "Home" ? candidates[0] : key === "End" ? candidates.at(-1) : key === "ArrowDown" ? candidates[index + 1] : key === "ArrowUp" ? candidates[index - 1] : void 0;
		return focus(target?.node.id);
	};
	const moveTypeahead = (id, event) => {
		if (event.key.trim().length === 0 || event.key.length !== 1 || event.primary || ((event.mods ?? 0) & 14) !== 0) return false;
		const target = typeahead.search(visible().map(({ node }) => ({
			id: node.id,
			disabled: node.disabled,
			textValue: node.label
		})), event.key, id);
		return focus(target?.id);
	};
	const handleKey = (item, event) => {
		const { id } = item.node;
		let handled = false;
		if ([
			"ArrowUp",
			"ArrowDown",
			"Home",
			"End"
		].includes(event.key)) handled = moveLinear(id, event.key);
		else if (event.key === "ArrowRight" && model().isBranch(id)) handled = isExpanded(id) ? focus(model().firstChild(id)) : setExpanded(id, true);
		else if (event.key === "ArrowLeft") handled = isExpanded(id) ? setExpanded(id, false) : focus(item.parentId ?? void 0);
		else handled = moveTypeahead(id, event);
		if (handled) event.preventDefault();
	};
	const renderTreeItem = (item) => {
		const id = untrack(() => item().node.id);
		let ownHandle;
		onCleanup(() => {
			if (handles.get(id) === ownHandle) handles.delete(id);
		});
		const branch = () => model().isBranch(id);
		return createComponent$1(Button, {
			unstyled: true,
			ref: (node) => {
				ownHandle = node;
				handles.set(id, node);
				if (pendingFocusId === id) {
					pendingFocusId = void 0;
					node.focus();
				}
			},
			role: "treeitem",
			get "aria-label"() {
				return item().node.label;
			},
			get "aria-expanded"() {
				return memo(() => {
					return !!branch();
				})() ? isExpanded(id) : void 0;
			},
			get "aria-selected"() {
				return isSelected(id);
			},
			get selected() {
				return isSelected(id);
			},
			get disabled() {
				return item().node.disabled;
			},
			get focusOrder() {
				return tabStop() === id ? 0 : -1;
			},
			class: (state) => mergeClasses("w-full h-8 min-w-0 pr-2 items-center gap-2 rounded-md text-sm", state.selected ? "bg-selected text-primary" : state.hovered ? "bg-control-hover text-primary" : "bg-transparent text-secondary", props.itemClass),
			get style() {
				return { "padding-left": `${8 + (item().level - 1) * 20}px` };
			},
			onFocus: () => setActiveId(id),
			onClick: () => activate(item().node),
			onKeyDown: (event) => handleKey(item(), event),
			get children() {
				return [memo(() => {
					return memo(() => {
						return !!branch();
					})() ? createComponent$1(Icon, {
						"aria-hidden": "true",
						get source() {
							return isExpanded(id) ? chevronDown : chevronRight;
						},
						size: 14,
						class: "flex-none text-muted"
					}) : createComponent$1(View, {
						"aria-hidden": "true",
						class: "w-3.5 h-3.5 flex-none"
					});
				}), memo(() => {
					return memo(() => {
						return !!props.renderItem;
					})() ? props.renderItem(item().node, {
						expanded: isExpanded(id),
						selected: isSelected(id),
						level: item().level
					}) : createComponent$1(Text, {
						maxLines: 1,
						class: "min-w-0 flex-1 text-sm",
						get children() {
							return item().node.label;
						}
					});
				})];
			}
		});
	};
	return virtual ? createComponent$1(VirtualList, {
		items: visible,
		get itemHeight() {
			return virtual.itemHeight;
		},
		get overscan() {
			return virtual.overscan;
		},
		get viewportHeight() {
			return virtual.viewportHeight;
		},
		getItemKey: (item) => item.node.id,
		role: "tree",
		get accessibilityLabel() {
			return props["aria-label"];
		},
		controllerRef: (controller) => {
			virtualController = controller;
		},
		get "class"() {
			return props.class;
		},
		children: (item) => renderTreeItem(item)
	}) : createComponent$1(View, {
		role: "tree",
		get "aria-label"() {
			return props["aria-label"];
		},
		get "class"() {
			return mergeClasses("min-w-0 flex flex-col gap-0.5", props.class);
		},
		get children() {
			return createComponent$1(For, {
				get each() {
					return visible();
				},
				children: (item) => renderTreeItem(() => item)
			});
		}
	});
}
//#endregion
export { TreeView, createTreeModel };

//# sourceMappingURL=tree-view.mjs.map