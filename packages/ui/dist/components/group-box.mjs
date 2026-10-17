import { Text, View } from "../primitives/view.mjs";
import { mergeClasses } from "@wabou/core/style";
import { Show, omit } from "solid-js";
import { createComponent as createComponent$1, memo, mergeProps } from "@wabou/core/renderer";
import { match } from "ts-pattern";
//#region src/components/group-box.tsx
function groupBoxContentClass(variant = "normal", className) {
	return mergeClasses("w-full min-w-0 flex flex-col gap-4 rounded-lg", match(variant).with("normal", () => "bg-transparent").with("fill", () => "p-4 bg-control").with("outline", () => "p-4 border border-subtle bg-transparent").exhaustive(), className);
}
/** A lightweight titled surface for related controls and settings rows. */
function GroupBox(props) {
	const variant = () => props.variant ?? "normal";
	const label = () => props["aria-label"] ?? (typeof props.title === "string" ? props.title : void 0);
	const rest = omit(props, "title", "description", "children", "variant", "class", "headerClass", "contentClass");
	return createComponent$1(View, mergeProps(rest, {
		get role() {
			return props.role ?? "group";
		},
		get "aria-label"() {
			return label();
		},
		get "class"() {
			return mergeClasses("w-full min-w-0 flex flex-col", variant() === "normal" ? "gap-4" : "gap-3", props.class);
		},
		get children() {
			return [createComponent$1(Show, {
				get when() {
					return memo(() => {
						return props.title !== void 0;
					})() && props.title !== null || props.description !== void 0 && props.description !== null;
				},
				get children() {
					return createComponent$1(View, {
						get "class"() {
							return mergeClasses("w-full min-w-0 flex flex-col gap-1", props.headerClass);
						},
						get children() {
							return [createComponent$1(Show, {
								get when() {
									return memo(() => {
										return props.title !== void 0;
									})() && props.title !== null;
								},
								get children() {
									return createComponent$1(Text, {
										role: "heading",
										class: "min-w-0 text-sm font-semibold text-primary",
										get children() {
											return props.title;
										}
									});
								}
							}), createComponent$1(Show, {
								get when() {
									return memo(() => {
										return props.description !== void 0;
									})() && props.description !== null;
								},
								get children() {
									return createComponent$1(Text, {
										class: "w-full min-w-0 whitespace-normal text-sm text-muted",
										get children() {
											return props.description;
										}
									});
								}
							})];
						}
					});
				}
			}), createComponent$1(View, {
				get "class"() {
					return groupBoxContentClass(variant(), props.contentClass);
				},
				get children() {
					return props.children;
				}
			})];
		}
	}));
}
//#endregion
export { GroupBox, groupBoxContentClass };

//# sourceMappingURL=group-box.mjs.map