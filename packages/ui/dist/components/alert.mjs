import { Icon, Text, View } from "../primitives/view.mjs";
import { componentToneClass } from "./tone.mjs";
import { componentsSurfaceClass } from "./theme.mjs";
import { Button } from "./button.mjs";
import { mergeClasses } from "@wabou/core/style";
import { createComponent, createContext, omit, useContext } from "solid-js";
import { createComponent as createComponent$1, memo, mergeProps } from "@wabou/core/renderer";
import { P, match } from "ts-pattern";
import checkCircle from "lucide-static/icons/circle-check.svg?raw";
import circleX from "lucide-static/icons/circle-x.svg?raw";
import info from "lucide-static/icons/info.svg?raw";
import triangleAlert from "lucide-static/icons/triangle-alert.svg?raw";
import x from "lucide-static/icons/x.svg?raw";
//#region src/components/alert.tsx
const AlertContext = createContext({ variant: () => "default" });
function alertColors(variant) {
	return match(variant).with("default", () => ({
		container: "border-subtle bg-surface",
		title: "text-primary",
		description: "text-secondary"
	})).with("info", () => ({
		container: componentToneClass("accent", "surface"),
		title: componentToneClass("accent", "text"),
		description: "text-secondary"
	})).with("success", () => ({
		container: componentToneClass("success", "surface"),
		title: componentToneClass("success", "text"),
		description: componentToneClass("success", "text")
	})).with("warning", () => ({
		container: "border-strong bg-control",
		title: "text-primary",
		description: "text-secondary"
	})).with(P.union("error", "destructive"), () => ({
		container: componentToneClass("danger", "surface"),
		title: componentToneClass("danger", "text"),
		description: componentToneClass("danger", "text")
	})).exhaustive();
}
function alertGeometry(size) {
	return match(size).with("sm", () => "gap-2 px-3 py-2").with("default", () => "gap-3 px-4 py-2.5").with("lg", () => "gap-3 px-5 py-3").exhaustive();
}
function alertIcon(variant) {
	return match(variant).with(P.union("default", "info"), () => info).with("success", () => checkCircle).with("warning", () => triangleAlert).with(P.union("error", "destructive"), () => circleX).exhaustive();
}
/** A native status callout with shadcn-compatible compound composition. */
function Alert(props) {
	const variant = () => props.variant ?? "default";
	const size = () => props.size ?? "default";
	const colors = () => alertColors(variant());
	const forwarded = omit(props, "variant", "size", "icon", "banner", "onClose", "title", "class", "children");
	const content = () => props.title === void 0 ? props.children : [createComponent$1(AlertTitle, { get children() {
		return props.title;
	} }), memo(() => {
		return memo(() => {
			return props.children === void 0;
		})() ? null : createComponent$1(AlertDescription, { get children() {
			return props.children;
		} });
	})];
	return createComponent(AlertContext, {
		value: { variant },
		get children() {
			return createComponent$1(View, mergeProps(forwarded, {
				role: "alert",
				get "aria-label"() {
					return props["aria-label"] ?? props.title;
				},
				get "class"() {
					return mergeClasses("w-full min-w-0 flex flex-row items-start", alertGeometry(size()), componentsSurfaceClass("raised"), colors().container, props.banner && "rounded-none border-0", props.class);
				},
				get children() {
					return [
						memo(() => {
							return memo(() => {
								return props.icon === false;
							})() ? null : createComponent$1(View, {
								class: "h-5 flex-none flex items-center",
								get children() {
									return props.icon ?? createComponent$1(Icon, {
										get source() {
											return alertIcon(variant());
										},
										size: 16,
										get "class"() {
											return colors().title;
										}
									});
								}
							});
						}),
						createComponent$1(View, {
							class: "min-w-0 flex-1 flex flex-col gap-1",
							get children() {
								return content();
							}
						}),
						memo(() => {
							return memo(() => {
								return !!props.onClose;
							})() ? createComponent$1(Button, {
								size: "icon",
								variant: "ghost",
								get "aria-label"() {
									return `Dismiss ${props["aria-label"] ?? props.title ?? "alert"}`;
								},
								class: "w-7 h-7 flex-none",
								get onClick() {
									return props.onClose;
								},
								get children() {
									return createComponent$1(Icon, {
										source: x,
										size: 14
									});
								}
							}) : props.onClose;
						})
					];
				}
			}));
		}
	});
}
function AlertTitle(props) {
	const context = useContext(AlertContext);
	return createComponent$1(Text, mergeProps(props, { get "class"() {
		return mergeClasses("w-full min-w-0 text-sm font-semibold", alertColors(context.variant()).title, props.class);
	} }));
}
function AlertDescription(props) {
	const context = useContext(AlertContext);
	return createComponent$1(Text, mergeProps(props, { get "class"() {
		return mergeClasses("w-full min-w-0 whitespace-normal text-sm", alertColors(context.variant()).description, props.class);
	} }));
}
/** Recovery and acknowledgement controls belonging to an alert. */
function AlertActions(props) {
	return createComponent$1(View, mergeProps(props, {
		get role() {
			return props.role ?? "group";
		},
		get "class"() {
			return mergeClasses("w-full min-w-0 mt-2 flex flex-row flex-wrap items-center gap-2", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
//#endregion
export { Alert, AlertActions, AlertDescription, AlertTitle, alertColors };

//# sourceMappingURL=alert.mjs.map