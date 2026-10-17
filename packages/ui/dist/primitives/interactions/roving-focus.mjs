import { createSignal } from "solid-js";
import { P, match } from "ts-pattern";
//#region src/primitives/interactions/roving-focus.ts
function createRovingFocus(options = {}) {
	const items = [];
	const [activeId, setActiveId] = createSignal(void 0, { ownedWrite: true });
	const [registryVersion, setRegistryVersion] = createSignal(0, { ownedWrite: true });
	const enabled = () => items.filter((item) => !item.disabled?.());
	const currentTabStop = () => {
		registryVersion();
		const candidates = enabled();
		const active = activeId();
		return candidates.find((item) => item.id === active)?.id ?? candidates.find((item) => options.preferred?.(item.id))?.id ?? candidates[0]?.id;
	};
	const activate = (id) => {
		if (!enabled().some((item) => item.id === id)) return false;
		setActiveId(id);
		return true;
	};
	return {
		register(item) {
			items.push(item);
			setRegistryVersion((version) => version + 1);
			return () => {
				const index = items.indexOf(item);
				if (index >= 0) items.splice(index, 1);
				if (activeId() === item.id) setActiveId(void 0);
				setRegistryVersion((version) => version + 1);
			};
		},
		activate,
		isTabStop(id) {
			return currentTabStop() === id;
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
			activate(target.id);
			options.onMove?.(target.id);
			target.target.focus();
			return true;
		}
	};
}
//#endregion
export { createRovingFocus };

//# sourceMappingURL=roving-focus.mjs.map