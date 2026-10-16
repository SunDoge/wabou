import { Icon, Text, View } from "../primitives/view.mjs";
import { Button } from "../primitives/button.mjs";
import { Table, TableBody, TableCell, TableHeader, TableRow } from "./table.mjs";
import { For, Show } from "solid-js";
import { createComponent as createComponent$1, memo } from "@wabou/core/renderer";
import arrowDown from "lucide-static/icons/arrow-down.svg?raw";
import arrowUp from "lucide-static/icons/arrow-up.svg?raw";
import arrowUpDown from "lucide-static/icons/arrow-up-down.svg?raw";
//#region src/components/data-table.tsx
function TableSortIndicator(props) {
	const direction = () => typeof props.direction === "function" ? props.direction() : props.direction;
	return createComponent$1(Icon, {
		get source() {
			return memo(() => {
				return direction() === "asc";
			})() ? arrowUp : direction() === "desc" ? arrowDown : arrowUpDown;
		},
		size: 12,
		get "class"() {
			return direction() ? "flex-none text-secondary" : "flex-none text-muted";
		}
	});
}
/** Shadcn-style table anatomy backed by the framework-agnostic TanStack core. */
function DataTable(props) {
	return createComponent$1(Table, {
		get "aria-label"() {
			return props["aria-label"];
		},
		get children() {
			return [createComponent$1(TableHeader, { get children() {
				return createComponent$1(For, {
					get each() {
						return props.model.table.getHeaderGroups();
					},
					children: (headerGroup) => createComponent$1(TableRow, {
						class: "h-11 border-strong bg-control",
						get children() {
							return createComponent$1(For, {
								get each() {
									return headerGroup.headers;
								},
								children: (header) => {
									const sortable = () => header.column.getCanSort();
									const direction = () => props.model.sorting().find(({ id }) => id === header.column.id)?.desc ? "desc" : props.model.sorting().some(({ id }) => id === header.column.id) ? "asc" : false;
									const label = () => String(header.column.columnDef.header ?? header.id);
									const accessibleLabel = () => {
										const sorted = direction();
										return sorted ? `${label()}, sorted ${sorted === "asc" ? "ascending" : "descending"}` : sortable() ? `Sort by ${label()}` : label();
									};
									return createComponent$1(Button, {
										unstyled: true,
										role: "columnheader",
										get "aria-label"() {
											return accessibleLabel();
										},
										class: "flex-1 min-w-0 px-4 justify-start gap-2 text-xs font-semibold text-secondary",
										get disabled() {
											return !sortable();
										},
										onClick: () => header.column.toggleSorting(),
										get children() {
											return [memo(() => {
												return label();
											}), createComponent$1(Show, {
												get when() {
													return sortable();
												},
												get children() {
													return createComponent$1(TableSortIndicator, { direction });
												}
											})];
										}
									});
								}
							});
						}
					})
				});
			} }), createComponent$1(TableBody, { get children() {
				return [createComponent$1(For, {
					get each() {
						return props.model.rows();
					},
					children: (row) => {
						const cells = () => createComponent$1(For, {
							get each() {
								return row.getVisibleCells();
							},
							children: (cell) => createComponent$1(TableCell, {
								class: "px-4",
								get children() {
									return props.renderCell?.({
										value: cell.getValue(),
										columnId: cell.column.id,
										row
									}) ?? createComponent$1(Text, {
										class: "w-full truncate text-sm text-primary",
										get children() {
											return String(cell.getValue() ?? "");
										}
									});
								}
							})
						});
						return createComponent$1(Show, {
							get when() {
								return props.selectable;
							},
							get fallback() {
								return createComponent$1(TableRow, {
									get "aria-label"() {
										return `Row ${row.id}`;
									},
									get children() {
										return cells();
									}
								});
							},
							get children() {
								return createComponent$1(Button, {
									unstyled: true,
									role: "row",
									get "aria-label"() {
										return `Select row ${row.id}`;
									},
									get "aria-selected"() {
										return props.model.rowSelection()[row.id] === true;
									},
									get selected() {
										return props.model.rowSelection()[row.id] === true;
									},
									get "class"() {
										return props.model.rowSelection()[row.id] === true ? "h-12 w-full flex border-b border-subtle bg-selected" : "h-12 w-full flex border-b border-subtle bg-surface";
									},
									onClick: () => row.toggleSelected(),
									get children() {
										return cells();
									}
								});
							}
						});
					}
				}), createComponent$1(Show, {
					get when() {
						return props.model.rows().length === 0;
					},
					get children() {
						return createComponent$1(View, {
							class: "h-24 flex items-center justify-center",
							get children() {
								return createComponent$1(Text, {
									class: "text-sm text-muted",
									get children() {
										return props.emptyMessage ?? "No results";
									}
								});
							}
						});
					}
				})];
			} })];
		}
	});
}
//#endregion
export { DataTable, TableSortIndicator };

//# sourceMappingURL=data-table.mjs.map