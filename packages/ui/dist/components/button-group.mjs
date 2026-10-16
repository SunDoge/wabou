import { Text, View } from "../primitives/view.mjs";
import { ButtonGroupContext, buttonGroupItemCorners, createButtonGroupContext, useButtonGroupItem } from "./button-group-context.mjs";
import { componentsControlContentSize, componentsDisabledInteractiveClass } from "./theme.mjs";
import { mergeClasses } from "@wabou/core/style";
import { createComponent, omit } from "solid-js";
import { createComponent as createComponent$1, mergeProps } from "@wabou/core/renderer";
import { match } from "ts-pattern";
//#region src/components/button-group.tsx
/** A single bordered control surface composed from ordinary Wabou buttons. */
function ButtonGroup(props) {
	const orientation = () => props.orientation ?? "horizontal";
	const size = () => props.size ?? "default";
	const variant = () => props.variant ?? "default";
	const disabled = () => props.disabled ?? false;
	const layout = () => match(orientation()).with("horizontal", () => "flex-row items-stretch").with("vertical", () => "flex-col items-stretch").exhaustive();
	const context = createButtonGroupContext(orientation, {
		size,
		variant,
		disabled
	});
	const forwarded = omit(props, "orientation", "size", "variant", "disabled", "class", "children");
	return createComponent(ButtonGroupContext, {
		value: context,
		get children() {
			return createComponent$1(View, mergeProps(forwarded, {
				role: "group",
				get "aria-label"() {
					return props["aria-label"];
				},
				get "aria-orientation"() {
					return orientation();
				},
				get "aria-disabled"() {
					return disabled();
				},
				get "class"() {
					return mergeClasses("min-w-0 flex gap-0 rounded-lg border border-strong bg-surface shadow-xs", layout(), props.class);
				},
				get children() {
					return props.children;
				}
			}));
		}
	});
}
function ButtonGroupText(props) {
	const groupItem = useButtonGroupItem();
	const size = () => groupItem?.size() ?? "default";
	return createComponent$1(Text, mergeProps(props, {
		get "class"() {
			return mergeClasses("flex-none flex items-center whitespace-nowrap font-medium text-secondary bg-control", componentsControlContentSize(size()), groupItem?.disabled() && "bg-surface-muted text-muted", groupItem && buttonGroupItemCorners(groupItem), props.class, componentsDisabledInteractiveClass(groupItem?.disabled() ?? false));
		},
		get children() {
			return props.children;
		}
	}));
}
function ButtonGroupSeparator(props) {
	return createComponent$1(View, {
		role: "separator",
		"aria-hidden": "true",
		get "class"() {
			return mergeClasses("flex-none self-stretch bg-strong", match(props.orientation ?? "vertical").with("vertical", () => "w-px min-h-full").with("horizontal", () => "h-px min-w-full").exhaustive(), props.class);
		}
	});
}
//#endregion
export { ButtonGroup, ButtonGroupSeparator, ButtonGroupText };

//# sourceMappingURL=button-group.mjs.map