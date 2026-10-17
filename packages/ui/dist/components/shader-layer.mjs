import { NativeWidget } from "../primitives/view.mjs";
import { mergeClasses } from "@wabou/core/style";
import { omit } from "solid-js";
import { createComponent as createComponent$1, mergeProps } from "@wabou/core/renderer";
//#region src/components/shader-layer.tsx
/** A layout-aware custom WGSL surface rendered and animated by the Vello host. */
function ShaderLayer(props) {
	const forwarded = omit(props, "source", "module", "values", "time", "speed", "animated", "paused", "class");
	const config = () => ({
		source: props.source,
		module: props.module ?? false,
		values: props.values ?? [],
		time: props.time ?? 0,
		speed: props.speed ?? 1,
		animated: props.animated ?? true,
		paused: props.paused ?? false
	});
	return createComponent$1(NativeWidget, mergeProps(forwarded, {
		tag: "shader-layer",
		get role() {
			return props.role ?? "img";
		},
		get "aria-label"() {
			return props["aria-label"] ?? "Custom GPU effect";
		},
		get "class"() {
			return mergeClasses("overflow-hidden", props.class);
		},
		get config() {
			return config();
		}
	}));
}
//#endregion
export { ShaderLayer };

//# sourceMappingURL=shader-layer.mjs.map