import { Text, View } from "../primitives/view.mjs";
import { componentsDisabledInteractiveClass } from "./theme.mjs";
import { mergeClasses } from "@wabou/core/style";
import { omit } from "solid-js";
import { createComponent as createComponent$1, memo, mergeProps } from "@wabou/core/renderer";
//#region src/components/settings.tsx
/**
* One settings row with stable explanatory and control regions.
* The row blocks its complete subtree when disabled; controls should also
* receive `disabled` when they need to expose that state independently.
*/
function SettingsItem(props) {
	const orientation = () => props.orientation ?? "horizontal";
	const rest = omit(props, "title", "description", "children", "orientation", "disabled", "class", "labelClass", "controlClass", "style");
	return createComponent$1(View, mergeProps(rest, {
		get role() {
			return props.role ?? "group";
		},
		get "aria-label"() {
			return props["aria-label"] ?? props.title;
		},
		get "aria-orientation"() {
			return orientation();
		},
		get "aria-disabled"() {
			return props.disabled;
		},
		get interactionBlocked() {
			return props.disabled || props.interactionBlocked;
		},
		get "class"() {
			return mergeClasses("w-full min-w-0 flex", orientation() === "horizontal" ? "flex-row items-start justify-between gap-6" : "flex-col items-stretch gap-3", props.class, componentsDisabledInteractiveClass(props.disabled ?? false));
		},
		get style() {
			return props.style;
		},
		get children() {
			return [createComponent$1(View, {
				get "class"() {
					return mergeClasses("min-w-0 flex-1 flex flex-col gap-1", props.labelClass);
				},
				get children() {
					return [createComponent$1(Text, {
						class: "min-w-0 whitespace-normal text-sm font-medium text-primary",
						get children() {
							return props.title;
						}
					}), memo(() => {
						return memo(() => {
							return !!(props.description !== void 0 && props.description !== null);
						})() ? createComponent$1(Text, {
							class: "min-w-0 whitespace-normal text-sm text-secondary",
							get children() {
								return props.description;
							}
						}) : null;
					})];
				}
			}), createComponent$1(View, {
				get "class"() {
					return mergeClasses("min-w-0 flex items-center gap-2", orientation() === "horizontal" ? "flex-none justify-end" : "w-full justify-start", props.controlClass);
				},
				get children() {
					return props.children;
				}
			})];
		}
	}));
}
/**
* A settings-page section with one explanatory label and one control surface.
* The page owns the responsive breakpoint and passes `stacked`; the component
* owns the repeated alignment, spacing, and surface contract.
*/
function SettingsSection(props) {
	return createComponent$1(View, {
		role: "group",
		get "aria-label"() {
			return props.title;
		},
		get "class"() {
			return mergeClasses("w-full min-w-0 flex items-start", props.stacked ? "flex-col gap-3" : "flex-row gap-7", props.class);
		},
		get children() {
			return [createComponent$1(View, {
				get "class"() {
					return mergeClasses("flex-none flex flex-col gap-1 pt-1", props.stacked ? "w-full" : "w-52");
				},
				get children() {
					return [createComponent$1(Text, {
						role: "heading",
						class: "text-base font-semibold text-primary",
						get children() {
							return props.title;
						}
					}), memo(() => {
						return memo(() => {
							return !!props.description;
						})() ? createComponent$1(Text, {
							class: "text-sm text-secondary whitespace-normal",
							get children() {
								return props.description;
							}
						}) : null;
					})];
				}
			}), createComponent$1(View, {
				get "class"() {
					return mergeClasses("w-full min-w-0 flex-1 flex flex-col gap-5 rounded-xl border border-subtle bg-surface px-5 py-5 shadow-xs", props.contentClass);
				},
				get children() {
					return props.children;
				}
			})];
		}
	});
}
/** A titled field group inside a SettingsSection control surface. */
function SettingsGroup(props) {
	return createComponent$1(View, {
		role: "group",
		get "aria-label"() {
			return props.title;
		},
		get "class"() {
			return mergeClasses("min-w-0 flex flex-col gap-4", props.class);
		},
		get children() {
			return [createComponent$1(View, {
				class: "min-w-0 flex flex-col gap-1",
				get children() {
					return [createComponent$1(Text, {
						class: "text-sm font-semibold text-primary",
						get children() {
							return props.title;
						}
					}), memo(() => {
						return memo(() => {
							return !!props.description;
						})() ? createComponent$1(Text, {
							class: "text-sm text-secondary whitespace-normal",
							get children() {
								return props.description;
							}
						}) : null;
					})];
				}
			}), createComponent$1(View, {
				class: "min-w-0 flex flex-col gap-4",
				get children() {
					return props.children;
				}
			})];
		}
	});
}
//#endregion
export { SettingsGroup, SettingsItem, SettingsSection };

//# sourceMappingURL=settings.mjs.map