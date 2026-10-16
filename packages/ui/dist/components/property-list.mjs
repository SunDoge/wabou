import { Text, View } from "../primitives/view.mjs";
import { mergeClasses } from "@wabou/core/style";
import { createComponent, mergeProps } from "@wabou/core/renderer";
//#region src/components/property-list.tsx
function PropertyList(props) {
	return createComponent(View, mergeProps(props, {
		role: "table",
		get "class"() {
			return mergeClasses("w-full min-w-0 flex flex-col overflow-hidden rounded-xl border border-subtle bg-surface", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
/** Compact two-column data intended for inspectors and settings summaries. */
function PropertyRow(props) {
	return createComponent(View, {
		role: "row",
		get "aria-label"() {
			return props.name;
		},
		get "class"() {
			return mergeClasses("min-h-10 w-full min-w-0 px-3 py-2 flex flex-row items-center gap-3 border-b border-subtle", props.class);
		},
		get children() {
			return [createComponent(Text, {
				role: "cell",
				get "class"() {
					return mergeClasses("w-48 flex-none text-xs font-mono text-accent whitespace-nowrap", props.nameClass);
				},
				get children() {
					return props.name;
				}
			}), createComponent(Text, {
				role: "cell",
				get "class"() {
					return mergeClasses("min-w-0 flex-1 text-xs text-muted whitespace-normal", props.valueClass);
				},
				get children() {
					return props.value;
				}
			})];
		}
	});
}
//#endregion
export { PropertyList, PropertyRow };

//# sourceMappingURL=property-list.mjs.map