import { createComponent, mergeProps } from "@wabou/core/renderer";
import { NativeWidget } from "@wabou/ui/primitives";
import { omit } from "solid-js";
//#region generated/index.tsx
/** Typed Solid wrapper for the Rust `pdf-view` native widget. */
function PdfView(props) {
	const nativeProps = omit(props, "path", "page", "renderScale", "background");
	const config = () => ({
		path: props.path,
		page: props.page,
		renderScale: props.renderScale,
		background: props.background
	});
	return createComponent(NativeWidget, mergeProps(nativeProps, {
		tag: "pdf-view",
		projectionBoundary: true,
		get config() {
			return config();
		}
	}));
}
//#endregion
export { PdfView };

//# sourceMappingURL=index.mjs.map