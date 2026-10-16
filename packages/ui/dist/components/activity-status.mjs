import { Text, View } from "../primitives/view.mjs";
import { Pulse } from "../primitives/motion.mjs";
import { componentToneClass } from "./tone.mjs";
import { mergeClasses } from "@wabou/core/style";
import { Show, omit } from "solid-js";
import { createComponent as createComponent$1, mergeProps } from "@wabou/core/renderer";
//#region src/components/activity-status.tsx
/** A consistently sized status dot. Animation is explicit so idle state never
* keeps the native frame clock alive accidentally. */
function ActivityStatusIndicator(props) {
	const className = () => mergeClasses("w-1.5 h-1.5 flex-none rounded-full", componentToneClass(props.tone === "muted" ? "neutral" : props.tone ?? "accent", "indicator"), props.class);
	return createComponent$1(Show, {
		get when() {
			return props.animated;
		},
		get fallback() {
			return createComponent$1(View, {
				"aria-hidden": "true",
				get "class"() {
					return className();
				}
			});
		},
		get children() {
			return createComponent$1(Pulse, {
				"aria-hidden": "true",
				get "class"() {
					return className();
				},
				from: .3,
				to: 1,
				duration: .8
			});
		}
	});
}
/** Shrink-safe inline progress or presence status for agent and desktop UI. */
function ActivityStatus(props) {
	const forwarded = omit(props, "label", "animated", "tone", "class", "textClass");
	return createComponent$1(View, mergeProps(forwarded, {
		get role() {
			return props.role ?? "status";
		},
		get "aria-label"() {
			return props["aria-label"] ?? props.label;
		},
		get "class"() {
			return mergeClasses("min-w-0 max-w-full flex flex-row items-center gap-1.5", props.class);
		},
		get children() {
			return [createComponent$1(ActivityStatusIndicator, {
				get animated() {
					return props.animated;
				},
				get tone() {
					return props.tone;
				}
			}), createComponent$1(Text, {
				get "class"() {
					return mergeClasses("min-w-0 truncate text-xs text-muted", props.textClass);
				},
				get children() {
					return props.label;
				}
			})];
		}
	}));
}
//#endregion
export { ActivityStatus, ActivityStatusIndicator };

//# sourceMappingURL=activity-status.mjs.map