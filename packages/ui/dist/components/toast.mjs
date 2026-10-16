import { Icon, Text, View } from "../primitives/view.mjs";
import { NotificationRegion, createNotifications } from "../primitives/notification.mjs";
import { componentsElevation, componentsSurfaceClass, useComponentsTheme } from "./theme.mjs";
import { Button } from "./button.mjs";
import { mergeClasses } from "@wabou/core/style";
import { createComponent, memo } from "@wabou/core/renderer";
import { match } from "ts-pattern";
import checkCircle from "lucide-static/icons/circle-check.svg?raw";
import info from "lucide-static/icons/info.svg?raw";
import triangleAlert from "lucide-static/icons/triangle-alert.svg?raw";
import x from "lucide-static/icons/x.svg?raw";
//#region src/components/toast.tsx
const treatment = (variant) => match(variant).with("default", () => ({
	icon: info,
	color: "text-accent"
})).with("success", () => ({
	icon: checkCircle,
	color: "text-success-primary"
})).with("warning", () => ({
	icon: triangleAlert,
	color: "text-accent"
})).with("destructive", () => ({
	icon: triangleAlert,
	color: "text-danger-primary"
})).exhaustive();
function ToastContent(props) {
	const theme = useComponentsTheme();
	const style = () => treatment(props.input.variant ?? "default");
	return createComponent(View, {
		get "class"() {
			return mergeClasses("w-full min-w-0 flex items-start gap-3 px-4 py-3.5", componentsSurfaceClass("floating"));
		},
		get shadows() {
			return componentsElevation(theme(), "floating");
		},
		get children() {
			return [
				createComponent(Icon, {
					get source() {
						return style().icon;
					},
					get "class"() {
						return mergeClasses("flex-none mt-0.5", style().color);
					},
					size: 18
				}),
				createComponent(View, {
					class: "min-w-0 flex-1 flex flex-col gap-1",
					get children() {
						return [
							createComponent(Text, {
								class: "text-sm font-medium text-primary",
								get children() {
									return props.input.title;
								}
							}),
							memo(() => {
								return memo(() => {
									return !!props.input.description;
								})() ? createComponent(Text, {
									class: "text-sm whitespace-normal text-muted",
									get children() {
										return props.input.description;
									}
								}) : props.input.description;
							}),
							memo(() => {
								return memo(() => {
									return !!props.input.action;
								})() ? createComponent(View, {
									class: "pt-1 flex items-center",
									get children() {
										return createComponent(Button, {
											size: "sm",
											variant: "outline",
											get "aria-label"() {
												return props.input.action.label;
											},
											onClick: () => {
												props.input.action?.onAction();
												if (props.input.action?.dismiss !== false) props.dismiss();
											},
											get children() {
												return props.input.action.label;
											}
										});
									}
								}) : props.input.action;
							})
						];
					}
				}),
				createComponent(Button, {
					size: "icon",
					variant: "ghost",
					get "aria-label"() {
						return `Dismiss ${props.input.title}`;
					},
					class: "w-7 h-7 flex-none",
					get onClick() {
						return props.dismiss;
					},
					get children() {
						return createComponent(Icon, {
							source: x,
							size: 14
						});
					}
				})
			];
		}
	});
}
/** Create an owner-scoped, styled toast queue over NotificationRegion. */
function createToasts(options = {}) {
	const notifications = createNotifications(options);
	const show = (input) => notifications.show({
		"aria-label": input.title,
		priority: input.variant === "destructive" ? "assertive" : "polite",
		duration: input.duration,
		onDismiss: input.onDismiss,
		content: (controls) => createComponent(ToastContent, {
			input,
			get dismiss() {
				return controls.dismiss;
			}
		})
	});
	return {
		notifications,
		show,
		success: (title, input = {}) => show({
			...input,
			title,
			variant: "success"
		}),
		warning: (title, input = {}) => show({
			...input,
			title,
			variant: "warning"
		}),
		error: (title, input = {}) => show({
			...input,
			title,
			variant: "destructive"
		}),
		dismiss: (id) => notifications.dismiss(id, "dismiss"),
		clear: notifications.clear
	};
}
const defaultToastMotion = (placement) => ({
	duration: .18,
	ease: "easeOut",
	fromY: placement.startsWith("top") ? -12 : 12
});
/** Render a non-blocking stack of styled toasts on the floating plane. */
function Toaster(props) {
	return createComponent(NotificationRegion, {
		get notifications() {
			return props.toasts.notifications;
		},
		get placement() {
			return props.placement ?? "bottom-end";
		},
		get "class"() {
			return props.class;
		},
		stackClass: "w-96 max-w-full",
		get itemClass() {
			return mergeClasses("w-full max-w-full", props.itemClass);
		},
		get motion() {
			return memo(() => {
				return props.motion === void 0;
			})() ? defaultToastMotion(props.placement ?? "bottom-end") : props.motion;
		}
	});
}
//#endregion
export { Toaster, createToasts };

//# sourceMappingURL=toast.mjs.map