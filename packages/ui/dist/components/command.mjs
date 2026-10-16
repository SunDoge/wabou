import { Text, View } from "../primitives/view.mjs";
import { Button } from "./button.mjs";
import { moveMenuHighlight } from "./menu-state.mjs";
import { filterCommandItems, reconcileCommandHighlight } from "./command-state.mjs";
import { Input } from "./input.mjs";
import { pickerOptionClass } from "./select-semantics.mjs";
import { mergeClasses } from "@wabou/core/style";
import { For, Match, Switch, createEffect, createMemo, createSignal } from "solid-js";
import { applyRef, createComponent as createComponent$1, memo } from "@wabou/core/renderer";
import { match } from "ts-pattern";
//#region src/components/command.tsx
/** Reusable command-result surface for search fields and inline completions. */
function CommandList(props) {
	return createComponent$1(View, {
		role: "listbox",
		get "aria-label"() {
			return props["aria-label"];
		},
		get "aria-activedescendant"() {
			return props.highlighted;
		},
		get "class"() {
			return mergeClasses("min-w-0 flex flex-col gap-1", props.class);
		},
		get children() {
			return createComponent$1(Switch, { get children() {
				return [
					createComponent$1(Match, {
						get when() {
							return props.loading;
						},
						get children() {
							return createComponent$1(Text, {
								role: "status",
								class: "px-3 py-4 text-sm text-muted text-center",
								get children() {
									return props.loadingText ?? "Loading results…";
								}
							});
						}
					}),
					createComponent$1(Match, {
						get when() {
							return memo(() => {
								return props.error !== void 0;
							})() && props.error !== null;
						},
						get children() {
							return createComponent$1(View, {
								role: "alert",
								get "aria-label"() {
									return props.errorText ?? "Could not load results";
								},
								class: "px-3 py-3 flex flex-col items-center gap-2 text-center",
								get children() {
									return [
										createComponent$1(Text, {
											class: "text-sm font-medium text-danger-primary",
											get children() {
												return props.errorText ?? "Could not load results";
											}
										}),
										createComponent$1(Text, {
											class: "max-w-full truncate text-xs text-danger-primary",
											get children() {
												return String(props.error);
											}
										}),
										memo(() => {
											return memo(() => {
												return !!props.onRetry;
											})() ? createComponent$1(Button, {
												size: "sm",
												variant: "outline",
												get "aria-label"() {
													return props.retryLabel ?? "Try again";
												},
												get onClick() {
													return props.onRetry;
												},
												get children() {
													return props.retryLabel ?? "Try again";
												}
											}) : props.onRetry;
										})
									];
								}
							});
						}
					}),
					createComponent$1(Match, {
						get when() {
							return props.items.length === 0;
						},
						get children() {
							return createComponent$1(Text, {
								role: "status",
								class: "px-3 py-4 text-sm text-muted text-center",
								get children() {
									return props.emptyText ?? "No results found.";
								}
							});
						}
					}),
					createComponent$1(Match, {
						when: true,
						get children() {
							return createComponent$1(For, {
								get each() {
									return props.items;
								},
								keyed: false,
								children: (item) => createComponent$1(View, {
									get id() {
										return item().id;
									},
									role: "option",
									get "aria-label"() {
										return item().label;
									},
									get "aria-selected"() {
										return props.highlighted === item().id;
									},
									get "aria-disabled"() {
										return item().disabled;
									},
									get "class"() {
										return mergeClasses("min-h-8 px-2 py-1 flex flex-row items-center gap-2 rounded-md", pickerOptionClass(item().disabled ?? false, props.highlighted === item().id), props.itemClass);
									},
									onPointerMove: () => !item().disabled && props.onHighlightChange?.(item().id),
									onClick: () => !item().disabled && props.onAction?.(item().id),
									get children() {
										return [
											memo(() => {
												return props.renderLeading?.(item());
											}),
											createComponent$1(View, {
												class: "min-w-0 flex-1 flex flex-col justify-center",
												get children() {
													return [createComponent$1(Text, {
														class: "min-w-0 truncate text-sm",
														get children() {
															return item().label;
														}
													}), memo(() => {
														return memo(() => {
															return !!item().description;
														})() ? createComponent$1(Text, {
															class: "min-w-0 truncate text-xs text-muted",
															get children() {
																return item().description;
															}
														}) : item().description;
													})];
												}
											}),
											memo(() => {
												return memo(() => {
													return !!item().shortcut;
												})() ? createComponent$1(Text, {
													"aria-hidden": "true",
													class: "flex-none rounded border border-subtle bg-surface px-1.5 py-0.5 text-xs text-muted",
													get children() {
														return item().shortcut;
													}
												}) : item().shortcut;
											})
										];
									}
								})
							});
						}
					})
				];
			} });
		}
	});
}
/**
* Shared command-list navigation for any focus owner, including native editors.
* The caller forwards key events while the list remains a passive popup.
*/
function createCommandListNavigation(items, options = {}) {
	const [highlighted, setHighlighted] = createSignal();
	createEffect(() => ({
		items: items(),
		highlighted: highlighted()
	}), ({ items: nextItems, highlighted: current }) => {
		setHighlighted(reconcileCommandHighlight(nextItems, current));
	});
	const select = (id) => {
		const item = items().find((candidate) => candidate.id === id);
		if (!item || item.disabled) return false;
		options.onAction?.(item.id);
		return true;
	};
	const move = (direction) => {
		const next = moveMenuHighlight(items(), highlighted(), direction);
		if (next === void 0) return false;
		setHighlighted(next);
		return true;
	};
	const handleKeyDown = (event) => {
		const handled = match(event.key).with("ArrowDown", () => move("next")).with("ArrowUp", () => move("previous")).with("Home", () => move("first")).with("End", () => move("last")).with("Enter", () => select(highlighted())).with("Escape", () => {
			options.onDismiss?.();
			return options.onDismiss !== void 0;
		}).otherwise(() => false);
		if (handled) event.preventDefault();
		return handled;
	};
	return {
		highlighted,
		setHighlighted,
		select,
		move,
		handleKeyDown
	};
}
/** Searchable command list whose filtering and keyboard behavior are host-independent. */
function Command(props) {
	const [uncontrolledQuery, setUncontrolledQuery] = createSignal(props.defaultQuery ?? "");
	const query = () => props.query ?? uncontrolledQuery();
	const filtered = createMemo(() => filterCommandItems(props.items, query()));
	const setQuery = (next) => {
		if (props.query === void 0) setUncontrolledQuery(next);
		props.onQueryChange?.(next);
	};
	const navigation = createCommandListNavigation(filtered, {
		onAction: (id) => {
			filtered().find((item) => item.id === id)?.onSelect?.();
			props.onAction?.(id);
		},
		onDismiss: props.onDismiss
	});
	return createComponent$1(View, {
		get "class"() {
			return mergeClasses("min-w-0 flex flex-col gap-2", props.class);
		},
		get children() {
			return [createComponent$1(Input, {
				get "aria-label"() {
					return props["aria-label"];
				},
				get value() {
					return query();
				},
				get placeholder() {
					return props.placeholder ?? "Type a command";
				},
				ref(r$) {
					var _ref$ = props.inputRef;
					typeof _ref$ === "function" || Array.isArray(_ref$) ? applyRef(_ref$, r$) : props.inputRef = r$;
				},
				onInput: (event) => setQuery(event.currentTarget.value),
				get onKeyDown() {
					return navigation.handleKeyDown;
				}
			}), createComponent$1(CommandList, {
				get "aria-label"() {
					return `${props["aria-label"]} results`;
				},
				get items() {
					return filtered();
				},
				get highlighted() {
					return navigation.highlighted();
				},
				get emptyText() {
					return props.emptyText;
				},
				get loading() {
					return props.loading;
				},
				get loadingText() {
					return props.loadingText;
				},
				get error() {
					return props.error;
				},
				get errorText() {
					return props.errorText;
				},
				get retryLabel() {
					return props.retryLabel;
				},
				get onRetry() {
					return props.onRetry;
				},
				get "class"() {
					return props.listClass;
				},
				get onHighlightChange() {
					return navigation.setHighlighted;
				},
				get onAction() {
					return navigation.select;
				}
			})];
		}
	});
}
//#endregion
export { Command, CommandList, createCommandListNavigation, filterCommandItems, reconcileCommandHighlight };

//# sourceMappingURL=command.mjs.map