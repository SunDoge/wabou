import { Icon, Text, View } from "../primitives/view.mjs";
import { componentsDisabledControlClass } from "./theme.mjs";
import { useFileDrop, useHost } from "@wabou/core";
import { mergeClasses } from "@wabou/core/style";
import { Show, createEffect, createSignal } from "solid-js";
import { createComponent as createComponent$1, memo, mergeProps } from "@wabou/core/renderer";
import fileUp from "lucide-static/icons/file-up.svg?raw";
//#region src/components/drop-zone.tsx
/** Inclusive hit test in the logical coordinate space shared with native DnD. */
function pointInLayoutRect(point, rect) {
	return point.x >= rect.x && point.y >= rect.y && point.x <= rect.x + rect.width && point.y <= rect.y + rect.height;
}
/**
* A component-local target for native filesystem drops.
*
* Native backends publish window-relative coordinates, so the zone measures
* its completed native layout before claiming an event. Events without a
* position are deliberately ignored instead of being delivered ambiguously to
* every mounted zone.
*/
function DropZone(props) {
	const host = useHost();
	const [active, setActive] = createSignal(false);
	let target;
	const contains = (position) => {
		if (!position || !target) return false;
		const rect = host.layout.measure(target);
		return rect !== null && pointInLayoutRect(position, rect);
	};
	useFileDrop((event) => {
		if (props.disabled) {
			setActive(false);
			return;
		}
		if (event.phase === "left") {
			setActive(false);
			return;
		}
		const inside = contains(event.position);
		if (event.phase !== "dropped") {
			setActive(inside);
			return;
		}
		setActive(false);
		if (!inside) return;
		const accepted = props.accept ? event.paths.filter((path) => props.accept?.(path)) : event.paths;
		const rejected = props.accept ? event.paths.filter((path) => !props.accept?.(path)) : [];
		if (accepted.length > 0) props.onDrop(accepted);
		if (rejected.length > 0) props.onRejected?.(rejected);
	});
	createEffect(() => props.disabled, (disabled) => {
		if (disabled) setActive(false);
	});
	const surface = (isActive) => createComponent$1(View, mergeProps(props, {
		ref: (node) => {
			target = node;
		},
		role: "group",
		get "aria-label"() {
			return props.label ?? "Drop files";
		},
		get "aria-disabled"() {
			return props.disabled ? "true" : void 0;
		},
		get "class"() {
			return mergeClasses("w-full min-w-0 min-h-36 px-6 py-5 flex flex-col items-center justify-center gap-3 rounded-xl border-2 text-center", isActive ? "border-accent bg-selected" : "border-strong bg-input", props.class, componentsDisabledControlClass(props.disabled ?? false));
		},
		get children() {
			return [
				createComponent$1(View, {
					get "class"() {
						return mergeClasses("w-10 h-10 flex-none items-center justify-center rounded-lg", isActive ? "bg-accent text-on-accent" : "bg-control text-secondary");
					},
					get children() {
						return createComponent$1(Icon, {
							source: fileUp,
							size: 20
						});
					}
				}),
				createComponent$1(Text, {
					class: "text-sm font-semibold text-primary",
					get children() {
						return memo(() => {
							return !!active();
						})() ? props.activeLabel ?? "Release to add files" : props.label ?? "Drop files here";
					}
				}),
				createComponent$1(Text, {
					class: "max-w-md text-xs text-muted whitespace-normal",
					get children() {
						return props.description ?? "Drag files from the desktop into this area.";
					}
				})
			];
		}
	}));
	return createComponent$1(Show, {
		get when() {
			return active();
		},
		get fallback() {
			return surface(false);
		},
		get children() {
			return surface(true);
		}
	});
}
//#endregion
export { DropZone, pointInLayoutRect };

//# sourceMappingURL=drop-zone.mjs.map