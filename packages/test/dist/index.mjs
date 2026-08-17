import { defaultHost } from "@wabou/solid-renderer";
//#region src/replay.ts
/** Execute a recorded trace against explicit page and window capabilities. */
async function replayActions(actions, page, window) {
	for (const action of actions) if (action.action === "nativeClose") await window.nativeClose(action.windowId, action.platform);
	else if (action.action === "showWindow") await window.show(action.windowId);
	else if (action.action === "clickByRole") await page.forWindow(action.windowId).getByRole(action.role, { name: action.label }).click();
	else {
		const locator = page.forWindow(action.windowId).getByRole(action.role, { name: action.label });
		const input = action.input;
		if (input.type === "probe") await locator.waitFor();
		else if (input.type === "drag") await locator.dragBy(input.deltaX, input.deltaY);
		else if (input.type === "key") await locator.press(input.key, {
			shift: (input.modifiers & 1) !== 0,
			control: (input.modifiers & 2) !== 0,
			alt: (input.modifiers & 4) !== 0,
			meta: (input.modifiers & 8) !== 0
		});
		else if (input.type === "text") await locator.type(input.text);
		else if (input.type === "paste") await locator.paste(input.text);
		else if (input.type === "ime") await locator.ime(input.text);
		else await locator.wheel(input.deltaY, input.deltaX);
	}
}
//#endregion
//#region src/index.ts
const tests = [];
const trace = [];
function capability() {
	const value = defaultHost.test;
	if (!value) throw new Error("@wabou/test requires `wabou test`");
	return value;
}
function createPage(windowId) {
	if (!Number.isSafeInteger(windowId) || windowId <= 0) throw new RangeError(`invalid Wabou window id ${windowId}`);
	return {
		forWindow(nextWindowId) {
			return createPage(nextWindowId);
		},
		async waitForIdle() {
			await Promise.resolve();
			await new Promise((resolve) => requestAnimationFrame(() => resolve()));
			if (!await capability().waitForIdle(windowId)) throw new Error(`native window ${windowId} did not become idle`);
		},
		getByRole(role, options) {
			const input = async (value) => {
				trace.push({
					action: "inputByRole",
					windowId,
					role,
					label: options.name,
					input: value
				});
				if (!await capability().inputByRole(windowId, role, options.name, JSON.stringify(value))) throw new Error(`no enabled ${role} named ${JSON.stringify(options.name)}`);
			};
			const snapshot = async () => {
				await input({ type: "probe" });
				const value = JSON.parse(capability().takeQueryResult());
				if (!value) throw new Error(`no semantic snapshot for ${role} named ${JSON.stringify(options.name)}`);
				return value;
			};
			return {
				windowId,
				async click() {
					trace.push({
						action: "clickByRole",
						windowId,
						role,
						label: options.name
					});
					if (!await capability().clickByRole(windowId, role, options.name)) throw new Error(`no enabled ${role} named ${JSON.stringify(options.name)}`);
				},
				dragBy(deltaX, deltaY) {
					return input({
						type: "drag",
						deltaX,
						deltaY
					});
				},
				press(key, modifiers = {}) {
					const bits = (modifiers.shift ? 1 : 0) | (modifiers.control ? 2 : 0) | (modifiers.alt ? 4 : 0) | (modifiers.meta ? 8 : 0);
					return input({
						type: "key",
						key,
						modifiers: bits
					});
				},
				type(text) {
					return input({
						type: "text",
						text
					});
				},
				paste(text) {
					return input({
						type: "paste",
						text
					});
				},
				ime(text) {
					return input({
						type: "ime",
						text
					});
				},
				wheel(deltaY, deltaX = 0) {
					return input({
						type: "wheel",
						deltaX,
						deltaY
					});
				},
				waitFor() {
					return input({ type: "probe" });
				},
				snapshot
			};
		}
	};
}
const context = {
	page: createPage(1),
	window: {
		async nativeClose(windowId, platform) {
			trace.push({
				action: "nativeClose",
				windowId,
				platform
			});
			if (!await capability().nativeClose(windowId, platform !== "wayland")) throw new Error(`failed to enqueue native close for window ${windowId}`);
		},
		async show(windowId) {
			trace.push({
				action: "showWindow",
				windowId
			});
			if (!await capability().showWindow(windowId)) throw new Error(`failed to enqueue show for window ${windowId}`);
		},
		state(windowId) {
			return JSON.parse(capability().windowState(windowId));
		}
	}
};
function test(name, body) {
	tests.push({
		name,
		body
	});
}
/** Register a previously recorded action trace as a behavior test. */
function replay(actions) {
	test("replay action trace", async ({ window }) => {
		await replayActions(actions, context.page, window);
	});
}
function expect(actual) {
	const locator = () => {
		if (!actual || typeof actual !== "object" || !("snapshot" in actual)) throw new Error("this assertion requires a Wabou locator");
		return actual;
	};
	const locatorSnapshot = async () => {
		return locator().snapshot();
	};
	const eventually = async (assertion, options = {}) => {
		const timeout = options.timeout ?? 1e3;
		const interval = options.interval ?? 16;
		const deadline = performance.now() + timeout;
		let diagnostic = "locator state did not match";
		do {
			await createPage(locator().windowId).waitForIdle();
			const failure = assertion(await locatorSnapshot());
			if (failure === null) return;
			diagnostic = failure;
			if (performance.now() < deadline) await new Promise((resolve) => setTimeout(resolve, interval));
		} while (performance.now() < deadline);
		throw new Error(`${diagnostic} after ${timeout}ms`);
	};
	return {
		toBe(expected) {
			if (!Object.is(actual, expected)) throw new Error(`expected ${JSON.stringify(actual)} to be ${JSON.stringify(expected)}`);
		},
		toEqual(expected) {
			const left = JSON.stringify(actual);
			const right = JSON.stringify(expected);
			if (left !== right) throw new Error(`expected ${left} to equal ${right}`);
		},
		toHaveText(expected, options) {
			return eventually((state) => {
				const value = state.value ?? state.name;
				return value === expected ? null : `expected locator text ${JSON.stringify(value)} to be ${JSON.stringify(expected)}`;
			}, options);
		},
		toHaveValue(expected, options) {
			return eventually((state) => state.value === expected ? null : `expected locator value ${JSON.stringify(state.value)} to be ${JSON.stringify(expected)}`, options);
		},
		toBeDisabled(options) {
			return eventually((state) => state.disabled ? null : "expected locator to be disabled", options);
		},
		toBeEnabled(options) {
			return eventually((state) => state.disabled ? "expected locator to be enabled" : null, options);
		},
		toBeChecked(options) {
			return eventually((state) => state.checked === true ? null : `expected locator to be checked, received ${JSON.stringify(state.checked)}`, options);
		},
		toBeUnchecked(options) {
			return eventually((state) => state.checked === false ? null : `expected locator to be unchecked, received ${JSON.stringify(state.checked)}`, options);
		},
		toBeIndeterminate(options) {
			return eventually((state) => state.checked === "mixed" ? null : `expected locator to be indeterminate, received ${JSON.stringify(state.checked)}`, options);
		},
		toBeSelected(options) {
			return eventually((state) => state.selected === true ? null : `expected locator to be selected, received ${JSON.stringify(state.selected)}`, options);
		},
		toBeDeselected(options) {
			return eventually((state) => state.selected === false ? null : `expected locator to be deselected, received ${JSON.stringify(state.selected)}`, options);
		},
		toBeExpanded(options) {
			return eventually((state) => state.expanded === true ? null : `expected locator to be expanded, received ${JSON.stringify(state.expanded)}`, options);
		},
		toBeCollapsed(options) {
			return eventually((state) => state.expanded === false ? null : `expected locator to be collapsed, received ${JSON.stringify(state.expanded)}`, options);
		},
		toBePressed(options) {
			return eventually((state) => state.pressed === true ? null : `expected locator to be pressed, received ${JSON.stringify(state.pressed)}`, options);
		},
		toBeUnpressed(options) {
			return eventually((state) => state.pressed === false ? null : `expected locator to be unpressed, received ${JSON.stringify(state.pressed)}`, options);
		},
		toBeFocused(options) {
			return eventually((state) => state.focused ? null : "expected locator to be focused", options);
		},
		toBeBlurred(options) {
			return eventually((state) => state.focused ? "expected locator to be blurred" : null, options);
		}
	};
}
expect.poll = function poll(read, options = {}) {
	const timeout = options.timeout ?? 1e3;
	const interval = options.interval ?? 10;
	return { async toBe(expected) {
		const deadline = performance.now() + timeout;
		let actual = read();
		while (!Object.is(actual, expected) && performance.now() < deadline) {
			await new Promise((resolve) => setTimeout(resolve, interval));
			actual = read();
		}
		if (!Object.is(actual, expected)) throw new Error(`expected ${JSON.stringify(actual)} to become ${JSON.stringify(expected)}`);
	} };
};
async function run() {
	const results = [];
	for (const entry of tests) try {
		await entry.body(context);
		results.push({
			name: entry.name,
			passed: true
		});
	} catch (error) {
		results.push({
			name: entry.name,
			passed: false,
			error: error instanceof Error ? `${error.message}${error.stack ? `\n${error.stack}` : ""}` : String(error)
		});
	}
	const report = {
		passed: results.every((result) => result.passed),
		tests: results,
		trace
	};
	capability().finish(JSON.stringify(report));
}
queueMicrotask(() => {
	run();
});
//#endregion
export { expect, replay, test };

//# sourceMappingURL=index.mjs.map