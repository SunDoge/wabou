import { defaultHost } from "@wabou/solid-renderer";
//#region src/index.ts
const tests = [];
const trace = [];
function capability() {
	const value = defaultHost.test;
	if (!value) throw new Error("@wabou/test requires `wabou test`");
	return value;
}
const context = {
	page: { getByRole(role, options) {
		const input = async (value) => {
			trace.push({
				action: "inputByRole",
				windowId: 1,
				role,
				label: options.name,
				input: value
			});
			if (!await capability().inputByRole(1, role, options.name, JSON.stringify(value))) throw new Error(`no enabled ${role} named ${JSON.stringify(options.name)}`);
		};
		return {
			async click() {
				trace.push({
					action: "clickByRole",
					windowId: 1,
					role,
					label: options.name
				});
				if (!await capability().clickByRole(1, role, options.name)) throw new Error(`no enabled ${role} named ${JSON.stringify(options.name)}`);
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
			}
		};
	} },
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
		for (const action of actions) if (action.action === "nativeClose") await window.nativeClose(action.windowId, action.platform);
		else if (action.action === "showWindow") await window.show(action.windowId);
		else if (action.action === "clickByRole") await context.page.getByRole(action.role, { name: action.label }).click();
		else {
			const locator = context.page.getByRole(action.role, { name: action.label });
			const input = action.input;
			if (input.type === "probe") await locator.waitFor();
			else if (input.type === "drag") await locator.dragBy(input.deltaX, input.deltaY);
			else if (input.type === "key") await capability().inputByRole(action.windowId, action.role, action.label, JSON.stringify(input));
			else if (input.type === "text") await locator.type(input.text);
			else if (input.type === "paste") await locator.paste(input.text);
			else if (input.type === "ime") await locator.ime(input.text);
			else await locator.wheel(input.deltaY, input.deltaX);
		}
	});
}
function expect(actual) {
	return {
		toBe(expected) {
			if (!Object.is(actual, expected)) throw new Error(`expected ${JSON.stringify(actual)} to be ${JSON.stringify(expected)}`);
		},
		toEqual(expected) {
			const left = JSON.stringify(actual);
			const right = JSON.stringify(expected);
			if (left !== right) throw new Error(`expected ${left} to equal ${right}`);
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