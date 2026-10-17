import { createEffect, createSignal, untrack } from "solid-js";
//#region src/primitives/presence.ts
/** Explicit mount lifecycle for content whose exit must finish before removal. */
function createPresence(open) {
	const [phase, setPhase] = createSignal(untrack(open) ? "present" : "unmounted");
	createEffect(open, (isOpen) => {
		setPhase((current) => {
			if (isOpen && (current === "unmounted" || current === "exiting")) return "entering";
			if (!isOpen && (current === "present" || current === "entering")) return "exiting";
			return current;
		});
	});
	return {
		phase,
		mounted: () => phase() !== "unmounted",
		finishEnter() {
			if (untrack(open)) setPhase((current) => current === "entering" ? "present" : current);
		},
		finishExit() {
			if (!untrack(open)) setPhase((current) => current === "exiting" ? "unmounted" : current);
		}
	};
}
//#endregion
export { createPresence };

//# sourceMappingURL=presence.mjs.map