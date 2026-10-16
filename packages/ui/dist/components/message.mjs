import { createFocusWithin } from "../primitives/focus.mjs";
import { createHover } from "../primitives/hover.mjs";
import { Text, View, translate2d as translate2d$1 } from "../primitives/view.mjs";
import { mergeClasses } from "@wabou/core/style";
import { Show, createContext, omit, useContext } from "solid-js";
import { createComponent as createComponent$1, memo, mergeProps } from "@wabou/core/renderer";
import { match } from "ts-pattern";
//#region src/components/message.tsx
const MessageContext = createContext({
	align: () => "start",
	interacting: () => false
});
const BubbleContext = createContext({
	align: () => "start",
	interacting: () => false,
	variant: () => "default"
});
function MessageGroup(props) {
	return createComponent$1(View, mergeProps(props, {
		get role() {
			return props.role ?? "group";
		},
		get "class"() {
			return mergeClasses("w-full min-w-0 flex flex-col gap-3", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function messageClass(align = "start", className) {
	return mergeClasses("relative w-full min-w-0 flex gap-2 text-sm", align === "end" ? "flex-row-reverse" : "flex-row", className);
}
function Message(props) {
	const hover = createHover();
	const focus = createFocusWithin();
	const forwarded = omit(props, "align", "class", "children", "onPointerEnter", "onPointerLeave", "onFocusIn", "onFocusOut");
	const context = {
		align: () => props.align ?? "start",
		interacting: () => hover.hovered() || focus.focusWithin()
	};
	return createComponent$1(MessageContext, {
		value: context,
		get children() {
			return createComponent$1(View, mergeProps(forwarded, {
				get role() {
					return props.role ?? "group";
				},
				get "class"() {
					return messageClass(context.align(), props.class);
				},
				onPointerEnter: (event) => {
					hover.bindings.onPointerEnter();
					props.onPointerEnter?.(event);
				},
				onPointerLeave: (event) => {
					hover.bindings.onPointerLeave();
					props.onPointerLeave?.(event);
				},
				onFocusIn: (event) => {
					focus.bindings.onFocusIn();
					props.onFocusIn?.(event);
				},
				onFocusOut: (event) => {
					focus.bindings.onFocusOut();
					props.onFocusOut?.(event);
				},
				get children() {
					return props.children;
				}
			}));
		}
	});
}
function MessageAvatar(props) {
	return createComponent$1(View, mergeProps(props, {
		get "class"() {
			return mergeClasses("w-8 h-8 flex-none self-end overflow-hidden rounded-full bg-control flex items-center justify-center", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function MessageContent(props) {
	const context = useContext(MessageContext);
	return createComponent$1(View, mergeProps(props, {
		get "class"() {
			return mergeClasses("flex-1 min-w-0 flex flex-col gap-2", context.align() === "end" ? "items-end" : "items-start", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function MessageHeader(props) {
	const context = useContext(MessageContext);
	return createComponent$1(Text, mergeProps(props, {
		get "class"() {
			return mergeClasses("max-w-full min-w-0 px-3 text-xs font-medium text-muted", context.align() === "end" && "text-right", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
const MessageFooter = MessageHeader;
function messageActionsClass(align = "start", className) {
	return mergeClasses("h-7 px-1 flex flex-row items-center gap-1", align === "end" ? "self-end justify-end" : "self-start justify-start", className);
}
/** Compact, consistently aligned actions belonging to one message. */
function MessageActions(props) {
	const context = useContext(MessageContext);
	const forwarded = omit(props, "align", "visibility", "class", "children");
	const align = () => props.align ?? context.align();
	const ownerAlign = () => context.align();
	const interactionClass = () => {
		if (props.visibility !== "interaction") return "opacity-100";
		return mergeClasses("absolute top-0 left-0 z-10 w-full", ownerAlign() === "end" ? "justify-start" : "justify-end", context.interacting() ? "opacity-100" : "opacity-0 pointer-events-none");
	};
	return createComponent$1(View, mergeProps(forwarded, {
		get role() {
			return props.role ?? "toolbar";
		},
		get "class"() {
			return messageActionsClass(align(), mergeClasses(interactionClass(), props.class));
		},
		get children() {
			return props.children;
		}
	}));
}
function BubbleGroup(props) {
	return createComponent$1(View, mergeProps(props, {
		get "class"() {
			return mergeClasses("w-full min-w-0 grid gap-2", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function bubbleClass(variant = "default", align = "start", className) {
	return mergeClasses("relative min-w-0 flex flex-col gap-1", variant === "ghost" ? "max-w-full" : "max-w-4/5", align === "end" ? "self-end items-end" : "self-start items-start", className);
}
function Bubble(props) {
	const message = useContext(MessageContext);
	const forwarded = omit(props, "variant", "align", "class", "children");
	const context = {
		variant: () => props.variant ?? "default",
		align: () => props.align ?? message.align(),
		interacting: message.interacting
	};
	return createComponent$1(BubbleContext, {
		value: context,
		get children() {
			return createComponent$1(View, mergeProps(forwarded, {
				get "class"() {
					return bubbleClass(context.variant(), context.align(), props.class);
				},
				get children() {
					return props.children;
				}
			}));
		}
	});
}
function bubbleContentClass(variant, className) {
	const colors = match(variant).with("default", () => "border-transparent bg-accent text-on-accent").with("secondary", () => "border-transparent bg-control text-primary").with("muted", () => "border-transparent bg-control text-secondary").with("tinted", () => "border-transparent bg-selected text-primary").with("outline", () => "border-subtle bg-surface text-primary").with("ghost", () => "border-transparent bg-transparent text-primary").with("destructive", () => "border-danger bg-danger-surface text-danger-primary").exhaustive();
	return mergeClasses("max-w-full min-w-0 overflow-hidden rounded-xl border", variant === "ghost" ? "p-0" : "px-3 py-2", colors, className);
}
function BubbleContent(props) {
	const context = useContext(BubbleContext);
	return createComponent$1(View, mergeProps(props, {
		get "class"() {
			return bubbleContentClass(context.variant(), props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function BubbleReactions(props) {
	const bubble = useContext(BubbleContext);
	const side = () => props.side ?? "bottom";
	const align = () => props.align ?? bubble.align();
	return createComponent$1(View, {
		get "class"() {
			return mergeClasses("relative z-10 flex-none flex items-center justify-center gap-1 rounded-full bg-control px-1.5 py-0.5 text-sm", align() === "end" ? "self-end" : "self-start", props.class);
		},
		get transform() {
			return translate2d$1(0, side() === "top" ? 4 : -4);
		},
		get children() {
			return props.children;
		}
	});
}
function Marker(props) {
	const variant = () => props.variant ?? "default";
	return createComponent$1(View, {
		get "class"() {
			return mergeClasses("w-full min-w-0 min-h-4 flex items-center gap-2 text-sm text-muted", variant() === "border" && "border-b border-subtle pb-2", props.class);
		},
		get children() {
			return [
				createComponent$1(Show, {
					get when() {
						return variant() === "separator";
					},
					get children() {
						return createComponent$1(View, {
							"aria-hidden": "true",
							class: "flex-1 min-w-0 h-px bg-subtle"
						});
					}
				}),
				memo(() => {
					return props.children;
				}),
				createComponent$1(Show, {
					get when() {
						return variant() === "separator";
					},
					get children() {
						return createComponent$1(View, {
							"aria-hidden": "true",
							class: "flex-1 min-w-0 h-px bg-subtle"
						});
					}
				})
			];
		}
	});
}
function MarkerIcon(props) {
	return createComponent$1(View, mergeProps(props, {
		"aria-hidden": "true",
		get "class"() {
			return mergeClasses("w-4 h-4 flex-none", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function MarkerContent(props) {
	return createComponent$1(Text, mergeProps(props, {
		get "class"() {
			return mergeClasses("min-w-0 whitespace-nowrap text-sm text-muted", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
//#endregion
export { Bubble, BubbleContent, BubbleGroup, BubbleReactions, Marker, MarkerContent, MarkerIcon, Message, MessageActions, MessageAvatar, MessageContent, MessageFooter, MessageGroup, MessageHeader, bubbleClass, bubbleContentClass, messageActionsClass, messageClass };

//# sourceMappingURL=message.mjs.map