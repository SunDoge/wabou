import { createMemo, createSignal } from "solid-js";
//#region src/primitives/interactions/form-draft.ts
/** Validation key used for errors that do not belong to one field. */
const FORM_ERROR = Symbol("wabou.form-error");
/**
* A small immutable draft for form fields with explicit reset and commit
* semantics. Transient request/error state belongs outside this model.
*/
function createFormDraft(initial, options = {}) {
	const [baselineBox, setBaselineBox] = createSignal({ value: { ...initial } });
	const [box, setBox] = createSignal({ value: { ...initial } });
	const value = () => box().value;
	const equals = options.equals ?? shallowEqual;
	const dirty = createMemo(() => !equals(value(), baselineBox().value));
	const errors = createMemo(() => options.validate?.(value()) ?? {});
	const valid = createMemo(() => Reflect.ownKeys(errors()).length === 0);
	const replace = (next) => setBox({ value: next });
	const resetTo = (next) => {
		setBaselineBox({ value: { ...next } });
		replace({ ...next });
	};
	const setField = (key, updater) => {
		setBox((currentBox) => {
			const current = currentBox.value;
			const previous = current[key];
			const next = typeof updater === "function" ? updater(previous) : updater;
			return Object.is(previous, next) ? currentBox : { value: {
				...current,
				[key]: next
			} };
		});
	};
	return {
		value,
		dirty,
		errors,
		valid,
		formError: () => errors()[FORM_ERROR],
		fieldError: (key) => errors()[key],
		field: (key) => value()[key],
		control: (key) => [() => value()[key], (next) => setField(key, next)],
		set: setField,
		patch: (patch) => {
			setBox((currentBox) => {
				const current = currentBox.value;
				for (const key of Reflect.ownKeys(patch)) if (!Object.is(current[key], patch[key])) return { value: {
					...current,
					...patch
				} };
				return currentBox;
			});
		},
		reset: () => replace({ ...baselineBox().value }),
		resetTo,
		commit: () => {
			setBox((currentBox) => {
				setBaselineBox({ value: { ...currentBox.value } });
				return currentBox;
			});
		}
	};
}
function shallowEqual(left, right) {
	const keys = Reflect.ownKeys(left);
	if (keys.length !== Reflect.ownKeys(right).length) return false;
	return keys.every((key) => Object.is(left[key], right[key]));
}
//#endregion
export { FORM_ERROR, createFormDraft };

//# sourceMappingURL=form-draft.mjs.map