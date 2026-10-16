import { createTransition } from "../animation.mjs";
import { createPresence } from "./presence.mjs";
import { createEffect, untrack } from "solid-js";
//#region src/primitives/transition-presence.ts
/**
* Couples logical presence to an interruptible visual transition.
*
* Closing disables the logical surface immediately while keeping its visual
* subtree mounted until progress reaches zero. Reopening during exit simply
* retargets the current transition instead of remounting the subtree.
*/
function createTransitionPresence(open, options = {}) {
	const presence = createPresence(open);
	const visuallyPresent = () => open() && (options.ready?.() ?? true);
	const transition = createTransition(() => visuallyPresent() ? 1 : 0, {
		initial: options.initialProgress,
		duration: options.duration ?? .16,
		ease: options.ease ?? "easeOut",
		reducedMotion: options.reducedMotion,
		onComplete(value) {
			if (value === 1 && untrack(open)) presence.finishEnter();
			else if (value === 0 && !untrack(open)) presence.finishExit();
		}
	});
	createEffect(() => [
		open(),
		visuallyPresent(),
		transition.value(),
		presence.phase()
	], ([isOpen, isVisible, progress, phase]) => {
		if (isOpen && isVisible && progress === 1 && phase === "entering") presence.finishEnter();
		else if (!isOpen && progress === 0 && phase === "exiting") presence.finishExit();
	});
	return {
		phase: presence.phase,
		mounted: presence.mounted,
		progress: transition.value,
		transition
	};
}
//#endregion
export { createTransitionPresence };

//# sourceMappingURL=transition-presence.mjs.map