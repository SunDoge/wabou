import { Text, View } from "../primitives/view.mjs";
import { Button } from "./button.mjs";
import { mergeClasses } from "@wabou/core/style";
import { createContext, omit, useContext } from "solid-js";
import { createComponent as createComponent$1, mergeProps } from "@wabou/core/renderer";
import { match } from "ts-pattern";
//#region src/components/attachment.tsx
const AttachmentContext = createContext({
	state: () => "done",
	size: () => "default",
	orientation: () => "horizontal"
});
function attachmentClass(options) {
	const state = options.state ?? "done";
	const size = options.size ?? "default";
	const orientation = options.orientation ?? "horizontal";
	return mergeClasses("max-w-full min-w-0 flex-none flex border bg-surface text-primary", match(orientation).with("horizontal", () => "min-w-40 flex-row flex-wrap items-center").with("vertical", () => "w-28 flex-col items-stretch").exhaustive(), match(size).with("default", () => "gap-2 rounded-xl p-2 text-sm").with("sm", () => "gap-2 rounded-lg p-1.5 text-xs").with("xs", () => "gap-1.5 rounded-md p-1 text-xs").exhaustive(), match(state).with("idle", () => "border-strong").with("uploading", () => "border-focus").with("processing", () => "border-accent").with("error", () => "border-danger bg-danger-surface").with("done", () => "border-subtle").exhaustive(), options.class);
}
/** File/task summary anatomy adapted from shadcn without DOM data selectors. */
function Attachment(props) {
	const forwarded = omit(props, "state", "size", "orientation", "class", "children");
	const context = {
		state: () => props.state ?? "done",
		size: () => props.size ?? "default",
		orientation: () => props.orientation ?? "horizontal"
	};
	return createComponent$1(AttachmentContext, {
		value: context,
		get children() {
			return createComponent$1(View, mergeProps(forwarded, {
				get "class"() {
					return attachmentClass({
						state: context.state(),
						size: context.size(),
						orientation: context.orientation(),
						class: props.class
					});
				},
				get children() {
					return props.children;
				}
			}));
		}
	});
}
function attachmentMediaClass(variant, context, className) {
	const size = context.size();
	const orientation = context.orientation();
	const state = context.state();
	return mergeClasses("aspect-square flex-none overflow-hidden flex items-center justify-center rounded-lg", orientation === "vertical" ? "w-full" : match(size).with("default", () => "w-10").with("sm", () => "w-8").with("xs", () => "w-7").exhaustive(), state === "error" ? "bg-danger-surface text-danger-primary" : "bg-control text-primary", variant === "image" && state !== "done" && "opacity-60", className);
}
function AttachmentMedia(props) {
	const context = useContext(AttachmentContext);
	return createComponent$1(View, {
		get "class"() {
			return attachmentMediaClass(props.variant ?? "icon", context, props.class);
		},
		get children() {
			return props.children;
		}
	});
}
function AttachmentContent(props) {
	return createComponent$1(View, mergeProps(props, {
		get "class"() {
			return mergeClasses("max-w-full min-w-0 flex-1 flex flex-col gap-0.5", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function AttachmentTitle(props) {
	return createComponent$1(Text, mergeProps(props, {
		get maxLines() {
			return props.maxLines ?? 1;
		},
		get "class"() {
			return mergeClasses("max-w-full min-w-0 font-medium text-primary", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function AttachmentDescription(props) {
	const context = useContext(AttachmentContext);
	return createComponent$1(Text, mergeProps(props, {
		get maxLines() {
			return props.maxLines ?? 1;
		},
		get "class"() {
			return mergeClasses("max-w-full min-w-0 text-xs", context.state() === "error" ? "text-danger-primary" : "text-muted", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function AttachmentActions(props) {
	return createComponent$1(View, mergeProps(props, {
		get "class"() {
			return mergeClasses("flex-none flex items-center gap-1", props.class);
		},
		get children() {
			return props.children;
		}
	}));
}
function AttachmentAction(props) {
	return createComponent$1(Button, mergeProps(props, {
		get variant() {
			return props.variant ?? "ghost";
		},
		get size() {
			return props.size ?? "sm";
		}
	}));
}
function attachmentGroupClass(className) {
	return mergeClasses("w-full min-w-0 overflow-x-auto overflow-y-hidden py-1 flex flex-row items-start gap-3", className);
}
function AttachmentGroup(props) {
	return createComponent$1(View, mergeProps(props, {
		get "class"() {
			return attachmentGroupClass(props.class);
		},
		get scrollbar() {
			return props.scrollbar ?? { visibility: "hidden" };
		},
		get children() {
			return props.children;
		}
	}));
}
//#endregion
export { Attachment, AttachmentAction, AttachmentActions, AttachmentContent, AttachmentDescription, AttachmentGroup, AttachmentMedia, AttachmentTitle, attachmentClass, attachmentGroupClass, attachmentMediaClass };

//# sourceMappingURL=attachment.mjs.map