import { Icon, Text, View } from "../primitives/view.mjs";
import { Button } from "./button.mjs";
import { ContentState } from "./content-state.mjs";
import { workbenchClass, workbenchContentClass, workbenchContentColumnClass, workbenchFooterClass, workbenchHeaderClass, workbenchInspectorClass, workbenchInspectorContentClass, workbenchInspectorHeaderClass, workbenchMainClass, workbenchSidebarClass } from "./workbench-style.mjs";
import { Sidebar } from "./sidebar.mjs";
import { omit } from "solid-js";
import { createComponent as createComponent$1, memo, mergeProps } from "@wabou/core/renderer";
import x from "lucide-static/icons/x.svg?raw";
//#region src/components/workbench.tsx
/** Full-window desktop application boundary with explicit shrink semantics. */
function Workbench(props) {
	const forwarded = omit(props, "class");
	return createComponent$1(View, mergeProps(forwarded, { get "class"() {
		return workbenchClass(props.class);
	} }));
}
/** Fixed-width navigation rail paired with a {@link WorkbenchMain}. */
function WorkbenchSidebar(props) {
	const forwarded = omit(props, "class");
	return createComponent$1(Sidebar, mergeProps(forwarded, { get "class"() {
		return workbenchSidebarClass(props.class);
	} }));
}
/** The resizable application column beside the navigation rail. */
function WorkbenchMain(props) {
	const forwarded = omit(props, "class");
	return createComponent$1(View, mergeProps(forwarded, { get "class"() {
		return workbenchMainClass(props.class);
	} }));
}
/** Shared 48px chrome row for both sidebar and content headers. */
function WorkbenchHeader(props) {
	const forwarded = omit(props, "class");
	return createComponent$1(View, mergeProps(forwarded, { get "class"() {
		return workbenchHeaderClass(props.class);
	} }));
}
/** Bounded application content. Add a ScrollArea inside when scrolling is needed. */
function WorkbenchContent(props) {
	const forwarded = omit(props, "class");
	return createComponent$1(View, mergeProps(forwarded, { get "class"() {
		return workbenchContentClass(props.class);
	} }));
}
/** A centered 896px desktop content column that still shrinks with its pane. */
function WorkbenchContentColumn(props) {
	const forwarded = omit(props, "class");
	return createComponent$1(View, mergeProps(forwarded, { get "class"() {
		return workbenchContentColumnClass(props.class);
	} }));
}
/** Fixed chrome below the workbench content, such as a composer or status bar. */
function WorkbenchFooter(props) {
	const forwarded = omit(props, "class");
	return createComponent$1(View, mergeProps(forwarded, { get "class"() {
		return workbenchFooterClass(props.class);
	} }));
}
/** Fixed-width auxiliary pane for file previews, diffs and contextual tools. */
function WorkbenchInspector(props) {
	const forwarded = omit(props, "class");
	return createComponent$1(View, mergeProps(forwarded, { get "class"() {
		return workbenchInspectorClass(props.class);
	} }));
}
/** Inspector title row with a stable height and bounded children. */
function WorkbenchInspectorHeader(props) {
	const forwarded = omit(props, "class");
	return createComponent$1(View, mergeProps(forwarded, { get "class"() {
		return workbenchInspectorHeaderClass(props.class);
	} }));
}
/**
* Consistent inspector chrome with a bounded title, optional description and
* correctly sized trailing actions.
*/
function WorkbenchInspectorTitlebar(props) {
	const forwarded = omit(props, "title", "description", "closeLabel", "onClose", "renderAction", "class");
	return createComponent$1(WorkbenchInspectorHeader, mergeProps(forwarded, {
		get "class"() {
			return props.class;
		},
		get children() {
			return [createComponent$1(View, {
				class: "min-w-0 flex-1 flex flex-col",
				get children() {
					return [createComponent$1(Text, {
						class: "w-full min-w-0 truncate font-semibold",
						get children() {
							return props.title;
						}
					}), memo(() => {
						return memo(() => {
							return props.description === void 0;
						})() ? null : createComponent$1(Text, {
							class: "w-full min-w-0 truncate text-xs text-muted",
							get children() {
								return props.description;
							}
						});
					})];
				}
			}), createComponent$1(View, {
				class: "flex-none flex flex-row items-center gap-1",
				get children() {
					return [memo(() => {
						return props.renderAction?.();
					}), memo(() => {
						return memo(() => {
							return !!props.onClose;
						})() ? createComponent$1(Button, {
							variant: "ghost",
							size: "icon",
							get "aria-label"() {
								return props.closeLabel ?? "Close inspector";
							},
							get onClick() {
								return props.onClose;
							},
							get children() {
								return createComponent$1(Icon, {
									source: x,
									size: 15
								});
							}
						}) : null;
					})];
				}
			})];
		}
	}));
}
/** Flexible, clipped inspector body. Add a ScrollArea inside when needed. */
function WorkbenchInspectorContent(props) {
	const forwarded = omit(props, "class");
	return createComponent$1(View, mergeProps(forwarded, { get "class"() {
		return workbenchInspectorContentClass(props.class);
	} }));
}
/** Mutually exclusive centered state for a bounded inspector body. */
function WorkbenchInspectorState(props) {
	return createComponent$1(ContentState, props);
}
//#endregion
export { Workbench, WorkbenchContent, WorkbenchContentColumn, WorkbenchFooter, WorkbenchHeader, WorkbenchInspector, WorkbenchInspectorContent, WorkbenchInspectorHeader, WorkbenchInspectorState, WorkbenchInspectorTitlebar, WorkbenchMain, WorkbenchSidebar, workbenchClass, workbenchContentClass, workbenchContentColumnClass, workbenchFooterClass, workbenchHeaderClass, workbenchInspectorClass, workbenchInspectorContentClass, workbenchInspectorHeaderClass, workbenchMainClass, workbenchSidebarClass };

//# sourceMappingURL=workbench.mjs.map