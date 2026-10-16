import { useWindow } from "@wabou/core";
import { createContext, useContext } from "solid-js";
import { createComponent as createComponent$1 } from "@wabou/core/renderer";
//#region src/animation/config.tsx
const DEFAULT_MOTION_CONFIG = Object.freeze({ reducedMotion: () => useWindow().reducedMotion() });
const MotionConfigContext = createContext(DEFAULT_MOTION_CONFIG);
/** Application-level motion policy inherited by all styled Wabou components. */
function MotionConfigProvider(props) {
	const parent = useMotionConfig();
	return createComponent$1(MotionConfigContext, {
		value: { reducedMotion: () => props.reducedMotion ?? parent.reducedMotion() },
		get children() {
			return props.children;
		}
	});
}
function useMotionConfig() {
	return useContext(MotionConfigContext);
}
function useReducedMotion() {
	return useMotionConfig().reducedMotion;
}
//#endregion
export { MotionConfigProvider, useMotionConfig, useReducedMotion };

//# sourceMappingURL=config.mjs.map