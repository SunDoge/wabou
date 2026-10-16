import { createCollection } from "./collection.mjs";
import { createControllableState } from "./state.mjs";
import { createTypeahead } from "./typeahead.mjs";
import { createEffect, createSignal } from "solid-js";
import { P, match } from "ts-pattern";
//#region src/primitives/interactions/select.ts
function updateSelect(state, event, options) {
	const collection = createCollection(() => options.items);
	const initialHighlight = (fallback) => {
		const candidate = fallback ? collection.find(fallback) : void 0;
		return candidate && !candidate.disabled ? candidate.id : collection.first()?.id;
	};
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
	} : openAt(initialHighlight(state.value))).with({ type: "CLOSE" }, () => ({
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
	} : openAt(initialHighlight(state.value))).with({ type: "ARROW_DOWN" }, () => state.open ? move("next") : openAt(initialHighlight(state.value))).with({ type: "ARROW_UP" }, () => state.open ? move("previous") : openAt(state.value ?? collection.last()?.id)).with({ type: "HOME" }, () => openAt(collection.first()?.id)).with({ type: "END" }, () => openAt(collection.last()?.id)).with(P.union({ type: "HIGHLIGHT" }, { type: "TYPEAHEAD" }), ({ id }) => collection.find(id)?.disabled ? {
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
	let observedOpen = false;
	let observedOpenInitialized = false;
	createEffect(open.value, (current) => {
		if (!observedOpenInitialized) {
			observedOpenInitialized = true;
			observedOpen = current;
			if (current) options.execute?.({ type: "FOCUS_CONTENT" });
			return;
		}
		if (current === observedOpen) return;
		observedOpen = current;
		options.execute?.({ type: current ? "FOCUS_CONTENT" : "FOCUS_TRIGGER" });
	});
	const send = (event) => {
		if (options.disabled?.()) return false;
		const controlledOpen = options.open?.();
		const result = updateSelect(state(), event, {
			items: options.items(),
			loop: options.loop
		});
		const previous = state();
		open.set(result.state.open);
		if (result.state.value !== void 0) value.set(result.state.value);
		if (!(controlledOpen !== void 0 && controlledOpen !== result.state.open)) setHighlighted(result.state.highlighted);
		for (const command of result.commands) if (command.type === "SCROLL_TO_ITEM" && result.state.open) options.execute?.(command);
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
export { createSelectInteraction, updateSelect };

//# sourceMappingURL=select.mjs.map