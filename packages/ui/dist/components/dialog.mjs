import { Text, View } from "../primitives/view.mjs";
import { Modal } from "../primitives/modal.mjs";
import { ScrollArea } from "../primitives/scroll-area.mjs";
import { componentsElevation, componentsSurfaceClass, useComponentsTheme } from "./theme.mjs";
import { rgba } from "@wabou/core";
import { mergeClasses } from "@wabou/core/style";
import { createComponent, memo, mergeProps } from "@wabou/core/renderer";
//#region src/components/dialog.tsx
function Dialog(props) {
	const theme = useComponentsTheme();
	return createComponent(Modal, mergeProps(props, {
		get motion() {
			return memo(() => {
				return props.motion === void 0;
			})() ? { fromScale: .98 } : props.motion;
		},
		get backdropClass() {
			return props.backdropClass;
		},
		get backdropStyle() {
			return {
				"background-color": rgba(51),
				...props.backdropStyle
			};
		},
		get contentClass() {
			return mergeClasses("w-[480px] max-w-full min-w-0 flex flex-col gap-4 p-5", componentsSurfaceClass("modal"), props.contentClass);
		},
		get contentShadows() {
			return memo(() => {
				return props.contentShadows === void 0;
			})() ? componentsElevation(theme(), "modal") : props.contentShadows;
		}
	}));
}
function DialogHeader(props) {
	return createComponent(View, {
		get "class"() {
			return mergeClasses("flex flex-col gap-1", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function DialogFooter(props) {
	return createComponent(View, {
		get "class"() {
			return mergeClasses("flex items-center justify-end gap-2", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function DialogScrollBody(props) {
	return createComponent(ScrollArea, mergeProps(props, {
		get "class"() {
			return mergeClasses("min-h-0 flex-1", props.class);
		},
		get contentClass() {
			return props.contentClass;
		},
		get children() {
			return props.children;
		}
	}));
}
function DialogTitle(props) {
	return createComponent(Text, {
		get "class"() {
			return mergeClasses("text-lg font-semibold text-primary", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function DialogDescription(props) {
	return createComponent(Text, {
		get "class"() {
			return mergeClasses("w-full min-w-0 whitespace-normal text-sm text-muted", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
//#endregion
export { Dialog, DialogDescription, DialogFooter, DialogHeader, DialogScrollBody, DialogTitle };

//# sourceMappingURL=dialog.mjs.map