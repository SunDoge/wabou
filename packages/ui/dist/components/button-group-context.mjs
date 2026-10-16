import { createContext, createSignal, onCleanup, useContext } from "solid-js";
//#region src/components/button-group-context.ts
const ButtonGroupContext = createContext(null);
function createButtonGroupContext(orientation, options) {
	const [items, setItems] = createSignal([], { ownedWrite: true });
	return {
		orientation,
		registerItem() {
			const token = {};
			setItems((current) => [...current, token]);
			onCleanup(() => setItems((current) => current.filter((item) => item !== token)));
			return {
				orientation,
				size: options.size,
				variant: options.variant,
				disabled: options.disabled,
				position: () => {
					const current = items();
					const index = current.indexOf(token);
					if (current.length <= 1) return "only";
					if (index === 0) return "first";
					if (index === current.length - 1) return "last";
					return "middle";
				}
			};
		}
	};
}
function buttonGroupItemCorners(item) {
	const position = item.position();
	if (position === "only") return "rounded-lg";
	if (position === "middle") return "rounded-none";
	if (item.orientation() === "horizontal") return position === "first" ? "rounded-l-lg" : "rounded-r-lg";
	return position === "first" ? "rounded-t-lg" : "rounded-b-lg";
}
function useButtonGroupItem() {
	return useContext(ButtonGroupContext)?.registerItem();
}
//#endregion
export { ButtonGroupContext, buttonGroupItemCorners, createButtonGroupContext, useButtonGroupItem };

//# sourceMappingURL=button-group-context.mjs.map