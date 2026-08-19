import { applyRef, createComponent, createElement, createFps, insertNode, memo, mergeProps } from "@wabou/core/renderer";
import { animate, createTransition } from "@wabou/animation";
import { Button as Button$1, Center, CodeEditor, CollapsiblePresence, Column, Icon, Modal, NetworkImage, PasswordInput as PasswordInput$1, Popover, Pulse, ScrollArea, Spin, Text, TextArea as TextArea$1, TextInput, View, createFocusWithin, createMeasuredSize, rotate2d, translate2d } from "@wabou/primitives";
import { For, createComponent as createComponent$1, createContext, createEffect, createMemo, createSignal, createUniqueId, getOwner, omit, onCleanup, untrack, useContext } from "solid-js";
import { P, match } from "ts-pattern";
import { shadow } from "@wabou/core/style";
import { CalendarDate, endOfMonth, isSameDay, startOfMonth } from "@internationalized/date";
import { rgba, useHost, useWindow } from "@wabou/core";
import calendarIcon from "lucide-static/icons/calendar.svg?raw";
import chevronLeft from "lucide-static/icons/chevron-left.svg?raw";
import chevronRight from "lucide-static/icons/chevron-right.svg?raw";
import { createControllableState, createDisclosure, createRovingFocus, createSelectInteraction, isSelected, toggleSelection } from "@wabou/primitives/interactions";
import chevronDown from "lucide-static/icons/chevron-down.svg?raw";
import check from "lucide-static/icons/check.svg?raw";
import minus from "lucide-static/icons/minus.svg?raw";
//#region src/class-names.ts
function join(...values) {
	return values.filter(Boolean).join(" ");
}
//#endregion
//#region src/range.ts
function finiteOr(value, fallback) {
	return value !== void 0 && Number.isFinite(value) ? value : fallback;
}
function normalizePercentage(value) {
	return Math.max(0, Math.min(100, finiteOr(value, 0)));
}
function normalizeRange(minValue, maxValue, stepValue) {
	const min = finiteOr(minValue, 0);
	const max = Math.max(min, finiteOr(maxValue, 100));
	const candidateStep = finiteOr(stepValue, 1);
	return {
		min,
		max,
		step: candidateStep > 0 ? candidateStep : 1
	};
}
function decimalPlaces(value) {
	const [coefficient, exponentText] = String(value).toLowerCase().split("e");
	const fractionLength = coefficient.split(".")[1]?.length ?? 0;
	const exponent = exponentText === void 0 ? 0 : Number(exponentText);
	return Math.max(0, Math.min(100, fractionLength - exponent));
}
//#endregion
//#region src/theme.ts
/**
* Native elevation recipes adapted from gpui-component. Wabou and GPUI both
* pass standard deviation directly to their renderer, so these values should
* not use CSS's doubled blur radius. Floating surfaces also carry a subtle
* foreground-colored ring: black in light mode, white in dark mode.
*/
function componentsElevation(theme, elevation) {
	if (elevation === "raised") return [shadow({
		offsetY: 1,
		stdDev: 2,
		color: 46
	})];
	if (elevation === "floating") return [
		shadow({
			spread: 1,
			stdDev: 0,
			color: theme === "dark" ? 4294967066 : 26
		}),
		shadow({
			offsetY: 4,
			stdDev: 3,
			spread: -1,
			color: 26
		}),
		shadow({
			offsetY: 2,
			stdDev: 2,
			spread: -2,
			color: 26
		})
	];
	return [shadow({
		offsetY: 20,
		stdDev: 25,
		spread: -5,
		color: 26
	}), shadow({
		offsetY: 8,
		stdDev: 10,
		spread: -6,
		color: 26
	})];
}
const defaultTheme = { theme: () => "dark" };
const ThemeContext = createContext(defaultTheme);
function ComponentsProvider(props) {
	return createComponent$1(ThemeContext, {
		value: { theme: () => props.theme ?? "dark" },
		get children() {
			return props.children;
		}
	});
}
function useComponentsTheme() {
	return (getOwner() ? useContext(ThemeContext) : defaultTheme).theme;
}
//#endregion
//#region src/avatar.tsx
function Avatar(props) {
	const size = () => match(props.size ?? "default").with("sm", () => "w-8 h-8 text-xs").with("default", () => "w-10 h-10 text-sm").with("lg", () => "w-12 h-12 text-base").exhaustive();
	return createComponent(Center, {
		role: "img",
		get ["aria-label"]() {
			return props.alt ?? props.fallback;
		},
		get ["class"]() {
			return join("flex-none overflow-hidden rounded-full bg-control border border-subtle", size(), props.class);
		},
		get children() {
			return memo(() => {
				return !!props.src;
			})() ? createComponent(NetworkImage, {
				"aria-hidden": "true",
				get url() {
					return props.src;
				},
				format: "raster",
				cache: "memory",
				class: "w-full h-full"
			}) : createComponent(Text, {
				"aria-hidden": "true",
				class: "font-medium text-secondary",
				get children() {
					return props.fallback;
				}
			});
		}
	});
}
function AvatarGroup(props) {
	return createComponent(View, {
		get ["class"]() {
			return join("flex items-center gap-1", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function AvatarGroupCount(props) {
	return createComponent(Center, {
		get ["class"]() {
			return join("w-10 h-10 flex-none rounded-full bg-control border border-subtle", props.class);
		},
		get children() {
			return createComponent(Text, {
				class: "text-xs font-medium text-muted",
				get children() {
					return props.children;
				}
			});
		}
	});
}
//#endregion
//#region src/config-editor.tsx
/**
* Experimental native configuration editor. Its Wabou-owned props deliberately
* hide the editor-core implementation so the backend can evolve independently.
*/
function ConfigEditor(props) {
	return createComponent(CodeEditor, mergeProps(props, {
		language: "json",
		get ["class"]() {
			return join("min-h-48 w-full rounded-md border border-strong bg-input text-primary", props.class);
		}
	}));
}
//#endregion
//#region src/date-picker.tsx
function dayOfWeek(value) {
	return new Date(Date.UTC(value.year, value.month - 1, value.day)).getUTCDay();
}
const DEFAULT_LABELS = {
	previousMonth: "Previous month",
	nextMonth: "Next month",
	today: "Today",
	selectToday: "Select today"
};
/** A Wabou-native calendar using @internationalized/date for date arithmetic. */
function Calendar(props) {
	const host = useHost();
	const systemToday = () => {
		const value = host.intl.today();
		return new CalendarDate(value.year, value.month, value.day);
	};
	const locale = () => {
		const requested = props.locale ?? host.intl.locale();
		return Intl.DateTimeFormat.supportedLocalesOf([requested])[0] ?? "en";
	};
	const labels = () => ({
		...DEFAULT_LABELS,
		...props.labels
	});
	const initial = props.value ?? props.defaultValue ?? systemToday();
	const [localValue, setLocalValue] = createSignal(initial);
	const [visibleMonth, setVisibleMonth] = createSignal(startOfMonth(initial));
	const [focusedDate, setFocusedDate] = createSignal(initial);
	const value = () => props.value ?? localValue();
	const dayRefs = /* @__PURE__ */ new Map();
	const monthInfo = createMemo(() => {
		const currentLocale = locale();
		const firstWeekday = new Intl.Locale(currentLocale).getWeekInfo().firstDay % 7;
		const weekday = new Intl.DateTimeFormat(currentLocale, {
			weekday: "short",
			timeZone: "UTC"
		});
		return {
			first_weekday: firstWeekday,
			month_label: new Intl.DateTimeFormat(currentLocale, {
				year: "numeric",
				month: "long",
				timeZone: "UTC"
			}).format(Date.UTC(visibleMonth().year, visibleMonth().month - 1, 1)),
			weekday_labels: Array.from({ length: 7 }, (_, offset) => {
				const index = (firstWeekday + offset) % 7;
				return weekday.format(Date.UTC(2024, 0, 7 + index));
			})
		};
	});
	const dateFormatters = createMemo(() => ({
		medium: new Intl.DateTimeFormat(locale(), {
			dateStyle: "medium",
			timeZone: "UTC"
		}),
		full: new Intl.DateTimeFormat(locale(), {
			dateStyle: "full",
			timeZone: "UTC"
		})
	}));
	const formatDate = (date, style) => dateFormatters()[style].format(Date.UTC(date.year, date.month - 1, date.day));
	const days = () => {
		const first = startOfMonth(visibleMonth());
		const offset = (dayOfWeek(first) - monthInfo().first_weekday + 7) % 7;
		const gridStart = first.subtract({ days: offset });
		return Array.from({ length: 42 }, (_, index) => gridStart.add({ days: index }));
	};
	const unavailable = (date) => props.disabled || props.minValue !== void 0 && date.compare(props.minValue) < 0 || props.maxValue !== void 0 && date.compare(props.maxValue) > 0 || props.isDateUnavailable?.(date) === true;
	const canShowMonth = (month) => (props.minValue === void 0 || endOfMonth(month).compare(props.minValue) >= 0) && (props.maxValue === void 0 || month.compare(props.maxValue) <= 0);
	const select = (date) => {
		if (unavailable(date)) return;
		if (props.value === void 0) setLocalValue(date);
		setVisibleMonth(startOfMonth(date));
		setFocusedDate(date);
		props.onValueChange?.(date);
	};
	const focusDate = (date) => {
		setFocusedDate(date);
		if (date.month !== visibleMonth().month || date.year !== visibleMonth().year) setVisibleMonth(startOfMonth(date));
		requestAnimationFrame(() => dayRefs.get(date.toString())?.focus());
	};
	const tabStop = () => {
		const focused = focusedDate();
		if (focused.year === visibleMonth().year && focused.month === visibleMonth().month && !unavailable(focused)) return focused;
		let candidate = startOfMonth(visibleMonth());
		while (candidate.month === visibleMonth().month) {
			if (!unavailable(candidate)) return candidate;
			candidate = candidate.add({ days: 1 });
		}
		return startOfMonth(visibleMonth());
	};
	const focusAvailable = (date, step) => {
		let candidate = date;
		for (let attempts = 0; attempts < 366; attempts++) {
			if (!unavailable(candidate)) {
				focusDate(candidate);
				return;
			}
			candidate = candidate.add({ days: step });
		}
	};
	const handleKeyDown = (event, date) => {
		let next;
		let step = 1;
		if (event.key === "ArrowLeft") {
			next = date.subtract({ days: 1 });
			step = -1;
		} else if (event.key === "ArrowRight") next = date.add({ days: 1 });
		else if (event.key === "ArrowUp") {
			next = date.subtract({ days: 7 });
			step = -1;
		} else if (event.key === "ArrowDown") next = date.add({ days: 7 });
		else if (event.key === "Home") {
			next = date.subtract({ days: (dayOfWeek(date) - monthInfo().first_weekday + 7) % 7 });
			step = -1;
		} else if (event.key === "End") next = date.add({ days: 6 - (dayOfWeek(date) - monthInfo().first_weekday + 7) % 7 });
		else if (event.key === "PageUp") {
			next = date.subtract({ months: 1 });
			step = -1;
		} else if (event.key === "PageDown") next = date.add({ months: 1 });
		else if (event.key === "Enter" || event.key === " ") select(date);
		else return;
		event.preventDefault();
		if (next) focusAvailable(next, step);
	};
	return createComponent(View, {
		get ["aria-label"]() {
			return props["aria-label"] ?? "Calendar";
		},
		class: "w-72 p-3 flex flex-col gap-3",
		get children() {
			return [
				createComponent(View, {
					class: "h-8 flex items-center justify-between",
					get children() {
						return [
							createComponent(Button$1, {
								unstyled: true,
								get ["aria-label"]() {
									return labels().previousMonth;
								},
								get disabled() {
									return props.disabled || !canShowMonth(visibleMonth().subtract({ months: 1 }));
								},
								class: "w-8 h-8 rounded-md items-center justify-center",
								onClick: () => setVisibleMonth((month) => month.subtract({ months: 1 })),
								get children() {
									return createComponent(Icon, {
										source: chevronLeft,
										size: 16
									});
								}
							}),
							createComponent(Text, {
								class: "font-medium text-sm text-primary",
								get children() {
									return monthInfo().month_label;
								}
							}),
							createComponent(Button$1, {
								unstyled: true,
								get ["aria-label"]() {
									return labels().nextMonth;
								},
								get disabled() {
									return props.disabled || !canShowMonth(visibleMonth().add({ months: 1 }));
								},
								class: "w-8 h-8 rounded-md items-center justify-center",
								onClick: () => setVisibleMonth((month) => month.add({ months: 1 })),
								get children() {
									return createComponent(Icon, {
										source: chevronRight,
										size: 16
									});
								}
							})
						];
					}
				}),
				createComponent(View, {
					class: "w-64 flex flex-wrap gap-1",
					get children() {
						return [createComponent(For, {
							get each() {
								return monthInfo().weekday_labels;
							},
							children: (day) => createComponent(Text, {
								class: "w-8 h-7 flex items-center justify-center text-xs text-muted",
								children: day
							})
						}), createComponent(For, {
							get each() {
								return days();
							},
							keyed: false,
							children: (date) => {
								const selected = () => isSameDay(date(), value());
								const outside = () => date().month !== visibleMonth().month;
								const disabled = () => unavailable(date());
								return createComponent(Button$1, {
									ref: (node) => dayRefs.set(date().toString(), node),
									unstyled: true,
									get ["aria-label"]() {
										return formatDate(date(), "full");
									},
									get ["aria-selected"]() {
										return selected();
									},
									get ["aria-current"]() {
										return isSameDay(date(), systemToday()) ? "date" : void 0;
									},
									get focusOrder() {
										return isSameDay(date(), tabStop()) ? 0 : -1;
									},
									get disabled() {
										return disabled();
									},
									class: (state) => join("w-8 h-8 rounded-md items-center justify-center text-sm", selected() ? "bg-accent text-on-accent" : state.hovered ? "bg-control-hover text-primary" : "bg-transparent text-primary", outside() && "text-muted"),
									get style() {
										return { opacity: disabled() ? .35 : 1 };
									},
									onClick: () => select(date()),
									onKeyDown: (event) => handleKeyDown(event, date()),
									get children() {
										return createComponent(Text, { get children() {
											return date().day;
										} });
									}
								});
							}
						})];
					}
				}),
				createComponent(View, {
					class: "pt-2 flex items-center border-t border-subtle",
					get children() {
						return createComponent(Button$1, {
							unstyled: true,
							get ["aria-label"]() {
								return labels().selectToday;
							},
							class: "h-8 px-2 rounded-md text-sm text-accent",
							onClick: () => select(systemToday()),
							get children() {
								return labels().today;
							}
						});
					}
				})
			];
		}
	});
}
/** A shadcn-inspired date picker composed from Wabou Popover and Calendar. */
function DatePicker(props) {
	const host = useHost();
	const theme = useComponentsTheme();
	const [localValue, setLocalValue] = createSignal(props.defaultValue);
	const [localOpen, setLocalOpen] = createSignal(props.defaultOpen ?? false);
	const open = () => props.open ?? localOpen();
	const setOpen = (next) => {
		if (props.open === void 0) setLocalOpen(next);
		props.onOpenChange?.(next);
	};
	const value = () => props.value ?? localValue();
	const locale = () => {
		const requested = props.locale ?? host.intl.locale();
		return Intl.DateTimeFormat.supportedLocalesOf([requested])[0] ?? "en";
	};
	const formatted = () => {
		const date = value();
		return date ? new Intl.DateTimeFormat(locale(), {
			dateStyle: "medium",
			timeZone: "UTC"
		}).format(Date.UTC(date.year, date.month - 1, date.day)) : props.placeholder ?? "Pick a date";
	};
	const select = (date) => {
		if (props.value === void 0) setLocalValue(date);
		props.onValueChange?.(date);
		setOpen(false);
	};
	return createComponent(Popover, {
		get ["aria-label"]() {
			return props["aria-label"];
		},
		get open() {
			return open();
		},
		onOpenChange: setOpen,
		placement: "bottom-start",
		contentClass: "rounded-lg border border-subtle bg-surface",
		get contentShadows() {
			return memo(() => {
				return props.contentShadows === void 0;
			})() ? componentsElevation(theme(), "floating") : props.contentShadows;
		},
		trigger: (trigger) => createComponent(Button$1, mergeProps({ unstyled: true }, trigger, {
			get ["aria-label"]() {
				return props["aria-label"];
			},
			get disabled() {
				return props.disabled;
			},
			get ["class"]() {
				return join("w-72 h-8 px-3 justify-start gap-2 rounded-md border border-subtle bg-input text-sm shadow-xs", props.class);
			},
			get children() {
				return [createComponent(Icon, {
					source: calendarIcon,
					class: "flex-none text-muted",
					size: 16
				}), createComponent(Text, {
					get ["class"]() {
						return value() ? "text-primary" : "text-muted";
					},
					get children() {
						return formatted();
					}
				})];
			}
		})),
		get children() {
			return createComponent(Calendar, mergeProps(props, {
				get value() {
					return value();
				},
				get ["aria-label"]() {
					return props["aria-label"];
				},
				onValueChange: select
			}));
		}
	});
}
//#endregion
//#region src/dialog.tsx
function Dialog(props) {
	const theme = useComponentsTheme();
	return createComponent(Modal, mergeProps(props, {
		get backdropClass() {
			return props.backdropClass;
		},
		get backdropStyle() {
			return {
				"background-color": rgba(51),
				...props.backdropStyle
			};
		},
		get contentClass() {
			return join("w-[480px] max-w-full min-w-0 flex flex-col gap-4 rounded-lg border border-subtle bg-surface p-5", props.contentClass);
		},
		get contentShadows() {
			return memo(() => {
				return props.contentShadows === void 0;
			})() ? componentsElevation(theme(), "modal") : props.contentShadows;
		}
	}));
}
function DialogHeader(props) {
	return createComponent(View, {
		get ["class"]() {
			return join("flex flex-col gap-1", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function DialogFooter(props) {
	return createComponent(View, {
		get ["class"]() {
			return join("flex items-center justify-end gap-2", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function DialogTitle(props) {
	return createComponent(Text, {
		get ["class"]() {
			return join("text-lg font-semibold text-primary", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function DialogDescription(props) {
	return createComponent(Text, {
		get ["class"]() {
			return join("w-full min-w-0 whitespace-normal text-sm text-muted", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
//#endregion
//#region src/disclosure.tsx
function DisclosureIndicator(props) {
	const rotation = createTransition(() => props.open() ? Math.PI : 0, {
		duration: .2,
		ease: "easeOut",
		reducedMotion: props.reducedMotion
	});
	return createComponent(View, {
		class: "w-4 h-4 flex-none",
		get transform() {
			return rotate2d(rotation.value());
		},
		"aria-hidden": "true",
		get children() {
			return createComponent(Icon, {
				source: chevronDown,
				class: "text-muted",
				size: 16
			});
		}
	});
}
const CollapsibleContext = createContext();
const useCollapsible = () => {
	const value = useContext(CollapsibleContext);
	if (!value) throw new Error("Collapsible parts must be inside Collapsible");
	return value;
};
function Collapsible(props) {
	const state = createDisclosure({
		open: () => props.open,
		defaultOpen: props.defaultOpen,
		disabled: () => props.disabled ?? false,
		onOpenChange: props.onOpenChange
	});
	const context = {
		open: state.open,
		toggle: state.toggle,
		disabled: state.disabled,
		reducedMotion: () => props.reducedMotion ?? false
	};
	return createComponent(CollapsibleContext, {
		value: context,
		get children() {
			return createComponent(View, {
				get ["class"]() {
					return join("flex flex-col", props.class);
				},
				get children() {
					return props.children;
				}
			});
		}
	});
}
function CollapsibleTrigger(props) {
	const context = useCollapsible();
	return createComponent(Button$1, {
		unstyled: true,
		get disabled() {
			return context.disabled();
		},
		get ["aria-expanded"]() {
			return context.open();
		},
		class: "w-full",
		get onClick() {
			return context.toggle;
		},
		get children() {
			return createComponent(View, {
				get ["class"]() {
					return join("w-full flex items-center justify-between gap-3", props.class);
				},
				get children() {
					return [memo(() => {
						return props.children;
					}), createComponent(DisclosureIndicator, {
						get open() {
							return context.open;
						},
						get reducedMotion() {
							return context.reducedMotion;
						}
					})];
				}
			});
		}
	});
}
function CollapsibleContent(props) {
	const context = useCollapsible();
	return createComponent(CollapsiblePresence, {
		get open() {
			return context.open();
		},
		get reducedMotion() {
			return context.reducedMotion();
		},
		get contentClass() {
			return props.class;
		},
		get children() {
			return props.children;
		}
	});
}
function nextAccordionValue(current, type, item, collapsible = false) {
	return toggleSelection(current, item, type, collapsible) ?? "";
}
const AccordionContext = createContext();
const AccordionItemContext = createContext();
const useAccordion = () => {
	const value = useContext(AccordionContext);
	if (!value) throw new Error("Accordion parts must be inside Accordion");
	return value;
};
const useAccordionItem = () => {
	const value = useContext(AccordionItemContext);
	if (!value) throw new Error("Accordion parts must be inside AccordionItem");
	return value;
};
function Accordion(props) {
	const type = () => props.type ?? "single";
	const state = createControllableState({
		value: () => props.value,
		defaultValue: props.defaultValue ?? (type() === "multiple" ? [] : ""),
		disabled: () => props.disabled ?? false,
		onChange: props.onValueChange
	});
	return createComponent(AccordionContext, {
		value: {
			active: (item) => isSelected(state.value(), item),
			toggle: (item) => {
				state.set(nextAccordionValue(state.value(), type(), item, props.collapsible));
			},
			disabled: () => props.disabled ?? false,
			reducedMotion: () => props.reducedMotion ?? false
		},
		get children() {
			return createComponent(View, {
				get ["class"]() {
					return join("flex flex-col", props.class);
				},
				get children() {
					return props.children;
				}
			});
		}
	});
}
function AccordionItem(props) {
	return createComponent(AccordionItemContext, {
		get value() {
			return {
				value: props.value,
				disabled: () => props.disabled ?? false
			};
		},
		get children() {
			return createComponent(View, {
				get ["class"]() {
					return join("flex flex-col border-b border-subtle", props.class);
				},
				get children() {
					return props.children;
				}
			});
		}
	});
}
function AccordionTrigger(props) {
	const root = useAccordion();
	const item = useAccordionItem();
	const open = () => root.active(item.value);
	return createComponent(Button$1, {
		unstyled: true,
		get disabled() {
			return root.disabled() || item.disabled();
		},
		get ["aria-expanded"]() {
			return open();
		},
		class: "w-full",
		onClick: () => root.toggle(item.value),
		get children() {
			return createComponent(View, {
				get ["class"]() {
					return join("w-full py-4 flex items-center justify-between gap-4", props.class);
				},
				get children() {
					return [createComponent(Text, {
						class: "min-w-0 whitespace-normal text-sm font-medium text-primary",
						get children() {
							return props.children;
						}
					}), createComponent(DisclosureIndicator, {
						open,
						get reducedMotion() {
							return root.reducedMotion;
						}
					})];
				}
			});
		}
	});
}
function AccordionContent(props) {
	const root = useAccordion();
	const item = useAccordionItem();
	return createComponent(CollapsiblePresence, {
		get open() {
			return root.active(item.value);
		},
		get reducedMotion() {
			return root.reducedMotion();
		},
		get contentClass() {
			return join("pb-4", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
//#endregion
//#region src/display.tsx
function Skeleton(props) {
	return createComponent(Pulse, {
		"aria-hidden": "true",
		get ["class"]() {
			return join("rounded-md bg-control", props.class);
		},
		from: .45,
		to: .85,
		duration: 1.8
	});
}
function Spinner(props) {
	return createComponent(Spin, {
		role: "status",
		get ["aria-label"]() {
			return props.label ?? "Loading";
		},
		get ["class"]() {
			return join("w-4 h-4 flex-none text-accent", props.class);
		},
		duration: .9,
		get children() {
			var _el$ = createElement("svg", {
				"aria-hidden": "true",
				class: "w-full h-full",
				viewBox: "0 0 24 24",
				fill: "none"
			});
			var _el$2 = createElement("circle", {
				cx: "12",
				cy: "12",
				r: "9",
				stroke: "currentColor",
				"stroke-width": "3",
				opacity: "0.25"
			});
			var _el$3 = createElement("path", {
				d: "M 12 3 A 9 9 0 0 1 21 12",
				stroke: "currentColor",
				"stroke-width": "3",
				"stroke-linecap": "round"
			});
			insertNode(_el$, _el$2);
			insertNode(_el$, _el$3);
			return _el$;
		}
	});
}
function Kbd(props) {
	return createComponent(Text, {
		get ["class"]() {
			return join("h-5 min-w-5 px-1 py-0.5 flex-none text-center rounded bg-control text-xs font-medium text-muted", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function KbdGroup(props) {
	return createComponent(View, {
		get ["class"]() {
			return join("inline-flex items-center gap-1", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
//#endregion
//#region src/forms.tsx
function Field(props) {
	const layout = () => match(props.orientation ?? "vertical").with("vertical", () => "flex-col gap-2").with("horizontal", () => "flex-row items-start gap-4").exhaustive();
	return createComponent(View, {
		get ["class"]() {
			return join("w-full flex", layout(), props.invalid && "text-danger-primary", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function FieldGroup(props) {
	return createComponent(View, {
		get ["class"]() {
			return join("flex flex-col gap-5", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function FieldLabel(props) {
	return createComponent(Text, {
		get ["class"]() {
			return join("text-sm font-medium text-primary", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function FieldContent(props) {
	return createComponent(View, {
		get ["class"]() {
			return join("min-w-0 flex-1 flex flex-col gap-1", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function FieldDescription(props) {
	return createComponent(Text, {
		get ["class"]() {
			return join("w-full min-w-0 whitespace-normal text-xs text-muted", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function FieldError(props) {
	return createComponent(Text, {
		role: "alert",
		get ["class"]() {
			return join("w-full min-w-0 whitespace-normal text-xs text-danger-primary", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function InputGroup(props) {
	const focus = createFocusWithin();
	return createComponent(View, mergeProps(() => {
		return focus.bindings;
	}, {
		get ["class"]() {
			return join("w-full h-8 flex items-center rounded-md border bg-input shadow-xs", focus.focusWithin() ? "border-focus" : "border-strong", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function InputGroupInput(props) {
	return createComponent(Input, mergeProps(props, { get ["class"]() {
		return join("flex-1 min-w-0 border-transparent bg-transparent", props.class);
	} }));
}
function InputGroupText(props) {
	return createComponent(Text, {
		get ["class"]() {
			return join("flex-none px-3 text-sm text-muted", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function InputGroupButton(props) {
	return createComponent(Button, mergeProps(props, {
		get size() {
			return props.size ?? "sm";
		},
		get variant() {
			return props.variant ?? "ghost";
		},
		get ["class"]() {
			return join("mx-1", props.class);
		}
	}));
}
function InputGroupTextArea(props) {
	return createComponent(TextArea$1, mergeProps(props, { get ["class"]() {
		return join("w-full h-24 px-3 py-2 border-transparent bg-transparent text-sm", props.class);
	} }));
}
//#endregion
//#region src/layout.tsx
function Empty(props) {
	return createComponent(Column, {
		get ["class"]() {
			return join("w-full min-h-64 p-8 items-center justify-center gap-4 rounded-lg border border-subtle bg-surface shadow-xs", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function EmptyHeader(props) {
	return createComponent(Column, {
		get ["class"]() {
			return join("max-w-md items-center gap-2", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function EmptyMedia(props) {
	return createComponent(Center, {
		get ["class"]() {
			return join("w-12 h-12 flex-none rounded-lg bg-control text-secondary", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function EmptyTitle(props) {
	return createComponent(Text, {
		get ["class"]() {
			return join("text-base font-semibold text-primary", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function EmptyDescription(props) {
	return createComponent(Text, {
		get ["class"]() {
			return join("w-full min-w-0 whitespace-normal text-center text-sm text-muted", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function EmptyContent(props) {
	return createComponent(View, {
		get ["class"]() {
			return join("flex items-center gap-2", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function ButtonGroup(props) {
	const layout = () => match(props.orientation ?? "horizontal").with("horizontal", () => "flex-row items-center").with("vertical", () => "flex-col items-stretch").exhaustive();
	return createComponent(View, {
		role: "group",
		get ["class"]() {
			return join("flex gap-1", layout(), props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function ButtonGroupText(props) {
	return createComponent(Text, {
		get ["class"]() {
			return join("px-2 text-sm text-muted", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
/**
* A horizontal primary/aside boundary with explicit flex shrink semantics.
* Use `SplitPaneMain` for the elastic region and `SplitPaneAside` for a
* class-sized fixed rail. Both regions clip at their own boundary, so content
* cannot paint across the divider or a rounded parent clip.
*/
function SplitPane(props) {
	return createComponent(View, {
		get ["class"]() {
			return join("w-full min-w-0 flex flex-row overflow-hidden", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function SplitPaneMain(props) {
	return createComponent(View, {
		get ["class"]() {
			return join("flex-1 min-w-0 overflow-hidden", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function SplitPaneAside(props) {
	return createComponent(View, {
		get ["class"]() {
			return join("flex-none min-w-0 overflow-hidden", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
//#endregion
//#region src/select.tsx
const ITEM_HEIGHT = 40;
const VISIBLE_ITEMS = 6;
/** Shadcn-inspired single Select backed by Wabou-native interaction state. */
function Select(props) {
	const theme = useComponentsTheme();
	const id = createUniqueId();
	let trigger;
	let content;
	let viewport;
	let scrollTop = 0;
	const items = () => props.options.map((option) => ({
		id: option.value,
		textValue: option.label,
		disabled: option.disabled
	}));
	const execute = (command) => {
		match(command).with({ type: "FOCUS_TRIGGER" }, () => requestAnimationFrame(() => trigger?.focus())).with({ type: "FOCUS_CONTENT" }, () => requestAnimationFrame(() => content?.focus())).with({ type: "SCROLL_TO_ITEM" }, ({ id }) => {
			const index = props.options.findIndex((option) => option.value === id);
			if (index < 0) return;
			const firstVisible = Math.floor(scrollTop / ITEM_HEIGHT);
			const lastVisible = firstVisible + VISIBLE_ITEMS - 1;
			const nextTop = index < firstVisible ? index * ITEM_HEIGHT : index > lastVisible ? (index - VISIBLE_ITEMS + 1) * ITEM_HEIGHT : scrollTop;
			if (nextTop !== scrollTop) {
				scrollTop = nextTop;
				requestAnimationFrame(() => viewport?.scrollTo({ top: nextTop }));
			}
		}).exhaustive();
	};
	const interaction = createSelectInteraction({
		items,
		value: () => props.value,
		defaultValue: props.defaultValue,
		open: () => props.open,
		defaultOpen: props.defaultOpen,
		disabled: () => props.disabled ?? false,
		onValueChange: props.onValueChange,
		onOpenChange: props.onOpenChange,
		execute
	});
	const selected = () => props.options.find((option) => option.value === interaction.value());
	const handleKeyDown = (event) => {
		if (match(event.key).with("ArrowDown", () => interaction.send({ type: "ARROW_DOWN" })).with("ArrowUp", () => interaction.send({ type: "ARROW_UP" })).with("Home", () => interaction.send({ type: "HOME" })).with("End", () => interaction.send({ type: "END" })).with("Enter", () => interaction.send({ type: interaction.open() ? "SELECT" : "OPEN" })).with(" ", () => interaction.send({ type: interaction.open() ? "SELECT" : "OPEN" })).with("Escape", () => interaction.send({ type: "CLOSE" })).otherwise((key) => interaction.typeahead(key))) event.preventDefault();
	};
	return createComponent(Popover, {
		contentRole: "presentation",
		popupRole: "listbox",
		get open() {
			return interaction.open();
		},
		onOpenChange: (open) => {
			interaction.send({ type: open ? "OPEN" : "CLOSE" });
		},
		placement: "bottom-start",
		get contentClass() {
			return join("w-72 p-1 rounded-lg border border-subtle bg-surface", props.contentClass);
		},
		get contentShadows() {
			return memo(() => {
				return props.contentShadows === void 0;
			})() ? componentsElevation(theme(), "floating") : props.contentShadows;
		},
		trigger: (popover) => createComponent(Button$1, {
			unstyled: true,
			role: "combobox",
			get disabled() {
				return props.disabled;
			},
			get ["aria-label"]() {
				return props["aria-label"];
			},
			"aria-haspopup": "listbox",
			get ["aria-expanded"]() {
				return interaction.open();
			},
			"aria-controls": `${id}-listbox`,
			ref: (node) => {
				trigger = node;
				popover.ref(node);
			},
			class: (state) => join("w-72 h-8 px-3 justify-between gap-3 rounded-md border bg-input text-sm shadow-xs", state.focused ? "border-focus" : "border-subtle", props.class),
			style: (state) => ({ opacity: state.disabled ? .45 : 1 }),
			get onClick() {
				return popover.onClick;
			},
			onKeyDown: (event) => {
				popover.onKeyDown(event);
				handleKeyDown(event);
			},
			get children() {
				return [createComponent(Text, {
					get ["class"]() {
						return join("min-w-0 flex-1 text-left truncate", selected() ? "text-primary" : "text-muted");
					},
					get children() {
						return selected()?.label ?? props.placeholder ?? "Select an option";
					}
				}), createComponent(Icon, {
					source: chevronDown,
					class: "flex-none text-muted",
					size: 16
				})];
			}
		}),
		get children() {
			return createComponent(ScrollArea, {
				ref: (node) => {
					viewport = node;
					scrollTop = 0;
				},
				class: "w-full flex-none",
				contentClass: "gap-1",
				get style() {
					return { height: `${Math.max(1, Math.min(props.options.length, VISIBLE_ITEMS)) * ITEM_HEIGHT - 4}px` };
				},
				onScroll: (event) => {
					scrollTop = event.scrollY ?? scrollTop;
				},
				get children() {
					return createComponent(View, {
						id: `${id}-listbox`,
						ref: (node) => content = node,
						role: "listbox",
						get ["aria-label"]() {
							return props["aria-label"];
						},
						get ["aria-activedescendant"]() {
							return memo(() => {
								return !!interaction.highlighted();
							})() ? `${id}-option-${interaction.highlighted()}` : void 0;
						},
						focusOrder: 0,
						class: "min-w-0 flex flex-col gap-1",
						onKeyDown: handleKeyDown,
						get children() {
							return createComponent(For, {
								get each() {
									return props.options;
								},
								keyed: false,
								children: (option) => {
									const selected = () => interaction.value() === option().value;
									const highlighted = () => interaction.highlighted() === option().value;
									return createComponent(View, {
										get id() {
											return `${id}-option-${option().value}`;
										},
										role: "option",
										get ["aria-selected"]() {
											return selected();
										},
										get ["aria-disabled"]() {
											return option().disabled;
										},
										get ["class"]() {
											return join("w-full h-8 flex-none px-3 flex items-center justify-between gap-3 rounded-md text-sm", highlighted() ? "bg-control-hover text-primary" : "bg-transparent text-secondary");
										},
										get style() {
											return { opacity: option().disabled ? .45 : 1 };
										},
										onPointerMove: () => interaction.send({
											type: "HIGHLIGHT",
											id: option().value
										}),
										onClick: () => interaction.send({
											type: "SELECT",
											id: option().value
										}),
										get children() {
											return [createComponent(Text, {
												class: "min-w-0 flex-1 text-sm whitespace-nowrap text-ellipsis",
												get children() {
													return option().label;
												}
											}), createComponent(View, {
												"aria-hidden": "true",
												class: "w-4 h-4 flex-none",
												get children() {
													return memo(() => {
														return !!selected();
													})() ? createComponent(Icon, {
														source: check,
														class: "text-accent",
														size: 16
													}) : selected();
												}
											})];
										}
									});
								}
							});
						}
					});
				}
			});
		}
	});
}
//#endregion
//#region src/selection.tsx
const SELECTION_INDICATOR_CLASS = "w-5 h-5 flex-none border";
function Checkbox(props) {
	const state = createControllableState({
		value: () => props.checked,
		defaultValue: props.defaultChecked ?? false,
		disabled: () => props.disabled ?? false,
		onChange: props.onCheckedChange
	});
	const checked = state.value;
	const toggle = () => {
		state.set(!checked());
	};
	const boxColors = () => match({
		checked: checked(),
		indeterminate: !!props.indeterminate
	}).with({ checked: true }, () => "bg-accent border-accent text-on-accent").with({ indeterminate: true }, () => "bg-accent border-accent text-on-accent").otherwise(() => "bg-input border-strong text-primary");
	const ariaChecked = () => match({
		checked: checked(),
		indeterminate: !!props.indeterminate
	}).with({ indeterminate: true }, () => "mixed").otherwise(({ checked }) => checked);
	const indicator = () => match({
		checked: checked(),
		indeterminate: !!props.indeterminate
	}).with({ indeterminate: true }, () => minus).with({ checked: true }, () => check).otherwise(() => void 0);
	return createComponent(Button$1, {
		unstyled: true,
		role: "checkbox",
		get disabled() {
			return props.disabled;
		},
		get ["aria-label"]() {
			return props["aria-label"] ?? props.label;
		},
		get ["aria-checked"]() {
			return ariaChecked();
		},
		get selected() {
			return checked();
		},
		class: (buttonState) => join("min-h-7 px-1 items-center gap-2 rounded-md border border-transparent", buttonState.hovered && "bg-control-hover", buttonState.focusVisible && "border-focus", props.class),
		style: (buttonState) => ({ opacity: buttonState.disabled ? .45 : 1 }),
		onClick: toggle,
		get children() {
			return [createComponent(Center, {
				"aria-hidden": "true",
				get ["class"]() {
					return join(SELECTION_INDICATOR_CLASS, "rounded text-xs font-bold", boxColors());
				},
				get children() {
					return memo(() => {
						return !!indicator();
					})() ? createComponent(Icon, {
						get source() {
							return indicator();
						},
						size: 14,
						class: "text-on-accent"
					}) : indicator();
				}
			}), memo(() => {
				return memo(() => {
					return !!props.label;
				})() ? createComponent(Text, {
					class: "text-sm text-secondary",
					get children() {
						return props.label;
					}
				}) : props.label;
			})];
		}
	});
}
const RadioContext = createContext();
function RadioGroup(props) {
	const state = createControllableState({
		value: () => props.value,
		defaultValue: props.defaultValue,
		disabled: () => props.disabled ?? false,
		onChange: (value) => value !== void 0 && props.onValueChange?.(value)
	});
	const value = state.value;
	const select = (next) => {
		state.set(next);
	};
	const roving = createRovingFocus({
		orientation: () => "vertical",
		onMove: select
	});
	return createComponent$1(RadioContext, {
		value: {
			value,
			select,
			disabled: () => props.disabled ?? false,
			register: (id, target, disabled) => roving.register({
				id,
				target,
				disabled
			}),
			move: roving.move
		},
		get children() {
			return createComponent(View, {
				role: "radiogroup",
				get ["aria-label"]() {
					return props["aria-label"];
				},
				get ["class"]() {
					return join("flex flex-col gap-3", props.class);
				},
				get children() {
					return props.children;
				}
			});
		}
	});
}
function RadioGroupItem(props) {
	const group = useContext(RadioContext);
	if (!group) throw new Error("RadioGroupItem must be used inside RadioGroup");
	const checked = () => group.value() === props.value;
	const disabled = () => group.disabled() || (props.disabled ?? false);
	let unregister;
	onCleanup(() => unregister?.());
	return createComponent(Button$1, {
		unstyled: true,
		role: "radio",
		get disabled() {
			return disabled();
		},
		get selected() {
			return checked();
		},
		get ["aria-label"]() {
			return props.label;
		},
		get ["aria-checked"]() {
			return checked();
		},
		ref: (node) => {
			unregister?.();
			unregister = group.register(props.value, node, disabled);
		},
		class: (buttonState) => join("min-h-7 px-1 items-center gap-2 rounded-md border border-transparent", buttonState.hovered && "bg-control-hover", buttonState.focusVisible && "border-focus", props.class),
		style: (buttonState) => ({ opacity: buttonState.disabled ? .45 : 1 }),
		onClick: () => group.select(props.value),
		onKeyDown: (event) => {
			if (group.move(props.value, event.key)) event.preventDefault();
		},
		get children() {
			return [createComponent(Center, {
				"aria-hidden": "true",
				get ["class"]() {
					return join(SELECTION_INDICATOR_CLASS, "rounded-full bg-input", match(checked()).with(true, () => "border-accent").with(false, () => "border-strong").exhaustive());
				},
				get children() {
					return memo(() => {
						return !!checked();
					})() ? createComponent(View, { class: "w-2.5 h-2.5 rounded-full bg-accent" }) : checked();
				}
			}), memo(() => {
				return memo(() => {
					return !!props.label;
				})() ? createComponent(Text, {
					class: "text-sm text-secondary",
					get children() {
						return props.label;
					}
				}) : props.label;
			})];
		}
	});
}
function Toggle(props) {
	const state = createControllableState({
		value: () => props.pressed,
		defaultValue: props.defaultPressed ?? false,
		disabled: () => props.disabled ?? false,
		onChange: props.onPressedChange
	});
	const pressed = state.value;
	const toggle = () => {
		state.set(!pressed());
	};
	const size = () => match(props.size ?? "default").with("sm", () => "h-6 min-w-6 px-2 text-xs").with("default", () => "h-8 min-w-8 px-2.5 text-sm").with("lg", () => "h-10 min-w-10 px-3 text-sm").exhaustive();
	const colors = (state) => match({
		selected: pressed(),
		hovered: state.hovered
	}).with({ selected: true }, () => "bg-selected border-accent text-primary").with({ hovered: true }, () => "bg-control-hover text-primary").otherwise(() => "bg-transparent text-secondary");
	return createComponent(Button$1, {
		unstyled: true,
		get disabled() {
			return props.disabled;
		},
		get selected() {
			return pressed();
		},
		get ["aria-label"]() {
			return props["aria-label"];
		},
		get ["aria-pressed"]() {
			return pressed();
		},
		class: (state) => join("items-center justify-center rounded-md border font-medium", size(), colors(state), match(props.variant ?? "default").with("outline", () => "border-strong").with("default", () => "border-transparent").exhaustive(), state.focusVisible && "border-focus", props.class),
		style: (state) => ({ opacity: state.disabled ? .45 : 1 }),
		onClick: toggle,
		get children() {
			return props.children;
		}
	});
}
const ToggleGroupContext = createContext();
/** Shadcn-style single-value toggle group with native roving focus. */
function ToggleGroup(props) {
	const state = createControllableState({
		value: () => props.value,
		defaultValue: props.defaultValue,
		disabled: () => props.disabled ?? false,
		onChange: (value) => value !== void 0 && props.onValueChange?.(value)
	});
	const roving = createRovingFocus({
		orientation: () => "horizontal",
		onMove: (value) => state.set(value)
	});
	const context = {
		value: state.value,
		disabled: () => props.disabled ?? false,
		select: (value) => state.set(value),
		register: (value, node, disabled) => roving.register({
			id: value,
			target: node,
			disabled
		}),
		move: roving.move
	};
	return createComponent$1(ToggleGroupContext, {
		value: context,
		get children() {
			return createComponent(View, {
				role: "group",
				get ["aria-label"]() {
					return props["aria-label"];
				},
				get ["class"]() {
					return join("flex flex-row items-center gap-0.5 rounded-md bg-control p-0.5", props.class);
				},
				get children() {
					return props.children;
				}
			});
		}
	});
}
function ToggleGroupItem(props) {
	const group = useContext(ToggleGroupContext);
	if (!group) throw new Error("ToggleGroupItem must be used inside ToggleGroup");
	const selected = () => group.value() === props.value;
	const disabled = () => group.disabled() || (props.disabled ?? false);
	let unregister;
	onCleanup(() => unregister?.());
	return createComponent(Button$1, {
		unstyled: true,
		get disabled() {
			return disabled();
		},
		get selected() {
			return selected();
		},
		get ["aria-pressed"]() {
			return selected();
		},
		ref: (node) => {
			unregister?.();
			unregister = group.register(props.value, node, disabled);
		},
		class: (state) => join("h-7 flex-1 px-3 items-center justify-center rounded-sm border border-transparent text-sm font-medium", selected() ? "bg-surface text-primary" : state.hovered ? "bg-control-hover text-primary" : "bg-transparent text-muted", state.focusVisible && "border-focus", props.class),
		style: (state) => ({ opacity: state.disabled ? .45 : 1 }),
		onClick: () => group.select(props.value),
		onKeyDown: (event) => {
			if (group.move(props.value, event.key)) event.preventDefault();
		},
		get children() {
			return props.children;
		}
	});
}
//#endregion
//#region src/slider.tsx
function Slider(props) {
	const range = () => normalizeRange(props.min, props.max, props.step);
	const min = () => range().min;
	const max = () => range().max;
	const step = () => range().step;
	const clamp = (value) => Math.max(min(), Math.min(max(), finiteOr(value, min())));
	const snap = (value) => {
		const stepped = min() + Math.round((clamp(value) - min()) / step()) * step();
		const precision = decimalPlaces(step());
		return clamp(Number(stepped.toFixed(precision)));
	};
	const [local, setLocal] = createSignal(snap(props.defaultValue ?? min()));
	const value = () => snap(props.value ?? local());
	const ratio = () => max() === min() ? 0 : (value() - min()) / (max() - min());
	const measured = createMeasuredSize();
	const [dragging, setDragging] = createSignal(false);
	const [focused, setFocused] = createSignal(false);
	const update = (next) => {
		if (props.disabled) return;
		const normalized = snap(next);
		const changed = normalized !== value();
		if (props.value === void 0) setLocal(normalized);
		if (changed) props.onValueChange?.(normalized);
	};
	const updateFromPointer = (event) => {
		const width = measured.width();
		if (width <= 0) return;
		event.preventDefault();
		update(min() + Math.max(0, Math.min(width, event.offsetX)) / width * (max() - min()));
	};
	const changeBy = (amount) => update(value() + amount);
	const onKeyDown = (event) => {
		if (props.disabled) return;
		if (event.key === "ArrowLeft" || event.key === "ArrowDown") changeBy(-step());
		else if (event.key === "ArrowRight" || event.key === "ArrowUp") changeBy(step());
		else if (event.key === "PageDown") changeBy(-step() * 10);
		else if (event.key === "PageUp") changeBy(step() * 10);
		else if (event.key === "Home") update(min());
		else if (event.key === "End") update(max());
		else return;
		event.preventDefault();
	};
	return createComponent(View, {
		ref(r$) {
			var _ref$ = measured.ref;
			typeof _ref$ === "function" || Array.isArray(_ref$) ? applyRef(_ref$, r$) : measured.ref = r$;
		},
		role: "slider",
		get ["aria-label"]() {
			return props.label;
		},
		get ["aria-valuemin"]() {
			return min();
		},
		get ["aria-valuemax"]() {
			return max();
		},
		get ["aria-valuenow"]() {
			return value();
		},
		get ["aria-valuetext"]() {
			return props.valueText?.(value()) ?? String(value());
		},
		get ["aria-disabled"]() {
			return props.disabled;
		},
		get focusOrder() {
			return props.disabled ? -1 : 0;
		},
		get ["class"]() {
			return join("h-7 relative flex items-center", props.disabled ? "cursor-not-allowed" : "cursor-pointer", props.class);
		},
		onFocus: () => setFocused(true),
		onBlur: () => {
			setFocused(false);
			setDragging(false);
		},
		onPointerDown: (event) => {
			setDragging(true);
			updateFromPointer(event);
		},
		onPointerMove: (event) => {
			if (dragging() && event.buttons !== 0) updateFromPointer(event);
		},
		onPointerUp: (event) => {
			if (dragging()) updateFromPointer(event);
			setDragging(false);
		},
		onPointerCancel: () => setDragging(false),
		onKeyDown,
		get style() {
			return { opacity: props.disabled ? .45 : 1 };
		},
		get children() {
			return [createComponent(View, {
				"aria-hidden": "true",
				class: "w-full h-1.5 overflow-hidden rounded-full border border-subtle bg-control",
				get children() {
					return createComponent(View, {
						class: "h-full rounded-full bg-accent",
						get style() {
							return { width: `${ratio() * 100}%` };
						}
					});
				}
			}), createComponent(View, {
				"aria-hidden": "true",
				get ["class"]() {
					return join("w-4 h-4 absolute rounded-full border bg-surface shadow-xs", focused() || dragging() ? "border-focus" : "border-strong");
				},
				get style() {
					return {
						left: `${ratio() * Math.max(0, measured.width() - 16)}px`,
						top: "6px"
					};
				}
			})];
		}
	});
}
//#endregion
//#region src/tabs.tsx
const orientationClass = (orientation, horizontal, vertical) => match(orientation).with("horizontal", () => horizontal).with("vertical", () => vertical).exhaustive();
const TabsContext = createContext();
function Tabs(props) {
	const state = createControllableState({
		value: () => props.value,
		defaultValue: props.defaultValue,
		onChange: (value) => value !== void 0 && props.onValueChange?.(value)
	});
	const value = state.value;
	const select = (next) => {
		state.set(next);
	};
	const roving = createRovingFocus({
		orientation: () => props.orientation ?? "horizontal",
		onMove: select
	});
	const context = {
		value,
		orientation: () => props.orientation ?? "horizontal",
		select,
		register: (next, node, disabled) => {
			const unregister = roving.register({
				id: next,
				target: node,
				disabled
			});
			if (value() === void 0) select(next);
			return unregister;
		},
		move: roving.move
	};
	return createComponent$1(TabsContext, {
		value: context,
		get children() {
			return createComponent(View, {
				get ["class"]() {
					return join("flex gap-3", orientationClass(context.orientation(), "flex-col", "flex-row"), props.class);
				},
				get children() {
					return props.children;
				}
			});
		}
	});
}
function TabsList(props) {
	const context = useContext(TabsContext);
	if (!context) throw new Error("TabsList must be used inside Tabs");
	return createComponent(View, {
		role: "tablist",
		get ["aria-label"]() {
			return props["aria-label"];
		},
		get ["aria-orientation"]() {
			return context.orientation();
		},
		get ["class"]() {
			return join("flex-none flex items-center gap-1", orientationClass(context.orientation(), "flex-row", "flex-col"), match(props.variant ?? "default").with("default", () => "p-0.5 rounded-md bg-control").with("line", () => "bg-transparent").exhaustive(), props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function TabsTrigger(props) {
	const context = useContext(TabsContext);
	if (!context) throw new Error("TabsTrigger must be used inside Tabs");
	const selected = () => context.value() === props.value;
	let unregister;
	onCleanup(() => unregister?.());
	return createComponent(Button$1, {
		unstyled: true,
		role: "tab",
		get disabled() {
			return props.disabled;
		},
		get selected() {
			return selected();
		},
		get ["aria-selected"]() {
			return selected();
		},
		ref: (node) => {
			unregister?.();
			unregister = context.register(props.value, node, () => props.disabled ?? false);
		},
		class: (state) => join("h-7 px-3 items-center justify-center rounded-sm border border-transparent text-sm font-medium", match({
			selected: selected(),
			hovered: state.hovered
		}).with({ selected: true }, () => "bg-surface text-primary shadow-xs").with({ hovered: true }, () => "bg-control-hover text-primary").otherwise(() => "bg-transparent text-muted"), state.focusVisible && "border-focus", props.class),
		style: (state) => ({ opacity: state.disabled ? .45 : 1 }),
		onClick: () => context.select(props.value),
		onKeyDown: (event) => {
			if (context.move(props.value, event.key)) event.preventDefault();
		},
		get children() {
			return createComponent(Text, {
				class: "text-sm font-medium",
				get children() {
					return props.children;
				}
			});
		}
	});
}
function TabsContent(props) {
	const context = useContext(TabsContext);
	if (!context) throw new Error("TabsContent must be used inside Tabs");
	return context.value() === props.value ? createComponent(View, {
		role: "tabpanel",
		get ["class"]() {
			return join("flex-1", props.class);
		},
		get children() {
			return props.children;
		}
	}) : null;
}
//#endregion
//#region src/title-bar.tsx
const titleBarClass = "border-b border-subtle";
const titleBarLayoutStyle = {
	display: "flex",
	"flex-direction": "row",
	"align-items": "center",
	"flex-shrink": 0,
	height: "40px"
};
const titleBarDragRegionLayoutStyle = {
	display: "flex",
	"flex-direction": "row",
	"align-items": "center",
	"flex-grow": 1,
	"flex-shrink": 1,
	"flex-basis": "0%",
	height: "100%"
};
/** Layout shell for an application-owned title bar. */
function TitleBar(props) {
	return createComponent(View, mergeProps(props, {
		get ["class"]() {
			return join(titleBarClass, props.class);
		},
		get style() {
			return {
				...titleBarLayoutStyle,
				...props.style
			};
		}
	}));
}
/** Explicit non-interactive region that moves the native window. */
function TitleBarDragRegion(props) {
	const window = useWindow();
	return createComponent(View, mergeProps(props, {
		get ["class"]() {
			return props.class;
		},
		get style() {
			return {
				...titleBarDragRegionLayoutStyle,
				...props.style
			};
		},
		onPointerDown: (event) => {
			if (event.button === 0) window.startDragging();
		},
		onDblClick: () => window.setMaximized(!window.maximized())
	}));
}
//#endregion
//#region src/index.tsx
function buttonColors(variant, state) {
	const focus = state.focusVisible ? "border-focus" : "";
	const passiveBorder = (variant) => match(variant).with("outline", () => "border-strong").with(P.union("default", "secondary", "ghost", "destructive"), () => "border-transparent").exhaustive();
	return match({
		variant,
		pressed: state.pressed,
		hovered: state.hovered
	}).with({
		variant: "default",
		pressed: true
	}, () => join("bg-accent-pressed border-transparent text-on-accent", focus)).with({
		variant: "default",
		hovered: true
	}, () => join("bg-accent-hover border-transparent text-on-accent", focus)).with({ variant: "default" }, () => join("bg-accent border-transparent text-on-accent", focus)).with({
		variant: "destructive",
		pressed: true
	}, () => join("bg-danger-pressed border-transparent text-on-accent", focus)).with({
		variant: "destructive",
		hovered: true
	}, () => join("bg-danger-hover border-transparent text-on-accent", focus)).with({ variant: "destructive" }, () => join("bg-danger border-transparent text-on-accent", focus)).with({
		variant: "secondary",
		pressed: true
	}, () => join("bg-control-pressed border-transparent text-primary", focus)).with({
		variant: "secondary",
		hovered: true
	}, () => join("bg-control-hover border-transparent text-primary", focus)).with({ variant: "secondary" }, () => join("bg-control border-transparent text-primary", focus)).with({ pressed: true }, ({ variant }) => join("bg-control-pressed text-secondary", passiveBorder(variant), focus)).with({ hovered: true }, ({ variant }) => join("bg-control-hover text-secondary", passiveBorder(variant), focus)).with({ variant: P.union("outline", "ghost") }, ({ variant }) => join("bg-transparent text-secondary", passiveBorder(variant), focus)).exhaustive();
}
function buttonSize(size) {
	return match(size).with("sm", () => "h-6 px-2 text-xs").with("default", () => "h-8 px-3 text-sm").with("lg", () => "h-10 px-4 text-base").with("icon", () => "w-8 h-8 p-0 text-sm").exhaustive();
}
function Button(props) {
	const local = props;
	const forwarded = omit(props, "variant", "size", "class", "style");
	const variant = () => local.variant ?? "default";
	const size = () => local.size ?? "default";
	return createComponent(Button$1, mergeProps(forwarded, {
		unstyled: true,
		class: (state) => join("inline-flex flex-none whitespace-nowrap items-center justify-center rounded-md border font-medium", buttonColors(variant(), state), buttonSize(size()), local.class),
		style: (state) => ({
			"border-width": 1,
			opacity: state.disabled ? .45 : 1,
			...typeof local.style === "function" ? local.style(state) : local.style
		})
	}));
}
function badgeColors(variant) {
	return match(variant).with("default", () => "bg-accent border-accent text-on-accent").with("secondary", () => "bg-control border-subtle text-primary").with("outline", () => "bg-transparent border-strong text-secondary").with("success", () => "bg-success-surface border-success-primary text-success-primary").with("destructive", () => "bg-danger-surface border-danger text-danger-primary").exhaustive();
}
function Badge(props) {
	return createComponent(Text, {
		get ["class"]() {
			return join("flex-none whitespace-nowrap px-2 py-0.5 rounded-md border text-xs font-medium", badgeColors(props.variant ?? "default"), props.class);
		},
		get children() {
			return props.children;
		}
	});
}
/** Live host frame-rate indicator with sensible performance thresholds. */
function Fps(props) {
	const measured = props.value === void 0 ? createFps() : () => props.value ?? 0;
	const value = () => Math.max(0, Math.round(measured()));
	const variant = () => match(value()).with(0, () => "outline").with(P.when((fps) => fps >= (props.goodAt ?? 55)), () => "success").with(P.when((fps) => fps < (props.warningBelow ?? 30)), () => "destructive").otherwise(() => "secondary");
	return createComponent(Badge, {
		get variant() {
			return variant();
		},
		get ["class"]() {
			return join("font-mono", props.class);
		},
		get children() {
			return [memo(() => {
				return value();
			}), memo(() => {
				return memo(() => {
					return props.label === "";
				})() ? "" : ` ${props.label ?? "fps"}`;
			})];
		}
	});
}
function Card(props) {
	const theme = useComponentsTheme();
	return createComponent(View, {
		get ["class"]() {
			return join("flex flex-col overflow-hidden rounded-lg border", "border-subtle bg-surface", props.class);
		},
		get shadows() {
			return memo(() => {
				return props.shadows === void 0;
			})() ? componentsElevation(theme(), "raised") : props.shadows;
		},
		get children() {
			return props.children;
		}
	});
}
function CardHeader(props) {
	return createComponent(View, {
		get ["class"]() {
			return join("flex flex-col gap-1 px-4 pt-4", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function CardTitle(props) {
	return createComponent(Text, {
		get ["class"]() {
			return join("text-base font-semibold", "text-primary", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function CardDescription(props) {
	return createComponent(Text, {
		get ["class"]() {
			return join("w-full min-w-0 whitespace-normal text-sm", "text-muted", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function CardContent(props) {
	return createComponent(View, {
		get ["class"]() {
			return join("flex flex-col gap-3 p-4", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function CardFooter(props) {
	return createComponent(View, {
		get ["class"]() {
			return join("flex items-center gap-2 px-4 pb-4", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function Separator(props) {
	const dimensions = () => match(props.orientation ?? "horizontal").with("horizontal", () => "h-px w-full").with("vertical", () => "w-px h-full").exhaustive();
	return createComponent(View, {
		"aria-hidden": "true",
		get ["class"]() {
			return join("flex-none", "bg-subtle", dimensions(), props.class);
		}
	});
}
function Alert(props) {
	const colors = () => match(props.variant ?? "default").with("default", () => ({
		container: "border-subtle bg-surface",
		title: "text-primary",
		body: "text-secondary"
	})).with("destructive", () => ({
		container: "border-danger bg-danger-surface",
		title: "text-danger-primary",
		body: "text-danger-primary"
	})).exhaustive();
	return createComponent(View, {
		role: "alert",
		get ["aria-label"]() {
			return props.title;
		},
		get ["class"]() {
			return join("flex flex-col gap-1 rounded-lg border p-4 shadow-xs", colors().container, props.class);
		},
		get children() {
			return [createComponent(Text, {
				get ["class"]() {
					return join("text-sm font-semibold", colors().title);
				},
				get children() {
					return props.title;
				}
			}), createComponent(Text, {
				get ["class"]() {
					return join("w-full min-w-0 whitespace-normal text-sm", colors().body);
				},
				get children() {
					return props.children;
				}
			})];
		}
	});
}
/** A plain-text input. Secrets must use {@link PasswordInput}. */
function Input(props) {
	return createComponent(TextInput, mergeProps(props, { get ["class"]() {
		return join("h-8 w-full px-3 rounded-md border text-sm shadow-xs", "border-subtle bg-input text-primary", props.disabled && "opacity-50", props.class);
	} }));
}
/** A native secret input whose value never crosses into JavaScript. */
function PasswordInput(props) {
	return createComponent(PasswordInput$1, mergeProps(props, { get ["class"]() {
		return join("h-8 w-full px-3 rounded-md border text-sm shadow-xs", "border-subtle bg-input text-primary", props.disabled && "opacity-50", props.class);
	} }));
}
function TextArea(props) {
	return createComponent(TextArea$1, mergeProps(props, { get ["class"]() {
		return join("h-24 w-full px-3 py-2 rounded-md border text-sm shadow-xs", "border-subtle bg-input text-primary", props.disabled && "opacity-50", props.class);
	} }));
}
function switchColors(checked, state) {
	return match({
		checked,
		pressed: state.pressed,
		hovered: state.hovered
	}).with({
		checked: true,
		pressed: true
	}, () => "bg-accent-pressed").with({
		checked: true,
		hovered: true
	}, () => "bg-accent-hover").with({ checked: true }, () => "bg-accent").with({
		checked: false,
		pressed: true
	}, () => "bg-control-pressed").with({
		checked: false,
		hovered: true
	}, () => "bg-control-hover").with({ checked: false }, () => "bg-control").exhaustive();
}
function Switch(props) {
	const [local, setLocal] = createSignal(props.defaultChecked ?? false);
	const checked = () => props.checked ?? local();
	const [thumbX, setThumbX] = createSignal(checked() ? 20 : 0);
	let movement;
	createEffect(checked, (isChecked) => {
		const target = isChecked ? 20 : 0;
		const from = untrack(thumbX);
		if (from === target) return;
		movement?.stop();
		movement = animate(from, target, {
			duration: .18,
			ease: [
				.22,
				1,
				.36,
				1
			],
			onUpdate: setThumbX
		});
	});
	onCleanup(() => movement?.stop());
	const toggle = () => {
		if (props.disabled) return;
		const next = !checked();
		if (props.checked === void 0) setLocal(next);
		props.onCheckedChange?.(next);
	};
	return createComponent(View, {
		class: "flex items-center gap-3",
		get children() {
			return [createComponent(Button$1, {
				unstyled: true,
				role: "switch",
				get disabled() {
					return props.disabled;
				},
				get ["aria-label"]() {
					return props["aria-label"] ?? props.label;
				},
				get ["aria-checked"]() {
					return checked();
				},
				class: (state) => join("w-11 h-6 flex-none rounded-full p-0.5", switchColors(checked(), state), state.focused && "border border-focus"),
				style: (state) => ({ opacity: state.disabled ? .45 : 1 }),
				onClick: toggle,
				get children() {
					return createComponent(View, {
						"aria-hidden": "true",
						class: "w-5 h-5 rounded-full bg-on-accent",
						get transform() {
							return translate2d(thumbX(), 0);
						}
					});
				}
			}), memo(() => {
				return memo(() => {
					return !!props.label;
				})() ? createComponent(Text, {
					class: "text-sm text-secondary",
					get children() {
						return props.label;
					}
				}) : props.label;
			})];
		}
	});
}
function Progress(props) {
	const value = () => normalizePercentage(props.value);
	return createComponent(View, {
		role: "progressbar",
		get ["aria-label"]() {
			return props.label ?? "Progress";
		},
		"aria-valuemin": 0,
		"aria-valuemax": 100,
		get ["aria-valuenow"]() {
			return value();
		},
		get ["aria-valuetext"]() {
			return `${value()} percent`;
		},
		get ["class"]() {
			return join("w-full h-2 overflow-hidden rounded-full", "bg-control", props.class);
		},
		get children() {
			return createComponent(View, {
				"aria-hidden": "true",
				class: "h-full bg-accent rounded-full",
				get style() {
					return { width: `${value()}%` };
				}
			});
		}
	});
}
//#endregion
export { Accordion, AccordionContent, AccordionItem, AccordionTrigger, Alert, Avatar, AvatarGroup, AvatarGroupCount, Badge, Button, ButtonGroup, ButtonGroupText, Calendar, CalendarDate, Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle, Checkbox, Collapsible, CollapsibleContent, CollapsibleTrigger, ComponentsProvider, ConfigEditor, DatePicker, Dialog, DialogDescription, DialogFooter, DialogHeader, DialogTitle, Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle, Field, FieldContent, FieldDescription, FieldError, FieldGroup, FieldLabel, Fps, Input, InputGroup, InputGroupButton, InputGroupInput, InputGroupText, InputGroupTextArea, Kbd, KbdGroup, PasswordInput, Progress, RadioGroup, RadioGroupItem, Select, Separator, Skeleton, Slider, Spinner, SplitPane, SplitPaneAside, SplitPaneMain, Switch, Tabs, TabsContent, TabsList, TabsTrigger, TextArea, TitleBar, TitleBarDragRegion, Toggle, ToggleGroup, ToggleGroupItem, componentsElevation, nextAccordionValue, titleBarClass, titleBarDragRegionLayoutStyle, titleBarLayoutStyle, useComponentsTheme };

//# sourceMappingURL=index.mjs.map