import { createSignal } from "solid-js";
import { P, match } from "ts-pattern";
//#region src/interactions/collection.ts
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
//#region src/interactions/machine.ts
/** Solid adapter for an Elm-style pure update function and explicit commands. */
function createMachine(options) {
	const [state, setState] = createSignal(() => options.initialState);
	const send = (event) => {
		const previous = state();
		const result = options.update(previous, event);
		const changed = !Object.is(previous, result.state);
		if (changed) setState(() => result.state);
		options.onTransition?.(result, event);
		for (const command of result.commands) options.execute?.(command, send);
		return changed;
	};
	return {
		state,
		send
	};
}
function unchanged(state) {
	return {
		state,
		commands: []
	};
}
//#endregion
//#region src/interactions/state.ts
function createControllableState(options) {
	const [local, setLocal] = createSignal(() => options.defaultValue);
	const value = () => options.value() ?? local();
	return {
		value,
		set(next) {
			if (options.disabled?.() || Object.is(value(), next)) return false;
			if (options.value() === void 0) setLocal(() => next);
			options.onChange?.(next);
			return true;
		}
	};
}
//#endregion
//#region src/interactions/disclosure.ts
function updateDisclosure(state, event) {
	return match(event).with({ type: "DISABLED" }, ({ disabled }) => ({
		state: {
			...state,
			disabled,
			open: disabled ? false : state.open
		},
		commands: []
	})).with({ type: "OPEN" }, () => state.disabled || state.open ? unchanged(state) : {
		state: {
			...state,
			open: true
		},
		commands: []
	}).with({ type: "CLOSE" }, () => !state.open ? unchanged(state) : {
		state: {
			...state,
			open: false
		},
		commands: []
	}).with({ type: "TOGGLE" }, () => state.disabled ? unchanged(state) : {
		state: {
			...state,
			open: !state.open
		},
		commands: []
	}).exhaustive();
}
function createDisclosure(options = {}) {
	const controlled = createControllableState({
		value: options.open ?? (() => void 0),
		defaultValue: options.defaultOpen ?? false,
		disabled: options.disabled,
		onChange: options.onOpenChange
	});
	const set = (type) => {
		const result = updateDisclosure({
			open: controlled.value(),
			disabled: options.disabled?.() ?? false
		}, { type });
		return controlled.set(result.state.open);
	};
	return {
		open: controlled.value,
		disabled: () => options.disabled?.() ?? false,
		openDisclosure: () => set("OPEN"),
		close: () => set("CLOSE"),
		toggle: () => set("TOGGLE")
	};
}
//#endregion
//#region src/interactions/roving-focus.ts
function createRovingFocus(options = {}) {
	const items = [];
	const enabled = () => items.filter((item) => !item.disabled?.());
	return {
		register(item) {
			items.push(item);
			return () => {
				const index = items.indexOf(item);
				if (index >= 0) items.splice(index, 1);
			};
		},
		move(current, key) {
			const orientation = options.orientation?.() ?? "horizontal";
			const direction = match({
				orientation,
				key
			}).with({ key: "Home" }, () => "first").with({ key: "End" }, () => "last").with(P.union({
				orientation: "horizontal",
				key: "ArrowRight"
			}, {
				orientation: "vertical",
				key: "ArrowDown"
			}), () => "next").with(P.union({
				orientation: "horizontal",
				key: "ArrowLeft"
			}, {
				orientation: "vertical",
				key: "ArrowUp"
			}), () => "previous").otherwise(() => void 0);
			if (!direction) return false;
			const candidates = enabled();
			if (candidates.length === 0) return false;
			const index = candidates.findIndex((item) => item.id === current);
			const target = match(direction).with("first", () => candidates[0]).with("last", () => candidates.at(-1)).with("next", () => candidates[index + 1] ?? (options.loop === false ? void 0 : candidates[0])).with("previous", () => candidates[index - 1] ?? (options.loop === false ? void 0 : candidates.at(-1))).exhaustive();
			if (!target) return false;
			options.onMove?.(target.id);
			target.target.focus();
			return true;
		}
	};
}
//#endregion
//#region src/interactions/selection.ts
function toggleSelection(current, item, mode, allowEmpty = false) {
	return match(mode).with("single", () => current === item && allowEmpty ? void 0 : item).with("multiple", () => {
		const values = Array.isArray(current) ? current : [];
		return values.includes(item) ? values.filter((value) => value !== item) : [...values, item];
	}).exhaustive();
}
function isSelected(selection, item) {
	return Array.isArray(selection) ? selection.includes(item) : selection === item;
}
//#endregion
//#region src/interactions/typeahead.ts
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
//#region src/interactions/select.ts
function updateSelect(state, event, options) {
	const collection = createCollection(() => options.items);
	const openAt = (id) => ({
		state: {
			...state,
			open: true,
			highlighted: id
		},
		commands: [{ type: "FOCUS_CONTENT" }, ...id ? [{
			type: "SCROLL_TO_ITEM",
			id
		}] : []]
	});
	const move = (direction) => {
		const candidate = direction === "next" ? collection.next(state.highlighted, options.loop ?? true) : collection.previous(state.highlighted, options.loop ?? true);
		if (!candidate) return {
			state,
			commands: []
		};
		return {
			state: {
				...state,
				highlighted: candidate.id
			},
			commands: [{
				type: "SCROLL_TO_ITEM",
				id: candidate.id
			}]
		};
	};
	return match(event).with({ type: "OPEN" }, () => state.open ? {
		state,
		commands: []
	} : openAt(state.value ?? collection.first()?.id)).with({ type: "CLOSE" }, () => ({
		state: {
			...state,
			open: false,
			highlighted: void 0
		},
		commands: state.open ? [{ type: "FOCUS_TRIGGER" }] : []
	})).with({ type: "TOGGLE" }, () => state.open ? {
		state: {
			...state,
			open: false,
			highlighted: void 0
		},
		commands: [{ type: "FOCUS_TRIGGER" }]
	} : openAt(state.value ?? collection.first()?.id)).with({ type: "ARROW_DOWN" }, () => state.open ? move("next") : openAt(state.value ?? collection.first()?.id)).with({ type: "ARROW_UP" }, () => state.open ? move("previous") : openAt(state.value ?? collection.last()?.id)).with({ type: "HOME" }, () => openAt(collection.first()?.id)).with({ type: "END" }, () => openAt(collection.last()?.id)).with(P.union({ type: "HIGHLIGHT" }, { type: "TYPEAHEAD" }), ({ id }) => collection.find(id)?.disabled ? {
		state,
		commands: []
	} : event.type === "TYPEAHEAD" && !state.open ? openAt(id) : {
		state: {
			...state,
			highlighted: id
		},
		commands: [{
			type: "SCROLL_TO_ITEM",
			id
		}]
	}).with({ type: "SELECT" }, ({ id }) => {
		const selected = id ?? state.highlighted;
		if (!selected || collection.find(selected)?.disabled) return {
			state,
			commands: []
		};
		const close = options.closeOnSelect ?? true;
		return {
			state: {
				open: close ? false : state.open,
				value: selected,
				highlighted: close ? void 0 : selected
			},
			commands: close ? [{ type: "FOCUS_TRIGGER" }] : []
		};
	}).exhaustive();
}
function createSelectInteraction(options) {
	const value = createControllableState({
		value: options.value ?? (() => void 0),
		defaultValue: options.defaultValue,
		disabled: options.disabled,
		onChange: (next) => next && options.onValueChange?.(next)
	});
	const open = createControllableState({
		value: options.open ?? (() => void 0),
		defaultValue: options.defaultOpen ?? false,
		disabled: options.disabled,
		onChange: options.onOpenChange
	});
	const [highlighted, setHighlighted] = createSignal();
	const typeahead = createTypeahead();
	const state = () => ({
		open: open.value(),
		value: value.value(),
		highlighted: highlighted()
	});
	const send = (event) => {
		if (options.disabled?.()) return false;
		const result = updateSelect(state(), event, {
			items: options.items(),
			loop: options.loop
		});
		const previous = state();
		open.set(result.state.open);
		if (result.state.value !== void 0) value.set(result.state.value);
		setHighlighted(result.state.highlighted);
		for (const command of result.commands) options.execute?.(command);
		return previous.open !== result.state.open || previous.value !== result.state.value || previous.highlighted !== result.state.highlighted;
	};
	return {
		state,
		open: open.value,
		value: value.value,
		highlighted,
		send,
		typeahead(key) {
			const item = typeahead.search(options.items(), key, highlighted());
			return item ? send({
				type: "TYPEAHEAD",
				id: item.id
			}) : false;
		}
	};
}
//#endregion
export { createCollection, createControllableState, createDisclosure, createMachine, createRovingFocus, createSelectInteraction, createTypeahead, isSelected, toggleSelection, unchanged, updateDisclosure, updateSelect };

//# sourceMappingURL=interactions.mjs.map