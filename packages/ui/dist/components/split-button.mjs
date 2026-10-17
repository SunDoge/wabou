import { Icon } from "../primitives/view.mjs";
import { Button } from "./button.mjs";
import { ButtonGroup, ButtonGroupSeparator } from "./button-group.mjs";
import { DropdownMenu } from "./dropdown-menu.mjs";
import { createComponent, mergeProps } from "@wabou/core/renderer";
import chevronDown from "lucide-static/icons/chevron-down.svg?raw";
//#region src/components/split-button.tsx
/**
* Desktop split action: the label always performs the primary command while
* the adjacent arrow owns alternative commands and their keyboard behavior.
*/
function SplitButton(props) {
	const menuLabel = () => props.menuLabel ?? `${props.label} alternatives`;
	return createComponent(ButtonGroup, {
		get "aria-label"() {
			return props.label;
		},
		get "class"() {
			return props.class;
		},
		get children() {
			return [
				createComponent(Button, {
					get variant() {
						return props.variant;
					},
					get size() {
						return props.size;
					},
					get disabled() {
						return props.disabled;
					},
					get onClick() {
						return props.onClick;
					},
					class: "flex-1",
					get children() {
						return props.label;
					}
				}),
				createComponent(ButtonGroupSeparator, {}),
				createComponent(DropdownMenu, {
					get "aria-label"() {
						return menuLabel();
					},
					get items() {
						return props.items;
					},
					get onAction() {
						return props.onAction;
					},
					trigger: (trigger) => createComponent(Button, mergeProps(trigger, {
						get "aria-label"() {
							return menuLabel();
						},
						get variant() {
							return props.variant;
						},
						size: "icon",
						get disabled() {
							return props.disabled;
						},
						get children() {
							return createComponent(Icon, {
								source: chevronDown,
								size: 14
							});
						}
					}))
				})
			];
		}
	});
}
//#endregion
export { SplitButton };

//# sourceMappingURL=split-button.mjs.map