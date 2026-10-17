import { Text, View } from "../primitives/view.mjs";
import { componentsElevation, componentsSurfaceClass, useComponentsTheme } from "./theme.mjs";
import { mergeClasses } from "@wabou/core/style";
import { createComponent, createContext, omit, useContext } from "solid-js";
import { createComponent as createComponent$1, memo, mergeProps } from "@wabou/core/renderer";
import { match } from "ts-pattern";
//#region src/components/card.tsx
const CardContext = createContext({ size: () => "default" });
function cardChrome(variant) {
	return match(variant).with("raised", () => componentsSurfaceClass("raised")).with("filled", () => "rounded-lg border border-transparent bg-control").with("outline", () => "rounded-lg border border-strong bg-transparent").with("plain", () => "rounded-none border-0 bg-transparent").exhaustive();
}
function cardGeometry(part, size) {
	return match({
		part,
		size
	}).with({
		part: "header",
		size: "sm"
	}, () => "gap-1 px-4 pt-4 pr-12").with({
		part: "header",
		size: "default"
	}, () => "gap-1.5 px-5 pt-5 pr-14").with({
		part: "header",
		size: "lg"
	}, () => "gap-2 px-6 pt-6 pr-16").with({
		part: "action",
		size: "sm"
	}, () => "top-4 right-4").with({
		part: "action",
		size: "default"
	}, () => "top-5 right-5").with({
		part: "action",
		size: "lg"
	}, () => "top-6 right-6").with({
		part: "content",
		size: "sm"
	}, () => "gap-3 px-4 pt-3 pb-4").with({
		part: "content",
		size: "default"
	}, () => "gap-4 px-5 pt-4 pb-5").with({
		part: "content",
		size: "lg"
	}, () => "gap-5 px-6 pt-5 pb-6").with({
		part: "footer",
		size: "sm"
	}, () => "gap-2 px-4 pb-4").with({
		part: "footer",
		size: "default"
	}, () => "gap-2 px-5 pb-5").with({
		part: "footer",
		size: "lg"
	}, () => "gap-3 px-6 pb-6").exhaustive();
}
function Card(props) {
	const theme = useComponentsTheme();
	const variant = () => props.variant ?? "raised";
	const size = () => props.size ?? "default";
	const rest = omit(props, "variant", "size", "class", "children", "shadows");
	return createComponent(CardContext, {
		value: { size },
		get children() {
			return createComponent$1(View, mergeProps(rest, {
				get "class"() {
					return mergeClasses("min-w-0 min-h-0 flex-none flex flex-col overflow-hidden", cardChrome(variant()), props.class);
				},
				get shadows() {
					return memo(() => {
						return !!(props.shadows === void 0 && variant() === "raised");
					})() ? componentsElevation(theme(), "raised") : props.shadows;
				},
				get children() {
					return props.children;
				}
			}));
		}
	});
}
function CardHeader(props) {
	const context = useContext(CardContext);
	return createComponent$1(View, mergeProps(props, {
		get "class"() {
			return mergeClasses("relative min-w-0 flex flex-col", cardGeometry("header", context.size()), props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function CardTitle(props) {
	const context = useContext(CardContext);
	return createComponent$1(Text, mergeProps(props, {
		get "class"() {
			return mergeClasses("min-w-0 font-semibold text-primary", match(context.size()).with("sm", () => "text-sm").with("default", () => "text-base").with("lg", () => "text-lg").exhaustive(), props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function CardDescription(props) {
	const context = useContext(CardContext);
	return createComponent$1(Text, mergeProps(props, {
		get "class"() {
			return mergeClasses("w-full min-w-0 whitespace-normal text-muted", match(context.size()).with("sm", () => "text-xs").with("default", () => "text-sm").with("lg", () => "text-base").exhaustive(), props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
/** Top-end action slot owned by the relative CardHeader surface. */
function CardAction(props) {
	const context = useContext(CardContext);
	return createComponent$1(View, mergeProps(props, {
		get "class"() {
			return mergeClasses("absolute flex-none flex items-center justify-end", cardGeometry("action", context.size()), props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function CardContent(props) {
	const context = useContext(CardContext);
	return createComponent$1(View, mergeProps(props, {
		get "class"() {
			return mergeClasses("min-w-0 min-h-0 flex flex-col", cardGeometry("content", context.size()), props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function CardFooter(props) {
	const context = useContext(CardContext);
	return createComponent$1(View, mergeProps(props, {
		get "class"() {
			return mergeClasses("min-w-0 flex items-center", cardGeometry("footer", context.size()), props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
//#endregion
export { Card, CardAction, CardContent, CardDescription, CardFooter, CardHeader, CardTitle };

//# sourceMappingURL=card.mjs.map