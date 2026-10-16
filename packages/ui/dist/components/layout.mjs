import { View } from "../primitives/view.mjs";
import { createMeasuredSize } from "../primitives/measure.mjs";
import { Dialog } from "./dialog.mjs";
import { mergeClasses } from "@wabou/core/style";
import { For, Show, createContext, createEffect, createMemo, omit, useContext } from "solid-js";
import { applyRef, createComponent as createComponent$1, mergeProps } from "@wabou/core/renderer";
import { match } from "ts-pattern";
//#region src/components/layout.tsx
const ResponsiveGridContext = createContext();
/** Read the completed native size and active column count of the nearest grid. */
function useResponsiveGrid() {
	const context = useContext(ResponsiveGridContext);
	if (!context) throw new Error("useResponsiveGrid must be used inside ResponsiveGrid");
	return context;
}
const responsiveGridColumnClass = (columns) => match(columns).with(1, () => "grid-cols-1").with(2, () => "grid-cols-2").with(3, () => "grid-cols-3").with(4, () => "grid-cols-4").exhaustive();
function responsiveGridColumnCount(options) {
	const maxColumns = options.maxColumns ?? 4;
	if (!Number.isFinite(options.width) || options.width <= 0) return Math.min(options.initialColumns ?? 1, maxColumns);
	const gap = Math.max(0, options.gap ?? 16);
	const minColumnWidth = Math.max(1, options.minColumnWidth);
	const columns = Math.min(maxColumns, Math.max(1, Math.floor((options.width + gap) / (minColumnWidth + gap))));
	const itemCount = Math.max(0, Math.floor(options.itemCount ?? 0));
	if (options.balanceLastRow && itemCount > 1 && columns > 1 && itemCount % columns === 1) return columns - 1;
	return columns;
}
function responsiveGridRemainderCount(itemCount, columns) {
	const remainder = Math.max(0, Math.floor(itemCount)) % columns;
	return remainder === 0 ? 0 : columns - remainder;
}
/**
* A grid that responds to its own native content box instead of the window.
*
* This is important inside sidebars, split panes and dialogs: window media
* queries do not know how much width the component actually receives.
*/
function ResponsiveGrid(props) {
	const measured = createMeasuredSize();
	const columns = createMemo(() => responsiveGridColumnCount({
		width: measured.width(),
		minColumnWidth: props.minColumnWidth,
		gap: props.gap,
		maxColumns: props.maxColumns,
		initialColumns: props.initialColumns,
		itemCount: props.itemCount,
		balanceLastRow: props.balanceLastRow
	}));
	const rest = omit(props, "children", "minColumnWidth", "gap", "maxColumns", "initialColumns", "itemCount", "balanceLastRow", "class", "ref");
	const state = {
		columns,
		width: measured.width,
		height: measured.height
	};
	return createComponent$1(ResponsiveGridContext, {
		value: state,
		get children() {
			return createComponent$1(View, mergeProps(rest, {
				ref: (node) => {
					measured.ref(node);
					props.ref?.(node);
				},
				get style() {
					return {
						gap: props.gap ?? 16,
						...props.style
					};
				},
				get "class"() {
					return mergeClasses("w-full min-w-0 grid", responsiveGridColumnClass(columns()), props.class);
				},
				get children() {
					return props.children;
				}
			}));
		}
	});
}
/** Fill the unused cells in the final row using the grid's measured columns. */
function ResponsiveGridRemainder(props) {
	const context = useResponsiveGrid();
	const cells = createMemo(() => Array.from({ length: responsiveGridRemainderCount(props.itemCount, context.columns()) }));
	return createComponent$1(For, {
		get each() {
			return cells();
		},
		children: () => createComponent$1(View, {
			"aria-hidden": true,
			get "class"() {
				return mergeClasses("min-w-0", props.class);
			}
		})
	});
}
/**
* A horizontal primary/aside boundary with explicit flex shrink semantics.
* Use `SplitPaneMain` for the elastic region and `SplitPaneAside` for a
* class-sized fixed rail. Both regions clip at their own boundary, so content
* cannot paint across the divider or a rounded parent clip.
*/
function SplitPane(props) {
	return createComponent$1(View, {
		get "class"() {
			return mergeClasses("w-full min-w-0 flex flex-row overflow-hidden", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function SplitPaneMain(props) {
	return createComponent$1(View, {
		get "class"() {
			return mergeClasses("flex-1 min-w-0 overflow-hidden", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function SplitPaneAside(props) {
	return createComponent$1(View, {
		get "class"() {
			return mergeClasses("flex-none min-w-0 overflow-hidden", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
const AdaptiveSplitPaneContext = createContext();
/**
* Master/detail layout whose detail region can move from an inline rail to a
* modal surface without changing the application's selection model.
*/
function AdaptiveSplitPane(props) {
	const measured = createMeasuredSize();
	const compact = () => props.compact ?? (measured.measured() && measured.width() <= (props.compactAt ?? 720));
	createEffect(() => props.compact !== void 0 || measured.measured() ? compact() : void 0, (value) => {
		if (value !== void 0) props.onCompactChange?.(value);
	});
	return createComponent$1(AdaptiveSplitPaneContext, {
		value: { compact },
		get children() {
			return createComponent$1(View, {
				ref(r$) {
					var _ref$ = measured.ref;
					typeof _ref$ === "function" || Array.isArray(_ref$) ? applyRef(_ref$, r$) : measured.ref = r$;
				},
				role: "group",
				get "aria-label"() {
					return props["aria-label"];
				},
				class: "w-full h-full min-w-0 min-h-0",
				get children() {
					return createComponent$1(SplitPane, {
						get "class"() {
							return mergeClasses("h-full", props.class);
						},
						get children() {
							return props.children;
						}
					});
				}
			});
		}
	});
}
function AdaptiveSplitPaneMain(props) {
	return createComponent$1(SplitPaneMain, {
		get "class"() {
			return props.class;
		},
		get children() {
			return props.children;
		}
	});
}
function AdaptiveSplitPaneDetail(props) {
	const context = useContext(AdaptiveSplitPaneContext);
	return createComponent$1(Show, {
		get when() {
			return context.compact();
		},
		get fallback() {
			return createComponent$1(SplitPaneAside, {
				get "class"() {
					return props.class;
				},
				get children() {
					return props.children;
				}
			});
		},
		get children() {
			return createComponent$1(Dialog, {
				get open() {
					return props.open;
				},
				onOpenChange: (open) => props.onOpenChange(open),
				get "aria-label"() {
					return props["aria-label"];
				},
				get contentClass() {
					return mergeClasses("h-11/12 p-0 overflow-hidden", props.modalClass);
				},
				get children() {
					return props.children;
				}
			});
		}
	});
}
//#endregion
export { AdaptiveSplitPane, AdaptiveSplitPaneDetail, AdaptiveSplitPaneMain, ResponsiveGrid, ResponsiveGridRemainder, SplitPane, SplitPaneAside, SplitPaneMain, responsiveGridColumnCount, responsiveGridRemainderCount, useResponsiveGrid };

//# sourceMappingURL=layout.mjs.map