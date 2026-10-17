import { subscribeHostMessages, subscribeJsonHostMessages } from "@wabou/core";
import { createSignal, onCleanup } from "solid-js";
//#region src/components/development-diagnostics.ts
/** Errors survive unrelated successful reloads and disappear on their own recovery. */
function useDevelopmentDiagnostics() {
	const errors = /* @__PURE__ */ new Map();
	const [diagnostic, setDiagnostic] = createSignal();
	const refresh = () => setDiagnostic([...errors.values()].at(-1));
	const onError = (error) => console.error("[wabou-hmr] invalid development diagnostic", error);
	const unsubscribeError = subscribeJsonHostMessages("wabou:dev-server-error", (value) => {
		errors.set("js:Vite", value);
		refresh();
	}, {
		decode(value) {
			if (!value || typeof value !== "object" || typeof value.message !== "string") throw new TypeError("diagnostic must contain a message");
			return value;
		},
		onError
	});
	const unsubscribeReady = subscribeHostMessages("wabou:dev-server-ready", () => {
		for (const key of errors.keys()) if (key.startsWith("js:")) errors.delete(key);
		refresh();
	});
	const unsubscribeStatus = subscribeJsonHostMessages("wabou:reload-status", (status) => {
		const key = `${status.source}:${status.target}`;
		if (status.phase === "error") errors.set(key, {
			message: status.error ?? "Replacement failed",
			id: status.target,
			plugin: status.source === "rust" ? "Rust" : "JavaScript"
		});
		else if (status.phase === "ready") errors.delete(key);
		refresh();
	}, {
		decode(value) {
			const status = value;
			if (!status || !["rust", "js"].includes(status.source) || typeof status.target !== "string" || ![
				"building",
				"ready",
				"error"
			].includes(status.phase) || status.error != null && typeof status.error !== "string") throw new TypeError("invalid reload status");
			return status;
		},
		onError
	});
	onCleanup(() => {
		unsubscribeError();
		unsubscribeReady();
		unsubscribeStatus();
	});
	return {
		diagnostic,
		dismiss: () => {
			errors.clear();
			refresh();
		}
	};
}
//#endregion
export { useDevelopmentDiagnostics };

//# sourceMappingURL=development-diagnostics.mjs.map