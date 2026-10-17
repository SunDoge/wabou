import { Icon, Text, View, rotate2d as rotate2d$1, translate2d as translate2d$1 } from "../primitives/view.mjs";
import { createMeasuredSize } from "../primitives/measure.mjs";
import { Button } from "./button.mjs";
import { Toolbar, ToolbarButton } from "./toolbar.mjs";
import { Tooltip } from "./tooltip.mjs";
import { mergeClasses } from "@wabou/core/style";
import { For, Show, createContext, createEffect, createSignal, omit, onCleanup, untrack, useContext } from "solid-js";
import { applyRef, createComponent as createComponent$1, memo, mergeProps, useHost } from "@wabou/core/renderer";
import arrowDown from "lucide-static/icons/arrow-down.svg?raw";
//#region src/components/message-scroller.tsx
function messageScrollRange(contentHeight, viewportHeight) {
	return Math.max(0, contentHeight - viewportHeight);
}
function isMessageScrollNearEnd(scrollY, contentHeight, viewportHeight, threshold = 24) {
	return scrollY >= messageScrollRange(contentHeight, viewportHeight) - Math.max(0, threshold);
}
/** Smallest vertical delta that reveals a target without disturbing visible content. */
function messageScrollRevealDelta(viewport, target, margin = 12) {
	const inset = Math.max(0, margin);
	const visibleTop = viewport.y + inset;
	const visibleBottom = viewport.y + viewport.height - inset;
	if (target.y < visibleTop) return target.y - visibleTop;
	const targetBottom = target.y + target.height;
	if (targetBottom > visibleBottom) return targetBottom - visibleBottom;
	return 0;
}
/** Vertical delta that places a target at the viewport's reading start. */
function messageScrollStartDelta(viewport, target, margin = 12) {
	return target.y - (viewport.y + Math.max(0, margin));
}
const MessageScrollerContext = createContext(null);
function requireMessageScroller() {
	const context = useContext(MessageScrollerContext);
	if (!context) throw new Error("MessageScroller components require a MessageScroller root");
	return context;
}
function useMessageScroller() {
	return requireMessageScroller();
}
/** Pick the last conversation anchor that has crossed the reading line. */
function activeMessageAnchor(viewport, anchors, offset = 64) {
	if (anchors.length === 0) return void 0;
	const line = viewport.y + Math.min(Math.max(0, offset), viewport.height / 3);
	const ordered = [...anchors].sort((first, second) => first.rect.y === second.rect.y ? first.rect.x - second.rect.x : first.rect.y - second.rect.y);
	let active = ordered[0]?.id;
	for (const anchor of ordered) {
		if (anchor.rect.y > line) break;
		active = anchor.id;
	}
	return active;
}
function MessageScroller(props) {
	const host = useHost();
	const forwarded = omit(props, "followEnd", "endThreshold", "class", "children");
	const [scrollY, setScrollY] = createSignal(0);
	const [followingEnd, setFollowingEnd] = createSignal(untrack(() => props.followEnd ?? true));
	const [activeAnchor, setActiveAnchor] = createSignal();
	const threshold = () => Math.max(0, props.endThreshold ?? 24);
	let viewport;
	let endFrame;
	let anchorFrame;
	const anchors = /* @__PURE__ */ new Map();
	let measuredAnchors = [];
	const newestAnchor = () => [...anchors.keys()].at(-1);
	const updateActiveFromScroll = (position = scrollY()) => {
		if (followingEnd() && isMessageScrollNearEnd(position, contentSize.height(), viewportSize.height(), threshold())) {
			const last = measuredAnchors.reduce((current, anchor) => !current || anchor.rect.y > current.rect.y ? anchor : current, void 0);
			setActiveAnchor(last?.id ?? newestAnchor());
			return;
		}
		setActiveAnchor(activeMessageAnchor({
			x: 0,
			y: position,
			width: viewportSize.width(),
			height: viewportSize.height()
		}, measuredAnchors));
	};
	const measureAnchors = () => {
		if (!viewport || anchors.size === 0) {
			measuredAnchors = [];
			setActiveAnchor(followingEnd() ? newestAnchor() : void 0);
			return;
		}
		const targets = [...anchors.entries()];
		const snapshot = host.layout.snapshot([viewport, ...targets.map(([, node]) => node)]);
		const nodeByKey = new Map(snapshot.nodes.map((node) => [`${node.id.lo}:${node.id.hi}`, node]));
		const viewportMetrics = nodeByKey.get(`${viewport.id.lo}:${viewport.id.hi}`);
		if (!viewportMetrics) return;
		measuredAnchors = targets.flatMap(([id, node]) => {
			const metrics = nodeByKey.get(`${node.id.lo}:${node.id.hi}`);
			return metrics ? [{
				id,
				rect: {
					x: metrics.rect.x - viewportMetrics.rect.x,
					y: metrics.rect.y - viewportMetrics.rect.y + scrollY(),
					width: metrics.rect.width,
					height: metrics.rect.height
				}
			}] : [];
		});
		updateActiveFromScroll();
	};
	const scheduleAnchorMeasure = () => {
		if (anchorFrame !== void 0) cancelAnimationFrame(anchorFrame);
		anchorFrame = requestAnimationFrame(() => {
			anchorFrame = void 0;
			measureAnchors();
		});
	};
	const scheduleEnd = (force = false) => {
		if (!force && !followingEnd() || !viewport) return;
		if (endFrame !== void 0) cancelAnimationFrame(endFrame);
		endFrame = requestAnimationFrame(() => {
			endFrame = void 0;
			viewport?.scrollTo({ top: Number.MAX_SAFE_INTEGER });
		});
	};
	const geometryChanged = () => {
		scheduleEnd();
		scheduleAnchorMeasure();
	};
	createEffect(() => props.followEnd, (next, previous) => {
		if (next === void 0 || next === previous) return;
		setFollowingEnd(next);
		if (next) scheduleEnd(true);
		else if (endFrame !== void 0) {
			cancelAnimationFrame(endFrame);
			endFrame = void 0;
		}
	});
	const viewportSize = createMeasuredSize({ onChange: geometryChanged });
	const contentSize = createMeasuredSize({ onChange: geometryChanged });
	const range = () => messageScrollRange(contentSize.height(), viewportSize.height());
	const nearEnd = () => isMessageScrollNearEnd(scrollY(), contentSize.height(), viewportSize.height(), threshold());
	const context = {
		followingEnd,
		canScrollStart: () => scrollY() > threshold(),
		canScrollEnd: () => range() > 0 && !nearEnd(),
		activeAnchor,
		scrollTo: (direction) => {
			setFollowingEnd(direction === "end");
			viewport?.scrollTo({ top: direction === "end" ? Number.MAX_SAFE_INTEGER : 0 });
		},
		scrollIntoView: (target, options) => {
			if (!viewport) return;
			const snapshot = host.layout.snapshot([viewport, target]);
			const viewportMetrics = snapshot.nodes.find((node) => node.id.lo === viewport?.id.lo && node.id.hi === viewport?.id.hi);
			const targetMetrics = snapshot.nodes.find((node) => node.id.lo === target.id.lo && node.id.hi === target.id.hi);
			if (!viewportMetrics || !targetMetrics) return;
			const delta = options?.align === "start" ? messageScrollStartDelta(viewportMetrics.rect, targetMetrics.rect, options.margin) : messageScrollRevealDelta(viewportMetrics.rect, targetMetrics.rect, options?.margin);
			if (delta === 0) return;
			setFollowingEnd(false);
			viewport.scrollBy({ top: delta });
		},
		scrollToAnchor: (anchor, options) => {
			const target = anchors.get(anchor);
			if (target) context.scrollIntoView(target, options);
		},
		setViewport: (node) => {
			viewport = node;
			viewportSize.ref(node);
			scheduleEnd();
			scheduleAnchorMeasure();
		},
		setContent: (node) => {
			contentSize.ref(node);
			scheduleAnchorMeasure();
		},
		handleScroll: (event) => {
			const next = Math.max(0, event.scrollY ?? 0);
			setScrollY(next);
			setFollowingEnd(isMessageScrollNearEnd(next, contentSize.height(), viewportSize.height(), threshold()));
			updateActiveFromScroll(next);
		},
		registerAnchor: (anchor, node) => {
			anchors.set(anchor, node);
			if (followingEnd()) setActiveAnchor(anchor);
			scheduleAnchorMeasure();
		},
		unregisterAnchor: (anchor, node) => {
			const current = anchors.get(anchor);
			if (current?.id.lo === node.id.lo && current.id.hi === node.id.hi) {
				anchors.delete(anchor);
				scheduleAnchorMeasure();
			}
		}
	};
	onCleanup(() => {
		if (endFrame !== void 0) cancelAnimationFrame(endFrame);
		if (anchorFrame !== void 0) cancelAnimationFrame(anchorFrame);
	});
	return createComponent$1(MessageScrollerContext, {
		value: context,
		get children() {
			return createComponent$1(View, mergeProps(forwarded, {
				get "class"() {
					return mergeClasses("relative w-full h-full min-w-0 min-h-0 flex flex-col overflow-hidden", props.class);
				},
				get transform() {
					return props.transform ?? translate2d$1(-16, 0);
				},
				get children() {
					return props.children;
				}
			}));
		}
	});
}
function MessageScrollerViewport(props) {
	const context = requireMessageScroller();
	const forwarded = omit(props, "class", "children", "ref", "onScroll");
	return createComponent$1(View, mergeProps(forwarded, {
		ref: (node) => {
			context.setViewport(node);
			props.ref?.(node);
		},
		get "class"() {
			return mergeClasses("w-full min-w-0 min-h-0 flex-1 overflow-x-hidden overflow-y-auto", props.class);
		},
		get scrollbar() {
			return props.scrollbar ?? { visibility: "auto" };
		},
		onScroll: (event) => {
			context.handleScroll(event);
			props.onScroll?.(event);
		},
		get children() {
			return props.children;
		}
	}));
}
function MessageScrollerContent(props) {
	const context = requireMessageScroller();
	const forwarded = omit(props, "class", "children", "ref");
	return createComponent$1(View, mergeProps(forwarded, {
		ref: (node) => {
			context.setContent(node);
			props.ref?.(node);
		},
		get "class"() {
			return mergeClasses("w-full min-w-0 min-h-full flex-none flex flex-col gap-4", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function MessageScrollerItem(props) {
	const context = useContext(MessageScrollerContext);
	const forwarded = omit(props, "anchor", "class", "children", "ref");
	let node;
	let registered;
	createEffect(() => props.anchor, (next) => {
		if (node && registered && registered !== next) {
			context?.unregisterAnchor(registered, node);
			registered = void 0;
		}
		if (node && next && next !== registered) {
			context?.registerAnchor(next, node);
			registered = next;
		}
	});
	onCleanup(() => {
		if (node && registered) context?.unregisterAnchor(registered, node);
	});
	return createComponent$1(View, mergeProps(forwarded, {
		ref: (handle) => {
			node = handle;
			const anchor = props.anchor;
			if (anchor) {
				context?.registerAnchor(anchor, handle);
				registered = anchor;
			}
			props.ref?.(handle);
		},
		get "data-message-anchor"() {
			return props.anchor;
		},
		get "class"() {
			return mergeClasses("w-full min-w-0 flex-none", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function MessageScrollerButton(props) {
	const context = requireMessageScroller();
	const direction = () => props.direction ?? "end";
	const active = () => direction() === "end" ? context.canScrollEnd() : context.canScrollStart();
	const forwarded = omit(props, "direction", "class", "children", "onClick");
	const label = () => direction() === "end" ? "Scroll to end" : "Scroll to start";
	return createComponent$1(Show, {
		get when() {
			return active();
		},
		get children() {
			return createComponent$1(Button, mergeProps(forwarded, {
				get "aria-label"() {
					return props["aria-label"] ?? label();
				},
				get variant() {
					return props.variant ?? "secondary";
				},
				get size() {
					return props.size ?? "icon";
				},
				get "class"() {
					return mergeClasses("absolute z-10 left-1/2 flex-none rounded-full shadow-sm", direction() === "end" ? "bottom-3" : "top-3", props.class);
				},
				onClick: (event) => {
					context.scrollTo(direction());
					props.onClick?.(event);
				},
				get children() {
					return props.children ?? createComponent$1(Icon, {
						"aria-hidden": "true",
						source: arrowDown,
						size: 16,
						get transform() {
							return memo(() => {
								return direction() === "start";
							})() ? rotate2d$1(Math.PI) : void 0;
						}
					});
				}
			}));
		}
	});
}
/** Compact anchor rail for navigating long retained conversations. */
function MessageScrollerNavigator(props) {
	const context = requireMessageScroller();
	const currentAnchor = () => context.followingEnd() ? props.items.at(-1)?.id : context.activeAnchor();
	const reveal = (id) => {
		context.scrollToAnchor(id, {
			margin: 24,
			align: "start"
		});
	};
	return createComponent$1(Show, {
		get when() {
			return props.items.length >= (props.minItems ?? 2);
		},
		get children() {
			return createComponent$1(View, {
				get "class"() {
					return mergeClasses("absolute z-20 right-2 top-4 bottom-14 w-8 flex flex-col items-center justify-center pointer-events-none", props.class);
				},
				get children() {
					return createComponent$1(Toolbar, {
						get "aria-label"() {
							return props["aria-label"];
						},
						orientation: "vertical",
						get "class"() {
							return mergeClasses("max-h-full p-0 py-1 gap-0 rounded-full bg-surface shadow-xs overflow-y-auto pointer-events-auto", props.railClass);
						},
						get children() {
							return createComponent$1(For, {
								get each() {
									return props.items;
								},
								keyed: false,
								children: (item, index) => createComponent$1(Tooltip, {
									placement: "left",
									openDelay: 240,
									contentClass: "max-w-sm",
									trigger: (tooltip) => createComponent$1(ToolbarButton, {
										ref(r$) {
											var _ref$ = tooltip.ref;
											typeof _ref$ === "function" || Array.isArray(_ref$) ? applyRef(_ref$, r$) : tooltip.ref = r$;
										},
										variant: "ghost",
										size: "icon",
										class: "w-7 h-7 p-0 rounded-full",
										get "aria-label"() {
											return props.itemAriaLabel(item(), index);
										},
										get "aria-current"() {
											return currentAnchor() === item().id ? "step" : void 0;
										},
										get onPointerEnter() {
											return tooltip.onPointerEnter;
										},
										get onPointerLeave() {
											return tooltip.onPointerLeave;
										},
										get onFocus() {
											return tooltip.onFocus;
										},
										get onBlur() {
											return tooltip.onBlur;
										},
										get onKeyDown() {
											return tooltip.onKeyDown;
										},
										onClick: () => reveal(item().id),
										get children() {
											return createComponent$1(View, {
												"aria-hidden": "true",
												get "class"() {
													return currentAnchor() === item().id ? "w-4 h-1 rounded-full bg-accent" : "w-3 h-1 rounded-full bg-subtle";
												}
											});
										}
									}),
									get children() {
										return createComponent$1(Text, {
											class: "text-xs text-primary whitespace-normal",
											get children() {
												return item().label;
											}
										});
									}
								})
							});
						}
					});
				}
			});
		}
	});
}
//#endregion
export { MessageScroller, MessageScrollerButton, MessageScrollerContent, MessageScrollerItem, MessageScrollerNavigator, MessageScrollerViewport, activeMessageAnchor, isMessageScrollNearEnd, messageScrollRange, messageScrollRevealDelta, messageScrollStartDelta, useMessageScroller };

//# sourceMappingURL=message-scroller.mjs.map