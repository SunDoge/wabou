import { useReducedMotion } from "../animation/config.mjs";
import "../animation.mjs";
import { View, createInternalPrimitive } from "./view.mjs";
import { createTransitionPresence } from "./transition-presence.mjs";
import { createRetainedItems } from "./retained-items.mjs";
import { translate2d } from "@wabou/core/style";
import { For, createComponent, createEffect, createSignal, onCleanup, untrack } from "solid-js";
import { Portal } from "@wabou/core/renderer";
//#region src/primitives/notification.ts
const finiteNonNegative = (value, fallback) => Number.isFinite(value) ? Math.max(0, value) : fallback;
/** Create an owner-scoped notification queue with explicit JavaScript timers. */
function createNotifications(options = {}) {
	const records = /* @__PURE__ */ new Map();
	const [revision, setRevision] = createSignal(0, { ownedWrite: true });
	const items = () => {
		revision();
		return [...records.values()].map((record) => record.item);
	};
	const defaultDuration = finiteNonNegative(options.defaultDuration ?? 5e3, 5e3);
	const configuredLimit = options.limit ?? 5;
	const limit = Number.isFinite(configuredLimit) ? Math.max(1, Math.floor(configuredLimit)) : 5;
	let nextId = 1;
	const dismiss = (id, reason = "programmatic") => {
		const record = records.get(id);
		if (!record) return false;
		if (record.timer !== void 0) clearTimeout(record.timer);
		records.delete(id);
		setRevision((current) => current + 1);
		record.item.onDismiss?.(reason);
		return true;
	};
	const schedule = (record) => {
		if (!record.autoDismiss) return;
		if (record.remaining <= 0) {
			dismiss(record.item.id, "timeout");
			return;
		}
		record.startedAt = Date.now();
		record.timer = setTimeout(() => dismiss(record.item.id, "timeout"), record.remaining);
	};
	const notifications = {
		items,
		show(input) {
			while (records.size >= limit) {
				const oldest = records.values().next().value?.item;
				if (!oldest || !dismiss(oldest.id, "overflow")) break;
			}
			const duration = finiteNonNegative(input.duration ?? defaultDuration, defaultDuration);
			const item = {
				...input,
				id: nextId++,
				duration,
				priority: input.priority ?? "polite"
			};
			const record = {
				item,
				autoDismiss: duration > 0,
				pauseCount: 0,
				remaining: duration,
				startedAt: 0
			};
			records.set(item.id, record);
			setRevision((current) => current + 1);
			schedule(record);
			return item.id;
		},
		dismiss,
		pause(id) {
			const record = records.get(id);
			if (!record) return;
			record.pauseCount++;
			if (record.pauseCount > 1 || record.timer === void 0) return;
			clearTimeout(record.timer);
			record.timer = void 0;
			record.remaining = Math.max(0, record.remaining - (Date.now() - record.startedAt));
		},
		resume(id) {
			const record = records.get(id);
			if (!record || record.pauseCount === 0) return;
			record.pauseCount--;
			if (record.pauseCount > 0 || record.timer !== void 0) return;
			schedule(record);
		},
		clear() {
			for (const id of [...records.keys()]) dismiss(id, "programmatic");
		}
	};
	onCleanup(() => {
		for (const record of records.values()) if (record.timer !== void 0) clearTimeout(record.timer);
		records.clear();
	});
	return notifications;
}
const alignment = (placement) => ({
	"align-items": placement.endsWith("start") ? "flex-start" : placement.endsWith("end") ? "flex-end" : "center",
	"justify-content": placement.startsWith("bottom") ? "flex-end" : "flex-start"
});
const renderNotificationPortal = (props, children) => {
	const stack = createInternalPrimitive("toast-stack", {
		get placement() {
			return props.placement ?? "top-end";
		},
		get class() {
			return props.stackClass ?? "w-96 max-w-full";
		},
		children
	});
	return createComponent(Portal, {
		plane: "floating",
		role: "presentation",
		get class() {
			return `pointer-events-none ${props.class ?? ""}`;
		},
		get style() {
			const placement = props.placement ?? "top-end";
			return {
				position: "absolute",
				left: 0,
				top: 0,
				width: "100%",
				height: "100%",
				display: "flex",
				"flex-direction": "column",
				gap: 8,
				padding: 16,
				...alignment(placement),
				...props.style
			};
		},
		children: stack
	});
};
/** Render a non-blocking stack on the native floating overlay plane. */
function NotificationRegion(props) {
	const motion = untrack(() => props.motion);
	if (motion === void 0 || motion === false) {
		const items = createComponent(For, {
			get each() {
				return props.notifications.items();
			},
			children: (item) => createComponent(View, {
				role: item.priority === "assertive" ? "alert" : "status",
				"aria-label": item["aria-label"],
				get class() {
					return `pointer-events-auto ${props.itemClass ?? ""}`;
				},
				get style() {
					return props.itemStyle;
				},
				onPointerEnter: () => props.notifications.pause(item.id),
				onPointerLeave: () => props.notifications.resume(item.id),
				onFocusIn: () => props.notifications.pause(item.id),
				onFocusOut: () => props.notifications.resume(item.id),
				get children() {
					return item.content({ dismiss: () => props.notifications.dismiss(item.id, "dismiss") });
				}
			})
		});
		return renderNotificationPortal(props, items);
	}
	const reducedMotion = useReducedMotion();
	const retained = createRetainedItems(props.notifications.items, (item) => item.id);
	const renderAnimatedItem = (retainedItem) => {
		const logicallyPresent = retainedItem.present;
		const duration = motion.duration ?? .18;
		const presence = createTransitionPresence(logicallyPresent, {
			initialProgress: 0,
			duration,
			ease: motion.ease ?? "easeOut",
			reducedMotion
		});
		createEffect(() => presence.phase(), (phase) => {
			if (phase === "unmounted") retained.release(retainedItem.key);
		});
		return createComponent(View, {
			get role() {
				return retainedItem.value().priority === "assertive" ? "alert" : "status";
			},
			get "aria-label"() {
				return retainedItem.value()["aria-label"];
			},
			get "aria-hidden"() {
				return logicallyPresent() ? void 0 : "true";
			},
			get interactionBlocked() {
				return !logicallyPresent();
			},
			get transform() {
				const remaining = 1 - presence.progress();
				return translate2d((motion.fromX ?? 0) * remaining, (motion.fromY ?? 0) * remaining);
			},
			get class() {
				return `pointer-events-auto ${props.itemClass ?? ""}`;
			},
			get style() {
				return {
					...props.itemStyle,
					opacity: presence.progress()
				};
			},
			onPointerEnter: () => props.notifications.pause(retainedItem.key),
			onPointerLeave: () => props.notifications.resume(retainedItem.key),
			onFocusIn: () => props.notifications.pause(retainedItem.key),
			onFocusOut: () => props.notifications.resume(retainedItem.key),
			get children() {
				const item = retainedItem.value();
				return item.content({ dismiss: () => props.notifications.dismiss(item.id, "dismiss") });
			}
		});
	};
	const items = createComponent(For, {
		get each() {
			return retained.entries();
		},
		children: renderAnimatedItem
	});
	return renderNotificationPortal(props, items);
}
//#endregion
export { NotificationRegion, createNotifications };

//# sourceMappingURL=notification.mjs.map