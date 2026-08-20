import { $ as createPress, A as createFormDraft, B as Text, D as createKeyedSelection, E as Row, F as NetworkImage, G as translate2d, H as TextInput, I as PasswordInput$1, J as createMeasuredSize, K as createPresence, L as Path, M as CodeEditor, N as Icon, O as isSelected, P as Image, Q as createActive, R as PathBuilder, S as createOverlayLayer, T as Column, U as View, V as TextArea, W as rotate2d$1, X as Link, Y as Button$1, Z as createButton, _ as Pulse, a as ScrollArea, at as animateKeyframes, b as Modal, ct as createRotation, et as createHover, g as createNotifications, h as NotificationRegion, i as createScrollReset, it as animate, j as CollapsiblePresence, k as toggleSelection, lt as createTransition, n as createTabs, nt as createFocusWithin, o as Popover, ot as createLoop, q as createContainerMatch, r as createShortcuts, rt as createAnimationFrame, st as createPulse, t as primitives_exports, tt as createFocus, v as Ripple, w as Center, x as OverlayPlaneProvider, y as Spin, z as Svg } from "./primitives-BkH0U8N7.mjs";
import { rgba, useDialog, useHost, useWindow } from "@wabou/core";
import { shadow } from "@wabou/core/style";
import { For, Show, createComponent, createContext, createEffect, createMemo, createSignal, createUniqueId, flush, getOwner, omit, onCleanup, untrack, useContext } from "solid-js";
import { applyRef, createComponent as createComponent$1, createElement, createFps, insertNode, memo, mergeProps } from "@wabou/core/renderer";
import { P, match } from "ts-pattern";
import { CalendarDate, endOfMonth, isSameDay, startOfMonth } from "@internationalized/date";
import calendarIcon from "lucide-static/icons/calendar.svg?raw";
import chevronLeft from "lucide-static/icons/chevron-left.svg?raw";
import chevronRight from "lucide-static/icons/chevron-right.svg?raw";
import folder from "lucide-static/icons/folder.svg?raw";
import chevronDown from "lucide-static/icons/chevron-down.svg?raw";
import check from "lucide-static/icons/check.svg?raw";
import minus from "lucide-static/icons/minus.svg?raw";
import { createMemoryHistory, createMemoryHistory as createMemoryHistory$1 } from "@tanstack/history";
import { BaseRootRoute, BaseRoute, RouterCore, notFound, redirect } from "@tanstack/router-core";
export * from "@wabou/core";
export * from "@wabou/core/i18n";
//#region src/components/class-names.ts
function join(...values) {
	return values.filter(Boolean).join(" ");
}
//#endregion
//#region src/components/range.ts
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
//#region src/components/theme.ts
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
	return createComponent(ThemeContext, {
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
//#region src/components/avatar.tsx
function Avatar(props) {
	const size = () => match(props.size ?? "default").with("sm", () => "w-8 h-8 text-xs").with("default", () => "w-10 h-10 text-sm").with("lg", () => "w-12 h-12 text-base").exhaustive();
	return createComponent$1(Center, {
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
			})() ? createComponent$1(NetworkImage, {
				"aria-hidden": "true",
				get url() {
					return props.src;
				},
				format: "raster",
				cache: "memory",
				class: "w-full h-full"
			}) : createComponent$1(Text, {
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
	return createComponent$1(View, {
		get ["class"]() {
			return join("flex items-center gap-1", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function AvatarGroupCount(props) {
	return createComponent$1(Center, {
		get ["class"]() {
			return join("w-10 h-10 flex-none rounded-full bg-control border border-subtle", props.class);
		},
		get children() {
			return createComponent$1(Text, {
				class: "text-xs font-medium text-muted",
				get children() {
					return props.children;
				}
			});
		}
	});
}
//#endregion
//#region src/components/button.tsx
function buttonColors(variant, state) {
	const focus = state.focusVisible ? "border-focus" : "";
	const passiveBorder = (value) => match(value).with("outline", () => "border-strong").with(P.union("default", "secondary", "ghost", "destructive"), () => "border-transparent").exhaustive();
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
	}, () => join("bg-control-hover border-transparent text-primary", focus)).with({ variant: "secondary" }, () => join("bg-control border-transparent text-primary", focus)).with({ pressed: true }, ({ variant: value }) => join("bg-control-pressed text-secondary", passiveBorder(value), focus)).with({ hovered: true }, ({ variant: value }) => join("bg-control-hover text-secondary", passiveBorder(value), focus)).with({ variant: P.union("outline", "ghost") }, ({ variant: value }) => join("bg-transparent text-secondary", passiveBorder(value), focus)).exhaustive();
}
function buttonSize(size) {
	return match(size).with("sm", () => "h-6 px-2 text-xs").with("default", () => "h-8 px-3 text-sm").with("lg", () => "h-10 px-4 text-base").with("icon", () => "w-8 h-8 p-0 text-sm").exhaustive();
}
function Button(props) {
	const local = props;
	const forwarded = omit(props, "variant", "size", "class", "style");
	const variant = () => local.variant ?? "default";
	const size = () => local.size ?? "default";
	return createComponent$1(Button$1, mergeProps(forwarded, {
		unstyled: true,
		class: (state) => join("inline-flex flex-none whitespace-nowrap items-center justify-center gap-2 rounded-md border font-medium", buttonColors(variant(), state), buttonSize(size()), local.class),
		style: (state) => ({
			"border-width": 1,
			opacity: state.disabled ? .45 : 1,
			...typeof local.style === "function" ? local.style(state) : local.style
		})
	}));
}
//#endregion
//#region src/components/config-editor.tsx
/**
* Experimental native configuration editor. Its Wabou-owned props deliberately
* hide the editor-core implementation so the backend can evolve independently.
*/
function ConfigEditor(props) {
	return createComponent$1(CodeEditor, mergeProps(props, {
		language: "json",
		get ["class"]() {
			return join("min-h-48 w-full rounded-md border border-strong bg-input text-primary", props.class);
		}
	}));
}
//#endregion
//#region src/components/date-picker.tsx
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
	return createComponent$1(View, {
		get ["aria-label"]() {
			return props["aria-label"] ?? "Calendar";
		},
		class: "w-72 p-3 flex flex-col gap-3",
		get children() {
			return [
				createComponent$1(View, {
					class: "h-8 flex items-center justify-between",
					get children() {
						return [
							createComponent$1(Button$1, {
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
									return createComponent$1(Icon, {
										source: chevronLeft,
										size: 16
									});
								}
							}),
							createComponent$1(Text, {
								class: "font-medium text-sm text-primary",
								get children() {
									return monthInfo().month_label;
								}
							}),
							createComponent$1(Button$1, {
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
									return createComponent$1(Icon, {
										source: chevronRight,
										size: 16
									});
								}
							})
						];
					}
				}),
				createComponent$1(View, {
					class: "w-64 flex flex-wrap gap-1",
					get children() {
						return [createComponent$1(For, {
							get each() {
								return monthInfo().weekday_labels;
							},
							children: (day) => createComponent$1(Text, {
								class: "w-8 h-7 flex items-center justify-center text-xs text-muted",
								children: day
							})
						}), createComponent$1(For, {
							get each() {
								return days();
							},
							keyed: false,
							children: (date) => {
								const selected = () => isSameDay(date(), value());
								const outside = () => date().month !== visibleMonth().month;
								const disabled = () => unavailable(date());
								return createComponent$1(Button$1, {
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
										return createComponent$1(Text, { get children() {
											return date().day;
										} });
									}
								});
							}
						})];
					}
				}),
				createComponent$1(View, {
					class: "pt-2 flex items-center border-t border-subtle",
					get children() {
						return createComponent$1(Button$1, {
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
	return createComponent$1(Popover, {
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
		trigger: (trigger) => createComponent$1(Button$1, mergeProps({ unstyled: true }, trigger, {
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
				return [createComponent$1(Icon, {
					source: calendarIcon,
					class: "flex-none text-muted",
					size: 16
				}), createComponent$1(Text, {
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
			return createComponent$1(Calendar, mergeProps(props, {
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
//#region src/components/dialog.tsx
function Dialog(props) {
	const theme = useComponentsTheme();
	return createComponent$1(Modal, mergeProps(props, {
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
	return createComponent$1(View, {
		get ["class"]() {
			return join("flex flex-col gap-1", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function DialogFooter(props) {
	return createComponent$1(View, {
		get ["class"]() {
			return join("flex items-center justify-end gap-2", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
/**
* The shrinking, independently scrollable region between a dialog's fixed
* header and footer. The dialog surface must have a bounded or maximum height.
*/
function DialogScrollBody(props) {
	return createComponent$1(ScrollArea, {
		get ["class"]() {
			return join("min-h-0 flex-1", props.class);
		},
		get contentClass() {
			return props.contentClass;
		},
		get children() {
			return props.children;
		}
	});
}
function DialogTitle(props) {
	return createComponent$1(Text, {
		get ["class"]() {
			return join("text-lg font-semibold text-primary", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function DialogDescription(props) {
	return createComponent$1(Text, {
		get ["class"]() {
			return join("w-full min-w-0 whitespace-normal text-sm text-muted", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
//#endregion
//#region src/components/directory-picker-state.ts
function directoryPickerOptions(value, options) {
	return {
		...options,
		directory: options?.directory ?? (value.trim() || void 0)
	};
}
//#endregion
//#region src/components/input.tsx
/** A plain-text input. Secrets must use `PasswordInput`. */
function Input(props) {
	return createComponent$1(TextInput, mergeProps(props, { get ["class"]() {
		return join("h-8 w-full px-3 rounded-md border text-sm shadow-xs", "border-subtle bg-input text-primary", props.disabled && "opacity-50", props.class);
	} }));
}
//#endregion
//#region src/components/directory-picker.tsx
/** A controlled path input paired with the operating system directory picker. */
function DirectoryPicker(props) {
	const nativeDialog = useDialog();
	const [pending, setPending] = createSignal(false);
	const local = props;
	const inputProps = omit(props, "value", "onValueChange", "dialogOptions", "browseLabel", "pendingLabel", "browseAriaLabel", "class", "inputClass", "buttonClass", "onBrowseError");
	async function browse() {
		if (pending() || inputProps.disabled) return;
		setPending(true);
		try {
			const selected = await nativeDialog.pickDirectory(directoryPickerOptions(local.value, local.dialogOptions));
			if (selected !== null) local.onValueChange(selected);
		} catch (error) {
			if (local.onBrowseError) local.onBrowseError(error);
			else throw error;
		} finally {
			setPending(false);
		}
	}
	return createComponent$1(View, {
		get ["class"]() {
			return join("w-full min-w-0 flex items-center gap-2", local.class);
		},
		get children() {
			return [createComponent$1(Input, mergeProps(inputProps, {
				get ["class"]() {
					return join("min-w-0 flex-1", local.inputClass);
				},
				get value() {
					return local.value;
				},
				onInput: (event) => local.onValueChange(event.currentTarget.value)
			})), createComponent$1(Button, {
				get ["class"]() {
					return join("flex-none", local.buttonClass);
				},
				variant: "outline",
				get disabled() {
					return Boolean(inputProps.disabled) || pending();
				},
				get ["aria-label"]() {
					return local.browseAriaLabel ?? local.browseLabel ?? "Browse directory";
				},
				onClick: () => void browse(),
				get children() {
					return [createComponent$1(Icon, {
						source: folder,
						size: 14
					}), memo(() => {
						return memo(() => {
							return !!pending();
						})() ? local.pendingLabel ?? "Opening…" : local.browseLabel ?? "Browse…";
					})];
				}
			})];
		}
	});
}
//#endregion
//#region src/primitives/interactions/collection.ts
function createCollection(source) {
	const enabled = () => source().filter((item) => !item.disabled);
	const adjacent = (id, delta, loop) => {
		const items = enabled();
		if (items.length === 0) return void 0;
		const next = (id === void 0 ? -1 : items.findIndex((item) => item.id === id)) + delta;
		if (next >= 0 && next < items.length) return items[next];
		if (!loop) return void 0;
		return delta === 1 ? items[0] : items[items.length - 1];
	};
	return {
		items: source,
		find: (id) => source().find((item) => item.id === id),
		indexOf: (id) => source().findIndex((item) => item.id === id),
		first: () => enabled()[0],
		last: () => enabled().at(-1),
		next: (id, loop = false) => adjacent(id, 1, loop),
		previous: (id, loop = false) => adjacent(id, -1, loop)
	};
}
//#endregion
//#region src/primitives/interactions/machine.ts
function unchanged(state) {
	return {
		state,
		commands: []
	};
}
//#endregion
//#region src/primitives/interactions/state.ts
function createControllableState(options) {
	const [local, setLocal] = createSignal({ value: options.defaultValue });
	const value = () => options.value() ?? local().value;
	return {
		value,
		set(next) {
			if (options.disabled?.() || Object.is(value(), next)) return false;
			if (options.value() === void 0) setLocal({ value: next });
			options.onChange?.(next);
			return true;
		}
	};
}
//#endregion
//#region src/primitives/interactions/disclosure.ts
function updateDisclosure(state, event) {
	return match(event).with({ type: "DISABLED" }, ({ disabled }) => ({
		state: {
			...state,
			disabled,
			open: disabled ? false : state.open
		},
		commands: []
	})).with({ type: "OPEN" }, () => state.disabled || state.open ? unchanged(state) : {
		state: {
			...state,
			open: true
		},
		commands: []
	}).with({ type: "CLOSE" }, () => !state.open ? unchanged(state) : {
		state: {
			...state,
			open: false
		},
		commands: []
	}).with({ type: "TOGGLE" }, () => state.disabled ? unchanged(state) : {
		state: {
			...state,
			open: !state.open
		},
		commands: []
	}).exhaustive();
}
function createDisclosure(options = {}) {
	const controlled = createControllableState({
		value: options.open ?? (() => void 0),
		defaultValue: options.defaultOpen ?? false,
		disabled: options.disabled,
		onChange: options.onOpenChange
	});
	const set = (type) => {
		const result = updateDisclosure({
			open: controlled.value(),
			disabled: options.disabled?.() ?? false
		}, { type });
		return controlled.set(result.state.open);
	};
	return {
		open: controlled.value,
		disabled: () => options.disabled?.() ?? false,
		openDisclosure: () => set("OPEN"),
		close: () => set("CLOSE"),
		toggle: () => set("TOGGLE")
	};
}
//#endregion
//#region src/primitives/interactions/roving-focus.ts
function createRovingFocus(options = {}) {
	const items = [];
	const enabled = () => items.filter((item) => !item.disabled?.());
	return {
		register(item) {
			items.push(item);
			return () => {
				const index = items.indexOf(item);
				if (index >= 0) items.splice(index, 1);
			};
		},
		move(current, key) {
			const orientation = options.orientation?.() ?? "horizontal";
			const direction = match({
				orientation,
				key
			}).with({ key: "Home" }, () => "first").with({ key: "End" }, () => "last").with(P.union({
				orientation: "horizontal",
				key: "ArrowRight"
			}, {
				orientation: "vertical",
				key: "ArrowDown"
			}), () => "next").with(P.union({
				orientation: "horizontal",
				key: "ArrowLeft"
			}, {
				orientation: "vertical",
				key: "ArrowUp"
			}), () => "previous").otherwise(() => void 0);
			if (!direction) return false;
			const candidates = enabled();
			if (candidates.length === 0) return false;
			const index = candidates.findIndex((item) => item.id === current);
			const target = match(direction).with("first", () => candidates[0]).with("last", () => candidates.at(-1)).with("next", () => candidates[index + 1] ?? (options.loop === false ? void 0 : candidates[0])).with("previous", () => candidates[index - 1] ?? (options.loop === false ? void 0 : candidates.at(-1))).exhaustive();
			if (!target) return false;
			options.onMove?.(target.id);
			target.target.focus();
			return true;
		}
	};
}
//#endregion
//#region src/primitives/interactions/typeahead.ts
function createTypeahead(options = {}) {
	let keys = "";
	let timer;
	const collator = typeof Intl === "undefined" || typeof Intl.Collator !== "function" ? void 0 : new Intl.Collator(options.locale, {
		usage: "search",
		sensitivity: "base"
	});
	const reset = () => {
		keys = "";
		if (timer !== void 0) clearTimeout(timer);
		timer = void 0;
	};
	return {
		search(items, key, activeId) {
			if (key.length !== 1) return void 0;
			keys += key;
			if (timer !== void 0) clearTimeout(timer);
			timer = setTimeout(reset, options.timeout ?? 350);
			const query = keys.length > 1 && [...keys].every((value) => value === keys[0]) ? keys[0] : keys;
			const enabled = items.filter((item) => !item.disabled && item.textValue);
			const active = enabled.findIndex((item) => item.id === activeId);
			return [...enabled.slice(active + 1), ...enabled.slice(0, active + 1)].find((item) => {
				const prefix = item.textValue?.slice(0, query.length) ?? "";
				return collator ? collator.compare(prefix, query) === 0 : prefix.toLowerCase() === query.toLowerCase();
			});
		},
		reset
	};
}
//#endregion
//#region src/primitives/interactions/select.ts
function updateSelect(state, event, options) {
	const collection = createCollection(() => options.items);
	const openAt = (id) => ({
		state: {
			...state,
			open: true,
			highlighted: id
		},
		commands: [{ type: "FOCUS_CONTENT" }, ...id ? [{
			type: "SCROLL_TO_ITEM",
			id
		}] : []]
	});
	const move = (direction) => {
		const candidate = direction === "next" ? collection.next(state.highlighted, options.loop ?? true) : collection.previous(state.highlighted, options.loop ?? true);
		if (!candidate) return {
			state,
			commands: []
		};
		return {
			state: {
				...state,
				highlighted: candidate.id
			},
			commands: [{
				type: "SCROLL_TO_ITEM",
				id: candidate.id
			}]
		};
	};
	return match(event).with({ type: "OPEN" }, () => state.open ? {
		state,
		commands: []
	} : openAt(state.value ?? collection.first()?.id)).with({ type: "CLOSE" }, () => ({
		state: {
			...state,
			open: false,
			highlighted: void 0
		},
		commands: state.open ? [{ type: "FOCUS_TRIGGER" }] : []
	})).with({ type: "TOGGLE" }, () => state.open ? {
		state: {
			...state,
			open: false,
			highlighted: void 0
		},
		commands: [{ type: "FOCUS_TRIGGER" }]
	} : openAt(state.value ?? collection.first()?.id)).with({ type: "ARROW_DOWN" }, () => state.open ? move("next") : openAt(state.value ?? collection.first()?.id)).with({ type: "ARROW_UP" }, () => state.open ? move("previous") : openAt(state.value ?? collection.last()?.id)).with({ type: "HOME" }, () => openAt(collection.first()?.id)).with({ type: "END" }, () => openAt(collection.last()?.id)).with(P.union({ type: "HIGHLIGHT" }, { type: "TYPEAHEAD" }), ({ id }) => collection.find(id)?.disabled ? {
		state,
		commands: []
	} : event.type === "TYPEAHEAD" && !state.open ? openAt(id) : {
		state: {
			...state,
			highlighted: id
		},
		commands: [{
			type: "SCROLL_TO_ITEM",
			id
		}]
	}).with({ type: "SELECT" }, ({ id }) => {
		const selected = id ?? state.highlighted;
		if (!selected || collection.find(selected)?.disabled) return {
			state,
			commands: []
		};
		const close = options.closeOnSelect ?? true;
		return {
			state: {
				open: close ? false : state.open,
				value: selected,
				highlighted: close ? void 0 : selected
			},
			commands: close ? [{ type: "FOCUS_TRIGGER" }] : []
		};
	}).exhaustive();
}
function createSelectInteraction(options) {
	const value = createControllableState({
		value: options.value ?? (() => void 0),
		defaultValue: options.defaultValue,
		disabled: options.disabled,
		onChange: (next) => next && options.onValueChange?.(next)
	});
	const open = createControllableState({
		value: options.open ?? (() => void 0),
		defaultValue: options.defaultOpen ?? false,
		disabled: options.disabled,
		onChange: options.onOpenChange
	});
	const [highlighted, setHighlighted] = createSignal();
	const typeahead = createTypeahead();
	const state = () => ({
		open: open.value(),
		value: value.value(),
		highlighted: highlighted()
	});
	const send = (event) => {
		if (options.disabled?.()) return false;
		const result = updateSelect(state(), event, {
			items: options.items(),
			loop: options.loop
		});
		const previous = state();
		open.set(result.state.open);
		if (result.state.value !== void 0) value.set(result.state.value);
		setHighlighted(result.state.highlighted);
		for (const command of result.commands) options.execute?.(command);
		return previous.open !== result.state.open || previous.value !== result.state.value || previous.highlighted !== result.state.highlighted;
	};
	return {
		state,
		open: open.value,
		value: value.value,
		highlighted,
		send,
		typeahead(key) {
			const item = typeahead.search(options.items(), key, highlighted());
			return item ? send({
				type: "TYPEAHEAD",
				id: item.id
			}) : false;
		}
	};
}
//#endregion
//#region src/components/disclosure.tsx
function DisclosureIndicator(props) {
	const rotation = createTransition(() => props.open() ? Math.PI : 0, {
		duration: .2,
		ease: "easeOut",
		reducedMotion: props.reducedMotion
	});
	return createComponent$1(View, {
		class: "w-4 h-4 flex-none",
		get transform() {
			return rotate2d$1(rotation.value());
		},
		"aria-hidden": "true",
		get children() {
			return createComponent$1(Icon, {
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
	return createComponent$1(CollapsibleContext, {
		value: context,
		get children() {
			return createComponent$1(View, {
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
	return createComponent$1(Button$1, {
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
			return createComponent$1(View, {
				get ["class"]() {
					return join("w-full flex items-center justify-between gap-3", props.class);
				},
				get children() {
					return [memo(() => {
						return props.children;
					}), createComponent$1(DisclosureIndicator, {
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
	return createComponent$1(CollapsiblePresence, {
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
	return createComponent$1(AccordionContext, {
		value: {
			active: (item) => isSelected(state.value(), item),
			toggle: (item) => {
				state.set(nextAccordionValue(state.value(), type(), item, props.collapsible));
			},
			disabled: () => props.disabled ?? false,
			reducedMotion: () => props.reducedMotion ?? false
		},
		get children() {
			return createComponent$1(View, {
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
	return createComponent$1(AccordionItemContext, {
		get value() {
			return {
				value: props.value,
				disabled: () => props.disabled ?? false
			};
		},
		get children() {
			return createComponent$1(View, {
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
	return createComponent$1(Button$1, {
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
			return createComponent$1(View, {
				get ["class"]() {
					return join("w-full py-4 flex items-center justify-between gap-4", props.class);
				},
				get children() {
					return [createComponent$1(Text, {
						class: "min-w-0 whitespace-normal text-sm font-medium text-primary",
						get children() {
							return props.children;
						}
					}), createComponent$1(DisclosureIndicator, {
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
	return createComponent$1(CollapsiblePresence, {
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
//#region src/components/display.tsx
function Skeleton(props) {
	return createComponent$1(Pulse, {
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
	return createComponent$1(Spin, {
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
	return createComponent$1(Text, {
		get ["class"]() {
			return join("h-5 min-w-5 px-1 py-0.5 flex-none text-center rounded bg-control text-xs font-medium text-muted", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function KbdGroup(props) {
	return createComponent$1(View, {
		get ["class"]() {
			return join("inline-flex items-center gap-1", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
//#endregion
//#region src/components/menu-state.ts
/** Resolve one keyboard move without coupling menu state to rendering. */
function moveMenuHighlight(items, current, move) {
	const enabled = items.filter((item) => !item.disabled);
	if (enabled.length === 0) return void 0;
	if (move === "first") return enabled[0].id;
	if (move === "last") return enabled.at(-1)?.id;
	const index = enabled.findIndex((item) => item.id === current);
	if (move === "next") return enabled[(index + 1) % enabled.length].id;
	return enabled[(index <= 0 ? enabled.length : index) - 1].id;
}
//#endregion
//#region src/components/dropdown-menu.tsx
/** A compact action menu with native focus, typeahead, and overlay routing. */
function DropdownMenu(props) {
	const theme = useComponentsTheme();
	const [uncontrolledOpen, setUncontrolledOpen] = createSignal(props.defaultOpen ?? false);
	const [highlighted, setHighlighted] = createSignal();
	const open = () => props.open ?? uncontrolledOpen();
	const typeahead = createTypeahead();
	let trigger;
	let content;
	const setOpen = (next, edge = "first") => {
		if (next) setHighlighted(moveMenuHighlight(props.items, void 0, edge));
		else {
			setHighlighted(void 0);
			typeahead.reset();
		}
		if (props.open === void 0) setUncontrolledOpen(next);
		props.onOpenChange?.(next);
		if (next) requestAnimationFrame(() => content?.focus());
		else requestAnimationFrame(() => trigger?.focus());
	};
	const select = (id) => {
		const item = props.items.find((candidate) => candidate.id === id);
		if (!item || item.disabled) return false;
		item.onSelect?.();
		props.onAction?.(item.id);
		setOpen(false);
		return true;
	};
	const move = (direction) => {
		const next = moveMenuHighlight(props.items, highlighted(), direction);
		if (next === void 0) return false;
		setHighlighted(next);
		return true;
	};
	const handleMenuKey = (event) => {
		if (match(event.key).with("ArrowDown", () => move("next")).with("ArrowUp", () => move("previous")).with("Home", () => move("first")).with("End", () => move("last")).with("Enter", () => select(highlighted())).with(" ", () => select(highlighted())).with("Escape", () => {
			setOpen(false);
			return true;
		}).otherwise((key) => {
			const item = typeahead.search(props.items, key, highlighted());
			if (item) setHighlighted(item.id);
			return item !== void 0;
		})) event.preventDefault();
	};
	return createComponent$1(Popover, {
		contentRole: "presentation",
		popupRole: "menu",
		get open() {
			return open();
		},
		onOpenChange: (next) => setOpen(next),
		placement: "bottom-end",
		get contentClass() {
			return join("w-56 p-1 flex flex-col gap-1 rounded-lg border border-subtle bg-surface", props.contentClass);
		},
		get contentShadows() {
			return memo(() => {
				return props.contentShadows === void 0;
			})() ? componentsElevation(theme(), "floating") : props.contentShadows;
		},
		trigger: (popover) => props.trigger({
			ref: (node) => {
				trigger = node;
				popover.ref(node);
			},
			onClick: popover.onClick,
			onKeyDown: (event) => {
				popover.onKeyDown(event);
				if (match(event.key).with("ArrowDown", () => {
					setOpen(true, "first");
					return true;
				}).with("ArrowUp", () => {
					setOpen(true, "last");
					return true;
				}).otherwise(() => false)) event.preventDefault();
			},
			"aria-haspopup": "menu",
			get "aria-expanded"() {
				return open();
			}
		}),
		get children() {
			return createComponent$1(View, {
				ref: (node) => content = node,
				role: "menu",
				get ["aria-label"]() {
					return props["aria-label"];
				},
				focusOrder: 0,
				class: "min-w-0 flex flex-col gap-1",
				onKeyDown: handleMenuKey,
				get children() {
					return createComponent$1(For, {
						get each() {
							return props.items;
						},
						keyed: false,
						children: (item) => [memo(() => {
							return memo(() => {
								return !!item().separatorBefore;
							})() ? createComponent$1(View, {
								role: "presentation",
								class: "h-px flex-none my-1 bg-control"
							}) : item().separatorBefore;
						}), createComponent$1(View, {
							role: "menuitem",
							get ["aria-label"]() {
								return item().label;
							},
							get ["aria-disabled"]() {
								return item().disabled;
							},
							get ["class"]() {
								return join("w-full min-h-8 flex-none px-2 py-1.5 flex flex-col justify-center rounded-md", highlighted() === item().id ? "bg-control-hover" : "bg-transparent", item().destructive ? "text-danger-primary" : "text-primary");
							},
							get style() {
								return { opacity: item().disabled ? .45 : 1 };
							},
							onPointerMove: () => !item().disabled && setHighlighted(item().id),
							onClick: () => select(item().id),
							get children() {
								return [createComponent$1(Text, {
									class: "text-sm",
									get children() {
										return item().label;
									}
								}), memo(() => {
									return memo(() => {
										return !!item().description;
									})() ? createComponent$1(Text, {
										class: "text-xs text-muted",
										get children() {
											return item().description;
										}
									}) : item().description;
								})];
							}
						})]
					});
				}
			});
		}
	});
}
//#endregion
//#region src/components/forms.tsx
function Field(props) {
	const layout = () => match(props.orientation ?? "vertical").with("vertical", () => "flex-col gap-2").with("horizontal", () => "flex-row items-start gap-4").exhaustive();
	return createComponent$1(View, {
		get ["class"]() {
			return join("w-full flex", layout(), props.invalid && "text-danger-primary", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function FieldGroup(props) {
	return createComponent$1(View, {
		get ["class"]() {
			return join("flex flex-col gap-5", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function FieldLabel(props) {
	return createComponent$1(Text, {
		get ["class"]() {
			return join("text-sm font-medium text-primary", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function FieldContent(props) {
	return createComponent$1(View, {
		get ["class"]() {
			return join("min-w-0 flex-1 flex flex-col gap-1", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function FieldDescription(props) {
	return createComponent$1(Text, {
		get ["class"]() {
			return join("w-full min-w-0 whitespace-normal text-xs text-muted", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function FieldError(props) {
	return createComponent$1(Text, {
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
	return createComponent$1(View, mergeProps(() => {
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
	return createComponent$1(Input, mergeProps(props, { get ["class"]() {
		return join("flex-1 min-w-0 border-transparent bg-transparent", props.class);
	} }));
}
function InputGroupText(props) {
	return createComponent$1(Text, {
		get ["class"]() {
			return join("flex-none px-3 text-sm text-muted", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function InputGroupButton(props) {
	return createComponent$1(Button, mergeProps(props, {
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
	return createComponent$1(TextArea, mergeProps(props, { get ["class"]() {
		return join("w-full h-24 px-3 py-2 border-transparent bg-transparent text-sm", props.class);
	} }));
}
//#endregion
//#region src/components/layout.tsx
const emptyClass = (variant = "surface", className) => join("w-full min-w-0 p-8 items-center justify-center gap-4", variant === "surface" ? "min-h-64 rounded-lg border border-subtle bg-surface shadow-xs" : "min-h-0 bg-transparent", className);
function Empty(props) {
	return createComponent$1(Column, {
		get ["class"]() {
			return emptyClass(props.variant, props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function EmptyHeader(props) {
	return createComponent$1(Column, {
		get ["class"]() {
			return join("max-w-md items-center gap-2", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function EmptyMedia(props) {
	return createComponent$1(Center, {
		get ["class"]() {
			return join("w-12 h-12 flex-none rounded-lg bg-control text-secondary", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function EmptyTitle(props) {
	return createComponent$1(Text, {
		role: "heading",
		get ["class"]() {
			return join("text-base font-semibold text-primary", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function EmptyDescription(props) {
	return createComponent$1(Text, {
		get ["class"]() {
			return join("w-full min-w-0 whitespace-normal text-center text-sm text-muted", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function EmptyContent(props) {
	return createComponent$1(View, {
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
	return createComponent$1(View, {
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
	return createComponent$1(Text, {
		get ["class"]() {
			return join("px-2 text-sm text-muted", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
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
	return Math.min(maxColumns, Math.max(1, Math.floor((options.width + gap) / (minColumnWidth + gap))));
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
		initialColumns: props.initialColumns
	}));
	const rest = omit(props, "children", "minColumnWidth", "gap", "maxColumns", "initialColumns", "class", "ref");
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
				get ["class"]() {
					return join("w-full min-w-0 grid", responsiveGridColumnClass(columns()), props.class);
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
			get ["class"]() {
				return join("min-w-0", props.class);
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
		get ["class"]() {
			return join("w-full min-w-0 flex flex-row overflow-hidden", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function SplitPaneMain(props) {
	return createComponent$1(View, {
		get ["class"]() {
			return join("flex-1 min-w-0 overflow-hidden", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function SplitPaneAside(props) {
	return createComponent$1(View, {
		get ["class"]() {
			return join("flex-none min-w-0 overflow-hidden", props.class);
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
	return createComponent$1(AdaptiveSplitPaneContext, {
		value: { compact: () => props.compact },
		get children() {
			return createComponent$1(SplitPane, {
				get ["class"]() {
					return props.class;
				},
				get children() {
					return props.children;
				}
			});
		}
	});
}
function AdaptiveSplitPaneMain(props) {
	return createComponent$1(SplitPaneMain, {
		get ["class"]() {
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
				get ["class"]() {
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
				get ["aria-label"]() {
					return props["aria-label"];
				},
				get contentClass() {
					return join("h-11/12 p-0 overflow-hidden", props.modalClass);
				},
				get children() {
					return props.children;
				}
			});
		}
	});
}
//#endregion
//#region src/components/navigation.tsx
function Breadcrumb(props) {
	return createComponent$1(View, {
		role: "group",
		get ["aria-label"]() {
			return props["aria-label"] ?? "Breadcrumb";
		},
		get ["class"]() {
			return join("min-w-0", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function BreadcrumbList(props) {
	return createComponent$1(View, mergeProps(props, { get ["class"]() {
		return join("min-w-0 flex flex-wrap items-center gap-1.5 text-sm text-muted", props.class);
	} }));
}
function BreadcrumbItem(props) {
	return createComponent$1(View, mergeProps(props, { get ["class"]() {
		return join("min-w-0 flex items-center gap-1.5", props.class);
	} }));
}
function BreadcrumbLink(props) {
	return createComponent$1(Button$1, mergeProps(props, {
		unstyled: true,
		role: "link",
		class: (state) => join("min-w-0 rounded-sm text-sm text-secondary", state.hovered && "text-primary", state.focusVisible && "border border-focus", props.class)
	}));
}
function BreadcrumbPage(props) {
	return createComponent$1(Text, {
		role: "link",
		"aria-current": "page",
		get ["class"]() {
			return join("min-w-0 text-sm font-medium text-primary", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function BreadcrumbSeparator(props) {
	return createComponent$1(Text, {
		"aria-hidden": true,
		get ["class"]() {
			return join("flex-none text-xs text-muted", props.class);
		},
		get children() {
			return props.children ?? "/";
		}
	});
}
function BreadcrumbEllipsis(props) {
	return createComponent$1(Text, {
		"aria-hidden": true,
		get ["class"]() {
			return join("flex-none text-sm text-muted", props.class);
		},
		children: "..."
	});
}
function Pagination(props) {
	return createComponent$1(View, {
		role: "group",
		get ["aria-label"]() {
			return props["aria-label"] ?? "Pagination";
		},
		get ["class"]() {
			return join("flex items-center", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function PaginationContent(props) {
	return createComponent$1(View, mergeProps(props, { get ["class"]() {
		return join("flex items-center gap-1", props.class);
	} }));
}
function PaginationItem(props) {
	return createComponent$1(View, mergeProps(props, { get ["class"]() {
		return join("flex-none", props.class);
	} }));
}
function PaginationLink(props) {
	const forwarded = omit(props, "active");
	return createComponent$1(Button, mergeProps(forwarded, {
		role: "link",
		size: "icon",
		get variant() {
			return props.active ? "outline" : "ghost";
		},
		get selected() {
			return props.active;
		},
		get ["aria-current"]() {
			return props.active ? "page" : void 0;
		}
	}));
}
function PaginationPrevious(props) {
	return createComponent$1(Button, mergeProps(props, {
		variant: "ghost",
		size: "sm",
		get children() {
			return props.children ?? "Previous";
		}
	}));
}
function PaginationNext(props) {
	return createComponent$1(Button, mergeProps(props, {
		variant: "ghost",
		size: "sm",
		get children() {
			return props.children ?? "Next";
		}
	}));
}
//#endregion
//#region src/components/page.tsx
const pageViewportClass = (className) => join("min-w-0 min-h-0 flex-1", className);
const pageViewportContentClass = (className) => join("w-full h-full", className);
/**
* A full-height application page boundary.
*
* This composes native scrolling with an explicitly sized content wrapper and
* optional identity-based scroll reset. Page implementations can therefore
* focus on their own layout instead of reconstructing flex/overflow rules.
*/
function PageViewport(props) {
	let viewport;
	createScrollReset({
		target: () => viewport,
		key: () => props.resetKey
	});
	return createComponent$1(ScrollArea, {
		get ["class"]() {
			return pageViewportClass(props.class);
		},
		get contentClass() {
			return pageViewportContentClass(props.contentClass);
		},
		get style() {
			return props.style;
		},
		get scrollbar() {
			return props.scrollbar;
		},
		get onScroll() {
			return props.onScroll;
		},
		ref: (node) => {
			viewport = node;
			props.ref?.(node);
		},
		get children() {
			return props.children;
		}
	});
}
const pageHeaderClass = (className, stacked = false) => join("min-w-0 min-h-14 flex-none flex justify-between gap-4", stacked ? "flex-col items-stretch" : "flex-row items-center", className);
/** Consistent page title, supporting text and trailing application actions. */
function PageHeader(props) {
	return createComponent$1(View, {
		get ["class"]() {
			return pageHeaderClass(props.class, props.stacked);
		},
		get children() {
			return [createComponent$1(View, {
				class: "min-w-0 flex-1 flex flex-row items-center gap-3",
				get children() {
					return [createComponent$1(View, {
						class: "min-w-0 flex flex-col gap-1",
						get children() {
							return [createComponent$1(Text, {
								role: "heading",
								class: "whitespace-nowrap text-4xl font-bold",
								get children() {
									return props.title;
								}
							}), createComponent$1(Show, {
								get when() {
									return props.description;
								},
								children: (description) => createComponent$1(Text, {
									class: "truncate text-sm text-muted",
									get children() {
										return description();
									}
								})
							})];
						}
					}), memo(() => {
						return props.titleAdornment;
					})];
				}
			}), createComponent$1(Show, {
				get when() {
					return props.actions;
				},
				children: (actions) => createComponent$1(View, {
					class: "flex-none flex flex-row items-center gap-2",
					get classList() {
						return { "w-full": props.stacked };
					},
					get children() {
						return actions();
					}
				})
			})];
		}
	});
}
//#endregion
//#region src/components/select-semantics.ts
/** Keep semantic ID references live for the same lifetime as the popup node. */
function selectControlsId(listboxId, open) {
	return open ? listboxId : void 0;
}
//#endregion
//#region src/components/select.tsx
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
	return createComponent$1(Popover, {
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
		trigger: (popover) => createComponent$1(Button$1, {
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
			get ["aria-controls"]() {
				return selectControlsId(`${id}-listbox`, interaction.open());
			},
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
				return [createComponent$1(Text, {
					get ["class"]() {
						return join("min-w-0 flex-1 text-left truncate", selected() ? "text-primary" : "text-muted");
					},
					get children() {
						return selected()?.label ?? props.placeholder ?? "Select an option";
					}
				}), createComponent$1(Icon, {
					source: chevronDown,
					class: "flex-none text-muted",
					size: 16
				})];
			}
		}),
		get children() {
			return createComponent$1(ScrollArea, {
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
					return createComponent$1(View, {
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
							return createComponent$1(For, {
								get each() {
									return props.options;
								},
								keyed: false,
								children: (option) => {
									const selected = () => interaction.value() === option().value;
									const highlighted = () => interaction.highlighted() === option().value;
									return createComponent$1(View, {
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
											return [createComponent$1(Text, {
												class: "min-w-0 flex-1 text-sm whitespace-nowrap text-ellipsis",
												get children() {
													return option().label;
												}
											}), createComponent$1(View, {
												"aria-hidden": "true",
												class: "w-4 h-4 flex-none",
												get children() {
													return memo(() => {
														return !!selected();
													})() ? createComponent$1(Icon, {
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
//#region src/components/selection.tsx
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
	return createComponent$1(Button$1, {
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
			return [createComponent$1(Center, {
				"aria-hidden": "true",
				get ["class"]() {
					return join(SELECTION_INDICATOR_CLASS, "rounded text-xs font-bold", boxColors());
				},
				get children() {
					return memo(() => {
						return !!indicator();
					})() ? createComponent$1(Icon, {
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
				})() ? createComponent$1(Text, {
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
	return createComponent(RadioContext, {
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
			return createComponent$1(View, {
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
	return createComponent$1(Button$1, {
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
			return [createComponent$1(Center, {
				"aria-hidden": "true",
				get ["class"]() {
					return join(SELECTION_INDICATOR_CLASS, "rounded-full bg-input", match(checked()).with(true, () => "border-accent").with(false, () => "border-strong").exhaustive());
				},
				get children() {
					return memo(() => {
						return !!checked();
					})() ? createComponent$1(View, { class: "w-2.5 h-2.5 rounded-full bg-accent" }) : checked();
				}
			}), memo(() => {
				return memo(() => {
					return !!props.label;
				})() ? createComponent$1(Text, {
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
	return createComponent$1(Button$1, {
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
	return createComponent(ToggleGroupContext, {
		value: context,
		get children() {
			return createComponent$1(View, {
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
	return createComponent$1(Button$1, {
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
		class: (state) => join("h-7 flex-1 px-3 items-center justify-center rounded-sm border border-transparent text-sm font-medium", selected() ? props.variant === "accent" ? "bg-accent text-on-accent" : "bg-surface text-primary" : state.hovered ? "bg-control-hover text-primary" : "bg-transparent text-muted", state.focusVisible && "border-focus", props.class),
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
//#region src/components/slider.tsx
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
	return createComponent$1(View, {
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
			return [createComponent$1(View, {
				"aria-hidden": "true",
				class: "w-full h-1.5 overflow-hidden rounded-full border border-subtle bg-control",
				get children() {
					return createComponent$1(View, {
						class: "h-full rounded-full bg-accent",
						get style() {
							return { width: `${ratio() * 100}%` };
						}
					});
				}
			}), createComponent$1(View, {
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
//#region src/components/tabs.tsx
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
	return createComponent(TabsContext, {
		value: context,
		get children() {
			return createComponent$1(View, {
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
	return createComponent$1(View, {
		role: "tablist",
		get ["aria-label"]() {
			return props["aria-label"];
		},
		get ["aria-orientation"]() {
			return context.orientation();
		},
		get ["class"]() {
			return memo(() => {
				return !!props.unstyled;
			})() ? props.class : join("flex-none flex items-center gap-1", orientationClass(context.orientation(), "flex-row", "flex-col"), match(props.variant ?? "default").with("default", () => "p-0.5 rounded-md bg-control").with("line", () => "bg-transparent").exhaustive(), props.class);
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
	return createComponent$1(Button$1, {
		unstyled: true,
		role: "tab",
		get ["aria-label"]() {
			return props["aria-label"];
		},
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
		class: (state) => props.unstyled ? typeof props.class === "function" ? props.class(state) : props.class ?? "" : join("h-7 px-3 items-center justify-center rounded-sm border border-transparent text-sm font-medium", match({
			selected: selected(),
			hovered: state.hovered
		}).with({ selected: true }, () => "bg-surface text-primary shadow-xs").with({ hovered: true }, () => "bg-control-hover text-primary").otherwise(() => "bg-transparent text-muted"), state.focusVisible && "border-focus", typeof props.class === "function" ? props.class(state) : props.class),
		style: (state) => ({ opacity: state.disabled ? .45 : 1 }),
		onClick: () => context.select(props.value),
		onKeyDown: (event) => {
			if (context.move(props.value, event.key)) event.preventDefault();
		},
		get children() {
			return createComponent$1(Text, {
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
	return context.value() === props.value ? createComponent$1(View, {
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
//#region src/components/title-bar.tsx
function windowFrameBackdropClassList(maximized, rounded = true) {
	return { "p-3": rounded && !maximized };
}
function windowFrameClientClassList(maximized, rounded = true, classList = {}) {
	const decorated = rounded && !maximized;
	return {
		...classList,
		"rounded-xl": decorated,
		border: decorated,
		"border-subtle": decorated,
		"overflow-hidden": decorated
	};
}
/** Two restrained client-decoration layers sized to fit the 12px backdrop. */
function windowFrameShadows(theme) {
	const ambient = theme === "dark" ? 77 : 36;
	const contact = theme === "dark" ? 82 : 46;
	return [shadow({
		offsetY: 2,
		spread: -1,
		stdDev: 3,
		color: ambient,
		radius: 12
	}), shadow({
		offsetY: 1,
		spread: 0,
		stdDev: 1.5,
		color: contact,
		radius: 12
	})];
}
/**
* Root frame for an application-owned title bar and window chrome.
*
* Rounded outer corners require the native window to preserve alpha and the
* Rust host to clear with a transparent base color. Maximized windows are
* intentionally square so their content reaches every display edge.
*/
function WindowFrame(props) {
	const window = useWindow();
	const theme = useComponentsTheme();
	const decorated = () => props.rounded !== false && !window.maximized();
	return createComponent$1(View, {
		class: "w-full h-full bg-transparent",
		get classList() {
			return windowFrameBackdropClassList(window.maximized(), props.rounded !== false);
		},
		get children() {
			return createComponent$1(View, mergeProps(props, {
				get ["class"]() {
					return join("w-full h-full", props.class);
				},
				get classList() {
					return windowFrameClientClassList(window.maximized(), props.rounded !== false, props.classList);
				},
				get shadows() {
					return memo(() => {
						return props.shadows !== void 0;
					})() ? props.shadows : memo(() => {
						return !!decorated();
					})() ? windowFrameShadows(theme()) : null;
				}
			}));
		}
	});
}
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
	return createComponent$1(View, mergeProps(props, {
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
	return createComponent$1(View, mergeProps(props, {
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
//#region src/components/tooltip-state.ts
/** Owns tooltip timers independently from rendering and positioning. */
function createTooltipDelayController(options) {
	let timer;
	const cancel = () => {
		if (timer !== void 0) clearTimeout(timer);
		timer = void 0;
	};
	const schedule = (open, delay) => {
		cancel();
		if (delay <= 0) {
			options.setOpen(open);
			return;
		}
		timer = setTimeout(() => {
			timer = void 0;
			options.setOpen(open);
		}, delay);
	};
	return {
		scheduleOpen: () => schedule(true, options.openDelay()),
		scheduleClose: () => schedule(false, options.closeDelay()),
		openNow: () => {
			cancel();
			options.setOpen(true);
		},
		closeNow: () => {
			cancel();
			options.setOpen(false);
		},
		dispose: cancel
	};
}
//#endregion
//#region src/components/tooltip.tsx
let tooltipId = 0;
/** A delayed, non-interactive label for pointer and keyboard focus targets. */
function Tooltip(props) {
	const id = `wabou-tooltip-${++tooltipId}`;
	const [uncontrolledOpen, setUncontrolledOpen] = createSignal(props.defaultOpen ?? false);
	const open = () => !props.disabled && (props.open ?? uncontrolledOpen());
	const setOpen = (next) => {
		if (props.disabled) next = false;
		if (props.open === void 0) setUncontrolledOpen(next);
		props.onOpenChange?.(next);
	};
	const delay = createTooltipDelayController({
		openDelay: () => props.openDelay ?? 500,
		closeDelay: () => props.closeDelay ?? 80,
		setOpen
	});
	onCleanup(delay.dispose);
	return createComponent$1(Popover, {
		get open() {
			return open();
		},
		onOpenChange: (next) => !next && delay.closeNow(),
		get placement() {
			return props.placement ?? "top";
		},
		get offset() {
			return props.offset ?? 8;
		},
		contentRole: "presentation",
		popupRole: "tooltip",
		closeOnEscape: true,
		restoreFocus: false,
		get contentClass() {
			return join("max-w-xs rounded-md border border-subtle bg-surface px-2 py-1 shadow-md", props.contentClass);
		},
		trigger: (popover) => props.trigger({
			ref: popover.ref,
			onPointerEnter: delay.scheduleOpen,
			onPointerLeave: delay.scheduleClose,
			onFocus: delay.openNow,
			onBlur: delay.closeNow,
			onKeyDown: (event) => {
				if (event.key === "Escape") delay.closeNow();
			}
		}),
		get children() {
			return createComponent$1(Text, {
				id,
				role: "tooltip",
				class: "whitespace-normal text-xs text-primary",
				get children() {
					return props.children;
				}
			});
		}
	});
}
//#endregion
//#region src/components/index.tsx
function badgeColors(variant) {
	return match(variant).with("default", () => "bg-accent border-accent text-on-accent").with("secondary", () => "bg-control border-subtle text-primary").with("outline", () => "bg-transparent border-strong text-secondary").with("success", () => "bg-success-surface border-success-primary text-success-primary").with("destructive", () => "bg-danger-surface border-danger text-danger-primary").exhaustive();
}
function Badge(props) {
	return createComponent$1(Text, {
		get ["class"]() {
			return join("flex-none whitespace-nowrap px-2 py-0.5 rounded-md border text-xs", props.weight === "normal" ? "font-normal" : "font-medium", badgeColors(props.variant ?? "default"), props.class);
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
	return createComponent$1(Badge, {
		get variant() {
			return variant();
		},
		weight: "normal",
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
	return createComponent$1(View, {
		ref(r$) {
			var _ref$ = props.ref;
			typeof _ref$ === "function" || Array.isArray(_ref$) ? applyRef(_ref$, r$) : props.ref = r$;
		},
		get role() {
			return props.role;
		},
		get ["aria-label"]() {
			return props["aria-label"];
		},
		get ["aria-hidden"]() {
			return props["aria-hidden"];
		},
		get ["class"]() {
			return join("min-w-0 min-h-0 flex flex-col overflow-hidden rounded-lg border", "border-subtle bg-surface", props.class);
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
	return createComponent$1(View, {
		get ["class"]() {
			return join("min-w-0 flex flex-col gap-1 px-4 pt-4", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function CardTitle(props) {
	return createComponent$1(Text, mergeProps(props, {
		get ["class"]() {
			return join("min-w-0 text-base font-semibold", "text-primary", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function CardDescription(props) {
	return createComponent$1(Text, mergeProps(props, {
		get ["class"]() {
			return join("w-full min-w-0 whitespace-normal text-sm", "text-muted", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function CardContent(props) {
	return createComponent$1(View, {
		get ["class"]() {
			return join("min-w-0 min-h-0 flex flex-col gap-3 p-4", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function CardFooter(props) {
	return createComponent$1(View, {
		get ["class"]() {
			return join("min-w-0 flex items-center gap-2 px-4 pb-4", props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function Separator(props) {
	const dimensions = () => match(props.orientation ?? "horizontal").with("horizontal", () => "h-px w-full").with("vertical", () => "w-px h-full").exhaustive();
	return createComponent$1(View, {
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
	return createComponent$1(View, {
		role: "alert",
		get ["aria-label"]() {
			return props.title;
		},
		get ["class"]() {
			return join("flex flex-col gap-1 rounded-lg border p-4 shadow-xs", colors().container, props.class);
		},
		get children() {
			return [createComponent$1(Text, {
				get ["class"]() {
					return join("text-sm font-semibold", colors().title);
				},
				get children() {
					return props.title;
				}
			}), createComponent$1(Text, {
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
/** A native secret input whose value never crosses into JavaScript. */
function PasswordInput(props) {
	return createComponent$1(PasswordInput$1, mergeProps(props, { get ["class"]() {
		return join("h-8 w-full px-3 rounded-md border text-sm shadow-xs", "border-subtle bg-input text-primary", props.disabled && "opacity-50", props.class);
	} }));
}
function TextArea$1(props) {
	return createComponent$1(TextArea, mergeProps(props, { get ["class"]() {
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
	return createComponent$1(View, {
		get ["class"]() {
			return join("w-full min-w-0 flex items-center gap-3", props.class);
		},
		get children() {
			return [createComponent$1(Button$1, {
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
					return createComponent$1(View, {
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
				})() ? createComponent$1(Text, {
					class: "min-w-0 flex-1 whitespace-normal text-sm text-secondary",
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
	return createComponent$1(View, {
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
			return createComponent$1(View, {
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
//#region src/router/data.tsx
globalThis.scrollTo ??= () => {};
function createMutableStore(initial) {
	const [read, write] = createSignal(() => initial, { ownedWrite: true });
	return {
		get: read,
		set(next) {
			const value = typeof next === "function" ? next(read()) : next;
			write(() => value);
		}
	};
}
function createReadonlyStore(read) {
	return { get: read };
}
const solidStores = () => ({
	createMutableStore,
	createReadonlyStore,
	batch: flush
});
/** Create a TanStack Router Core instance adapted to Solid 2 and native history. */
function createDataRouter(options) {
	const history = options.history ?? createMemoryHistory$1({ initialEntries: ["/"] });
	const router = new RouterCore({
		isServer: false,
		origin: "wabou://app",
		...options,
		history
	}, solidStores);
	router.startTransition = async (commit) => {
		flush(commit);
		return true;
	};
	return router;
}
const DataRouterContext = createContext();
function requireDataRouter() {
	if (!getOwner()) throw new Error("Wabou data-router hooks must be used inside <RouterProvider>");
	const router = useContext(DataRouterContext);
	if (!router) throw new Error("Wabou data-router hooks must be used inside <RouterProvider>");
	return router;
}
function matchView(router, index) {
	const match = router.state.matches[index];
	if (!match) return void 0;
	const options = router.routesById[match.routeId].options;
	return match.status === "error" ? options.errorComponent : match.status === "notFound" ? options.notFoundComponent : match.status === "pending" ? options.pendingComponent : options.component;
}
/** Preserve a matched component while its route and selected view are stable. */
function RouteMatch(props) {
	const match = () => props.router.state.matches[props.index];
	let outlet;
	return createComponent(Show, {
		get when() {
			return matchView(props.router, props.index);
		},
		keyed: true,
		children: (view) => createComponent(view, {
			get error() {
				return match()?.error;
			},
			get children() {
				return outlet ??= createComponent(RouteOutlet, {
					router: props.router,
					index: props.index + 1
				});
			}
		})
	});
}
/** Key only the route level that changed instead of rebuilding all matches. */
function RouteOutlet(props) {
	return createComponent(Show, {
		get when() {
			return props.router.state.matches[props.index]?.routeId;
		},
		keyed: true,
		get fallback() {
			return props.fallback ?? null;
		},
		children: (_routeId) => createComponent(RouteMatch, {
			router: props.router,
			index: props.index
		})
	});
}
/** Own router lifecycle and render its current native component branch. */
function RouterProvider(props) {
	const router = untrack(() => props.router);
	let disposed = false;
	let loadScheduled = false;
	const scheduleLoad = () => {
		if (disposed || loadScheduled) return;
		loadScheduled = true;
		Promise.resolve().then(async () => {
			loadScheduled = false;
			if (!disposed) await router.load();
		}).catch((error) => {
			console.error(`[wabou-router] route load failed: ${String(error)}`);
		});
	};
	const unsubscribe = router.history.subscribe(scheduleLoad);
	onCleanup(() => {
		disposed = true;
		unsubscribe();
	});
	scheduleLoad();
	return createComponent(DataRouterContext, {
		value: router,
		get children() {
			return createComponent(RouteOutlet, {
				router,
				index: 0,
				get fallback() {
					return props.fallback;
				}
			});
		}
	});
}
function useRouter() {
	return requireDataRouter();
}
function useRouterState(selector) {
	const router = requireDataRouter();
	return createMemo(() => selector(router));
}
function useNavigate() {
	return requireDataRouter().navigate;
}
function useLocation() {
	const router = requireDataRouter();
	return createMemo(() => router.state.location);
}
/**
* Reactively report whether a native router destination is active.
*
* This delegates path, base-path, parameter, and trailing-slash behavior to
* Router Core instead of duplicating pathname comparisons in navigation UI.
*/
function useRouteActive(to, options = {}) {
	const router = requireDataRouter();
	const exact = options.exact ?? to === "/";
	return createMemo(() => router.matchRoute({ to }, {
		fuzzy: !exact,
		includeSearch: options.includeSearch ?? false,
		pending: options.pending
	}) !== false);
}
function useParams() {
	const router = requireDataRouter();
	return createMemo(() => router.state.matches.at(-1)?.params ?? {});
}
function useLoaderData() {
	const router = requireDataRouter();
	return createMemo(() => router.state.matches.at(-1)?.loaderData);
}
//#endregion
export { Accordion, AccordionContent, AccordionItem, AccordionTrigger, AdaptiveSplitPane, AdaptiveSplitPaneDetail, AdaptiveSplitPaneMain, Alert, Avatar, AvatarGroup, AvatarGroupCount, Badge, BaseRootRoute, BaseRoute, Breadcrumb, BreadcrumbEllipsis, BreadcrumbItem, BreadcrumbLink, BreadcrumbList, BreadcrumbPage, BreadcrumbSeparator, Button, ButtonGroup, ButtonGroupText, Calendar, CalendarDate, Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle, Center, Checkbox, CodeEditor, Collapsible, CollapsibleContent, CollapsiblePresence, CollapsibleTrigger, Column, ComponentsProvider, ConfigEditor, DatePicker, Dialog, DialogDescription, DialogFooter, DialogHeader, DialogScrollBody, DialogTitle, DirectoryPicker, DropdownMenu, Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle, Field, FieldContent, FieldDescription, FieldError, FieldGroup, FieldLabel, Fps, Icon, Image, Input, InputGroup, InputGroupButton, InputGroupInput, InputGroupText, InputGroupTextArea, Kbd, KbdGroup, Modal, NetworkImage, NotificationRegion, OverlayPlaneProvider, PageHeader, PageViewport, Pagination, PaginationContent, PaginationItem, PaginationLink, PaginationNext, PaginationPrevious, PasswordInput, Path, PathBuilder, Popover, Button$1 as PrimitiveButton, Link as PrimitiveLink, PasswordInput$1 as PrimitivePasswordInput, TextArea as PrimitiveTextArea, TextInput as PrimitiveTextInput, Progress, Pulse, RadioGroup, RadioGroupItem, ResponsiveGrid, ResponsiveGridRemainder, Ripple, RouterProvider, Row, ScrollArea, Select, Separator, Skeleton, Slider, Spin, Spinner, SplitPane, SplitPaneAside, SplitPaneMain, Svg, Switch, Tabs, TabsContent, TabsList, TabsTrigger, Text, TextArea$1 as TextArea, TitleBar, TitleBarDragRegion, Toggle, ToggleGroup, ToggleGroupItem, Tooltip, View, WindowFrame, animate, animateKeyframes, componentsElevation, createActive, createAnimationFrame, createButton, createContainerMatch, createDataRouter, createFocus, createFocusWithin, createFormDraft, createHover, createKeyedSelection, createLoop, createMeasuredSize, createMemoryHistory, createNotifications, createOverlayLayer, createPresence, createPress, createPulse, createRotation, createScrollReset, createShortcuts, createTabs, createTooltipDelayController, createTransition, emptyClass, moveMenuHighlight, nextAccordionValue, notFound, pageHeaderClass, pageViewportClass, pageViewportContentClass, primitives_exports as primitives, redirect, responsiveGridColumnCount, responsiveGridRemainderCount, titleBarClass, titleBarDragRegionLayoutStyle, titleBarLayoutStyle, useComponentsTheme, useLoaderData, useLocation, useNavigate, useParams, useResponsiveGrid, useRouteActive, useRouter, useRouterState, windowFrameBackdropClassList, windowFrameClientClassList, windowFrameShadows };

//# sourceMappingURL=index.mjs.map