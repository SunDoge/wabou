import { Text, View } from "../primitives/view.mjs";
import { Popover as Popover$1 } from "../primitives/popover.mjs";
import { componentsElevation, componentsSurfaceClass, useComponentsTheme } from "./theme.mjs";
import { mergeClasses } from "@wabou/core/style";
import { createComponent, memo, mergeProps } from "@wabou/core/renderer";
//#region src/components/popover.tsx
/** A ready-to-use floating surface backed by Wabou's collision-aware overlay. */
function Popover(props) {
	const theme = useComponentsTheme();
	return createComponent(Popover$1, mergeProps(props, {
		get contentClass() {
			return mergeClasses("min-w-48 max-w-sm min-h-0 p-4 flex flex-col gap-3", componentsSurfaceClass("floating"), props.contentClass);
		},
		get contentShadows() {
			return memo(() => {
				return props.contentShadows === void 0;
			})() ? componentsElevation(theme(), "floating") : props.contentShadows;
		}
	}));
}
function PopoverHeader(props) {
	return createComponent(View, {
		get "class"() {
			return mergeClasses("min-w-0 flex flex-col gap-1", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function PopoverTitle(props) {
	return createComponent(Text, {
		role: "heading",
		get "class"() {
			return mergeClasses("whitespace-normal text-sm font-semibold text-primary", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function PopoverDescription(props) {
	return createComponent(Text, {
		get "class"() {
			return mergeClasses("min-w-0 whitespace-normal text-xs text-muted", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function PopoverFooter(props) {
	return createComponent(View, {
		get "class"() {
			return mergeClasses("min-w-0 flex items-center justify-end gap-2", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
//#endregion
export { Popover, PopoverDescription, PopoverFooter, PopoverHeader, PopoverTitle };

//# sourceMappingURL=popover.mjs.map