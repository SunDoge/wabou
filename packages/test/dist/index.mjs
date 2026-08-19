import { defaultHost } from "@wabou/core/renderer";
import { isResourceKeyParts } from "@wabou/core/protocol";
//#region src/locator-query.ts
var LocatorAmbiguousError = class extends Error {};
/** Decode and validate the request-scoped envelope without choosing a match. */
function decodeNativeLocatorQuery(raw) {
	if (raw == null) return null;
	const query = JSON.parse(raw);
	if (!Number.isSafeInteger(query.matchCount) || query.matchCount < 1) throw new Error(`native locator query returned an invalid match count`);
	return query;
}
/** Whether a locator occurrence is absent from this completed query frame. */
function locatorQueryIsAbsent(raw, index) {
	const query = decodeNativeLocatorQuery(raw);
	return query === null || index !== void 0 && index >= query.matchCount;
}
/** Count matches without applying strict or indexed locator selection. */
function locatorQueryMatchCount(raw) {
	return decodeNativeLocatorQuery(raw)?.matchCount ?? 0;
}
/** Decode one request-scoped native query and enforce strict locator identity. */
function decodeLocatorQuery(raw, description, index) {
	const query = decodeNativeLocatorQuery(raw);
	if (query === null) return null;
	if (index === void 0 && query.matchCount !== 1) throw new LocatorAmbiguousError(`found ${query.matchCount} matches for ${description}; expected exactly one`);
	if (index !== void 0 && index >= query.matchCount) return null;
	if (query.snapshot === null) throw new Error(`native locator query omitted the selected snapshot`);
	return query.snapshot;
}
//#endregion
//#region src/poll.ts
function duration(value, fallback, name) {
	const resolved = value ?? fallback;
	if (!Number.isFinite(resolved) || resolved < 0) throw new RangeError(`${name} must be a finite non-negative number`);
	return resolved;
}
/** Resolve defaults before an operation is recorded so replay is deterministic. */
function resolvePollOptions(options = {}) {
	return {
		timeout: duration(options.timeout, 1e3, "timeout"),
		interval: duration(options.interval, 16, "interval")
	};
}
/** Poll an observable value with one explicit clock and retry policy. */
async function pollUntil(read, matches, options = {}, beforeRead) {
	const { timeout, interval } = resolvePollOptions(options);
	const deadline = performance.now() + timeout;
	let value;
	for (;;) {
		await beforeRead?.();
		value = await read();
		if (matches(value)) return {
			matched: true,
			value
		};
		const remaining = deadline - performance.now();
		if (remaining <= 0) return {
			matched: false,
			value
		};
		await new Promise((resolve) => setTimeout(resolve, Math.min(interval, remaining)));
	}
}
//#endregion
//#region src/replay.ts
/** Execute a recorded trace against explicit page and window capabilities. */
async function replayActions(actions, page, window, assertLocator, assertWindow) {
	for (const action of actions) if (action.action === "nativeClose") await window.nativeClose(action.windowId, action.platform);
	else if (action.action === "showWindow") await window.show(action.windowId);
	else if (action.action === "clickByRole") await page.forWindow(action.windowId).getByRole(action.role, {
		name: action.label,
		index: action.index
	}).click(action.wait);
	else if (action.action === "waitForByRole") await page.forWindow(action.windowId).getByRole(action.role, {
		name: action.label,
		index: action.index
	}).waitFor(action.wait);
	else if (action.action === "assertByRole") await assertLocator(page.forWindow(action.windowId).getByRole(action.role, {
		name: action.label,
		index: action.index
	}), action);
	else if (action.action === "assertWindowState") await assertWindow(window, action);
	else {
		const locator = page.forWindow(action.windowId).getByRole(action.role, {
			name: action.label,
			index: action.index
		});
		const input = action.input;
		if (input.type === "probe") await locator.waitFor(action.wait);
		else if (input.type === "drag") await locator.dragBy(input.deltaX, input.deltaY, action.wait);
		else if (input.type === "key") await locator.press(input.key, {
			shift: (input.modifiers & 1) !== 0,
			control: (input.modifiers & 2) !== 0,
			alt: (input.modifiers & 4) !== 0,
			meta: (input.modifiers & 8) !== 0
		}, action.wait);
		else if (input.type === "text") await locator.type(input.text, action.wait);
		else if (input.type === "paste") await locator.paste(input.text, action.wait);
		else if (input.type === "ime") await locator.ime(input.text, action.wait);
		else await locator.wheel(input.deltaY, input.deltaX, action.wait);
	}
}
//#endregion
//#region src/timeout.ts
const MAX_TEST_TIMEOUT = 6e4;
const DEFAULT_TEST_TIMEOUT = 5e3;
const SUITE_TIMEOUT = 6e4;
function testTimeout(value) {
	const timeout = value ?? 5e3;
	if (!Number.isFinite(timeout) || timeout <= 0 || timeout > 6e4) throw new RangeError(`test timeout must be a finite number between 1 and ${MAX_TEST_TIMEOUT}ms`);
	return timeout;
}
/** Bound a combined replay by its authored waits without removing the hard cap. */
function replayTimeout(actions) {
	let timeout = DEFAULT_TEST_TIMEOUT;
	for (const action of actions) {
		const wait = "wait" in action ? action.wait : void 0;
		timeout += typeof wait === "object" && wait !== null && "timeout" in wait && typeof wait.timeout === "number" ? wait.timeout : 100;
		if (timeout >= 6e4) return MAX_TEST_TIMEOUT;
	}
	return timeout;
}
var TestTimeoutError = class extends Error {};
var SuiteTimeoutError = class extends Error {};
/** Bound one test body without allowing a timeout to continue the suite. */
async function withTestTimeout(name, timeout, operation) {
	let timer;
	const expired = new Promise((_, reject) => {
		timer = setTimeout(() => reject(new TestTimeoutError(`test ${JSON.stringify(name)} timed out after ${timeout}ms`)), timeout);
	});
	try {
		return await Promise.race([Promise.resolve().then(operation), expired]);
	} finally {
		if (timer !== void 0) clearTimeout(timer);
	}
}
/** Bound the complete scenario so native watchdogs remain a final fallback. */
async function withSuiteTimeout(timeout, operation, activeTest = () => void 0) {
	let timer;
	const expired = new Promise((_, reject) => {
		timer = setTimeout(() => {
			const active = activeTest();
			reject(new SuiteTimeoutError(`test suite timed out after ${timeout}ms${active === void 0 ? "" : ` while running ${JSON.stringify(active)}`}`));
		}, timeout);
	});
	try {
		return await Promise.race([Promise.resolve().then(operation), expired]);
	} finally {
		if (timer !== void 0) clearTimeout(timer);
	}
}
//#endregion
//#region src/validation.ts
const MAX_SAFE_JAVASCRIPT_INTEGER = 9007199254740991;
/** Keep authored pointer input JSON-safe and reproducible across native hosts. */
function validateInputDeltas(kind, deltaX, deltaY) {
	if (!Number.isFinite(deltaX) || !Number.isFinite(deltaY)) throw new RangeError(`${kind} deltas must be finite numbers`);
}
/** A physical key pair without a key identity cannot be routed predictably. */
function validateKey(key) {
	if (typeof key !== "string" || key.length === 0) throw new RangeError("key must be a non-empty string");
}
function validateWindowKey(windowKey) {
	if (!isResourceKeyParts(windowKey)) throw new RangeError("window key must contain a non-zero slot and odd generation");
}
function validateSurfaceGeneration(value) {
	if (!Number.isSafeInteger(value) || value < 0 || value > MAX_SAFE_JAVASCRIPT_INTEGER) throw new RangeError(`surface generation must be an integer between 0 and ${MAX_SAFE_JAVASCRIPT_INTEGER}`);
}
function validateWindowPresence(value) {
	if (value !== "visible" && value !== "hidden" && value !== "surface-released" && value !== "closed") throw new RangeError(`unknown window presence ${JSON.stringify(value)}`);
}
function validateTolerance(kind, tolerance) {
	if (!Number.isFinite(tolerance) || tolerance < 0) throw new RangeError(`${kind} tolerance must be a finite non-negative number`);
}
function validateLocatorCount(value) {
	if (!Number.isSafeInteger(value) || value < 0) throw new RangeError("locator count must be a non-negative safe integer");
}
//#endregion
//#region src/index.ts
const TEST_ARTIFACT_VERSION = 1;
const tests = [];
const testNames = /* @__PURE__ */ new Set();
const registrationErrors = [];
const trace = [];
const MAX_LOCATOR_INDEX = 4294967295;
var LocatorNotFoundError = class extends Error {};
function capability() {
	const value = defaultHost.test;
	if (!value) throw new Error("@wabou/test requires `wabou test`");
	return value;
}
function windowLabel(windowId) {
	return `${windowId.lo}v${windowId.hi}`;
}
function createPage(windowId) {
	validateWindowKey(windowId);
	return {
		forWindow(nextWindowId) {
			return createPage(nextWindowId);
		},
		async waitForIdle() {
			await Promise.resolve();
			await new Promise((resolve) => requestAnimationFrame(() => resolve()));
			if (!await capability().waitForIdle(windowId.lo, windowId.hi)) throw new Error(`native window ${windowLabel(windowId)} did not become idle`);
		},
		getByRole(role, options) {
			const index = options.index;
			if (index !== void 0 && (!Number.isSafeInteger(index) || index < 0 || index > MAX_LOCATOR_INDEX)) throw new RangeError(`locator index must be an integer between 0 and ${MAX_LOCATOR_INDEX}`);
			const locatorLabel = `${role} named ${JSON.stringify(options.name)}${index === void 0 ? "" : ` at index ${index}`}`;
			const description = `${locatorLabel} in window ${windowLabel(windowId)}`;
			const sendInput = async (value) => {
				if (!await capability().inputByRole(windowId.lo, windowId.hi, role, options.name, JSON.stringify(value), index ?? null)) return false;
				return true;
			};
			const input = async (value) => {
				if (!await sendInput(value)) throw new Error(`no enabled ${locatorLabel}`);
			};
			const probe = async () => {
				return decodeLocatorQuery(await capability().queryByRole(windowId.lo, windowId.hi, role, options.name, index ?? null), description, index);
			};
			const snapshot = async () => {
				const value = await probe();
				if (!value) throw new LocatorNotFoundError(`no ${locatorLabel}`);
				return value;
			};
			const waitUntilActionable = async (assertionOptions = {}) => {
				const wait = resolvePollOptions(assertionOptions);
				let ambiguity;
				if ((await pollUntil(async () => {
					try {
						const value = await probe();
						ambiguity = void 0;
						return value;
					} catch (error) {
						if (!(error instanceof LocatorAmbiguousError)) throw error;
						ambiguity = error.message;
						return null;
					}
				}, (state) => state !== null && !state.disabled, wait, () => createPage(windowId).waitForIdle())).matched) return;
				throw new Error((ambiguity === void 0 ? void 0 : `${ambiguity} after ${wait.timeout}ms`) ?? `no enabled ${locatorLabel} after ${wait.timeout}ms`);
			};
			return {
				windowId,
				role,
				name: options.name,
				index,
				async click(assertionOptions) {
					const wait = resolvePollOptions(assertionOptions);
					trace.push({
						action: "clickByRole",
						windowId,
						role,
						label: options.name,
						index,
						wait
					});
					await waitUntilActionable(wait);
					if (!await capability().clickByRole(windowId.lo, windowId.hi, role, options.name, index ?? null)) throw new Error(`no enabled ${locatorLabel}`);
				},
				async dragBy(deltaX, deltaY, assertionOptions) {
					validateInputDeltas("drag", deltaX, deltaY);
					const wait = resolvePollOptions(assertionOptions);
					trace.push({
						action: "inputByRole",
						windowId,
						role,
						label: options.name,
						index,
						input: {
							type: "drag",
							deltaX,
							deltaY
						},
						wait
					});
					await waitUntilActionable(wait);
					await input({
						type: "drag",
						deltaX,
						deltaY
					});
				},
				async press(key, modifiers = {}, assertionOptions) {
					validateKey(key);
					const bits = (modifiers.shift ? 1 : 0) | (modifiers.control ? 2 : 0) | (modifiers.alt ? 4 : 0) | (modifiers.meta ? 8 : 0);
					const wait = resolvePollOptions(assertionOptions);
					trace.push({
						action: "inputByRole",
						windowId,
						role,
						label: options.name,
						index,
						input: {
							type: "key",
							key,
							modifiers: bits
						},
						wait
					});
					await waitUntilActionable(wait);
					await input({
						type: "key",
						key,
						modifiers: bits
					});
				},
				async type(text, assertionOptions) {
					const wait = resolvePollOptions(assertionOptions);
					trace.push({
						action: "inputByRole",
						windowId,
						role,
						label: options.name,
						index,
						input: {
							type: "text",
							text
						},
						wait
					});
					await waitUntilActionable(wait);
					await input({
						type: "text",
						text
					});
				},
				async paste(text, assertionOptions) {
					const wait = resolvePollOptions(assertionOptions);
					trace.push({
						action: "inputByRole",
						windowId,
						role,
						label: options.name,
						index,
						input: {
							type: "paste",
							text
						},
						wait
					});
					await waitUntilActionable(wait);
					await input({
						type: "paste",
						text
					});
				},
				async ime(text, assertionOptions) {
					const wait = resolvePollOptions(assertionOptions);
					trace.push({
						action: "inputByRole",
						windowId,
						role,
						label: options.name,
						index,
						input: {
							type: "ime",
							text
						},
						wait
					});
					await waitUntilActionable(wait);
					await input({
						type: "ime",
						text
					});
				},
				async wheel(deltaY, deltaX = 0, assertionOptions) {
					validateInputDeltas("wheel", deltaX, deltaY);
					const wait = resolvePollOptions(assertionOptions);
					trace.push({
						action: "inputByRole",
						windowId,
						role,
						label: options.name,
						index,
						input: {
							type: "wheel",
							deltaX,
							deltaY
						},
						wait
					});
					await waitUntilActionable(wait);
					await input({
						type: "wheel",
						deltaX,
						deltaY
					});
				},
				async waitFor(assertionOptions = {}) {
					const wait = resolvePollOptions(assertionOptions);
					trace.push({
						action: "waitForByRole",
						windowId,
						role,
						label: options.name,
						index,
						wait
					});
					let ambiguity;
					if ((await pollUntil(async () => {
						try {
							const value = await probe();
							ambiguity = void 0;
							return value;
						} catch (error) {
							if (!(error instanceof LocatorAmbiguousError)) throw error;
							ambiguity = error.message;
							return null;
						}
					}, (state) => state !== null, wait, () => createPage(windowId).waitForIdle())).matched) return;
					throw new Error((ambiguity === void 0 ? void 0 : `${ambiguity} after ${wait.timeout}ms`) ?? `no ${locatorLabel} after ${wait.timeout}ms`);
				},
				snapshot
			};
		}
	};
}
const context = {
	page: createPage({
		lo: __wabou_window_id_lo,
		hi: __wabou_window_id_hi
	}),
	window: {
		current: {
			lo: __wabou_window_id_lo,
			hi: __wabou_window_id_hi
		},
		async nativeClose(windowId, platform) {
			validateWindowKey(windowId);
			trace.push({
				action: "nativeClose",
				windowId,
				platform
			});
			if (!await capability().nativeClose(windowId.lo, windowId.hi, platform !== "wayland")) throw new Error(`failed to enqueue native close for window ${windowLabel(windowId)}`);
		},
		async show(windowId) {
			validateWindowKey(windowId);
			trace.push({
				action: "showWindow",
				windowId
			});
			if (!await capability().showWindow(windowId.lo, windowId.hi)) throw new Error(`failed to enqueue show for window ${windowLabel(windowId)}`);
		},
		state(windowId) {
			validateWindowKey(windowId);
			return JSON.parse(capability().windowState(windowId.lo, windowId.hi));
		}
	}
};
function test(name, body, options = {}) {
	if (name.trim() === "") {
		registrationErrors.push("test name cannot be empty");
		return;
	}
	if (testNames.has(name)) {
		registrationErrors.push(`duplicate test name ${JSON.stringify(name)}`);
		return;
	}
	let timeout;
	try {
		timeout = testTimeout(options.timeout);
	} catch (error) {
		registrationErrors.push(`invalid options for test ${JSON.stringify(name)}: ${error instanceof Error ? error.message : String(error)}`);
		return;
	}
	testNames.add(name);
	tests.push({
		name,
		body,
		timeout
	});
}
function locatorAssertionDiagnostic(assertion, state) {
	if (assertion.type === "absent" || assertion.type === "count") throw new Error(`${assertion.type} assertions do not accept a locator snapshot`);
	if (assertion.type === "text") {
		const value = state.value ?? state.name;
		return value === assertion.expected ? null : `expected locator text ${JSON.stringify(value)} to be ${JSON.stringify(assertion.expected)}`;
	}
	if (assertion.type === "value") return state.value === assertion.expected ? null : `expected locator value ${JSON.stringify(state.value)} to be ${JSON.stringify(assertion.expected)}`;
	if (assertion.type === "numericRange") {
		const actual = {
			value: state.numericValue,
			min: state.minNumericValue,
			max: state.maxNumericValue
		};
		for (const key of [
			"value",
			"min",
			"max"
		]) {
			const expected = assertion.expected[key];
			if (expected !== void 0 && (actual[key] === null || Math.abs(actual[key] - expected) > assertion.tolerance)) return `expected locator numeric range.${key} ${JSON.stringify(actual[key])} to be within ${assertion.tolerance} of ${expected}`;
		}
		return null;
	}
	if (assertion.type === "disabled") return state.disabled === assertion.expected ? null : `expected locator to be ${assertion.expected ? "disabled" : "enabled"}`;
	if (assertion.type === "checked") return state.checked === assertion.expected ? null : `expected locator to be ${assertion.expected === "mixed" ? "indeterminate" : assertion.expected ? "checked" : "unchecked"}, received ${JSON.stringify(state.checked)}`;
	if (assertion.type === "selected") return state.selected === assertion.expected ? null : `expected locator to be ${assertion.expected ? "selected" : "deselected"}, received ${JSON.stringify(state.selected)}`;
	if (assertion.type === "expanded") return state.expanded === assertion.expected ? null : `expected locator to be ${assertion.expected ? "expanded" : "collapsed"}, received ${JSON.stringify(state.expanded)}`;
	if (assertion.type === "pressed") return state.pressed === assertion.expected ? null : `expected locator to be ${assertion.expected ? "pressed" : "unpressed"}, received ${JSON.stringify(state.pressed)}`;
	if (assertion.type === "bounds") {
		for (const key of [
			"x",
			"y",
			"width",
			"height"
		]) {
			const expected = assertion.expected[key];
			if (expected !== void 0 && Math.abs(state.bounds[key] - expected) > assertion.tolerance) return `expected locator bounds.${key} ${state.bounds[key]} to be within ${assertion.tolerance}px of ${expected}`;
		}
		return null;
	}
	return state.focused === assertion.expected ? null : `expected locator to be ${assertion.expected ? "focused" : "blurred"}`;
}
async function locatorAbsenceDiagnostic(target) {
	const raw = await capability().queryByRole(target.windowId.lo, target.windowId.hi, target.role, target.name, target.index ?? null);
	if (locatorQueryIsAbsent(raw, target.index)) return null;
	const query = decodeNativeLocatorQuery(raw);
	if (query === null) throw new Error("unreachable absent locator query");
	const occurrence = target.index === void 0 ? "" : ` at index ${target.index}`;
	return `expected ${target.role} named ${JSON.stringify(target.name)}${occurrence} to be absent, found ${query.matchCount} matching semantic ${query.matchCount === 1 ? "node" : "nodes"}`;
}
async function locatorCountDiagnostic(target, expected) {
	if (target.index !== void 0) throw new Error("toHaveCount requires an unindexed locator");
	const actual = locatorQueryMatchCount(await capability().queryByRole(target.windowId.lo, target.windowId.hi, target.role, target.name, null));
	return actual === expected ? null : `expected ${target.role} named ${JSON.stringify(target.name)} to have ${expected} ${expected === 1 ? "match" : "matches"}, found ${actual}`;
}
async function assertLocatorEventually(target, assertion, options = {}) {
	const wait = resolvePollOptions(options);
	trace.push({
		action: "assertByRole",
		windowId: target.windowId,
		role: target.role,
		label: target.name,
		index: target.index,
		assertion,
		wait
	});
	const result = await pollUntil(async () => {
		if (assertion.type === "absent") return locatorAbsenceDiagnostic(target);
		if (assertion.type === "count") return locatorCountDiagnostic(target, assertion.expected);
		try {
			return locatorAssertionDiagnostic(assertion, await target.snapshot());
		} catch (error) {
			if (!(error instanceof LocatorNotFoundError) && !(error instanceof LocatorAmbiguousError)) throw error;
			return error.message;
		}
	}, (diagnostic) => diagnostic === null, wait, () => createPage(target.windowId).waitForIdle());
	if (!result.matched) throw new Error(`${result.value} after ${wait.timeout}ms`);
}
/** Register a previously recorded action trace as a behavior test. */
function replay(actions) {
	test("replay action trace", async ({ window }) => {
		await replayActions(actions, context.page, window, replayLocatorAssertion, replayWindowAssertion);
	}, { timeout: replayTimeout(actions) });
}
async function replayLocatorAssertion(locator, action) {
	await assertLocatorEventually(locator, action.assertion, action.wait);
}
async function replayWindowAssertion(window, action) {
	await assertWindowStateEventually(window, action.windowId, action.expected, action.wait);
}
async function assertWindowStateEventually(target, windowId, expected, options = {}) {
	validateWindowKey(windowId);
	validateWindowPresence(expected.presence);
	validateSurfaceGeneration(expected.surfaceGeneration);
	const wait = resolvePollOptions(options);
	const recorded = {
		presence: expected.presence,
		surfaceGeneration: expected.surfaceGeneration
	};
	trace.push({
		action: "assertWindowState",
		windowId,
		expected: recorded,
		wait
	});
	const result = await pollUntil(() => target.state(windowId), (actual) => actual?.presence === recorded.presence && actual.surfaceGeneration === recorded.surfaceGeneration, wait);
	if (!result.matched) throw new Error(`expected window ${windowId} state ${JSON.stringify(result.value)} to be ${JSON.stringify(recorded)} after ${wait.timeout}ms`);
}
function expect(actual) {
	const locator = () => {
		if (!actual || typeof actual !== "object" || !("snapshot" in actual)) throw new Error("this assertion requires a Wabou locator");
		return actual;
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
		toHaveState(windowId, expected, options) {
			if (actual !== context.window) throw new Error("toHaveState requires the Wabou test window capability");
			return assertWindowStateEventually(context.window, windowId, expected, options);
		},
		toHaveText(expected, options) {
			return assertLocatorEventually(locator(), {
				type: "text",
				expected
			}, options);
		},
		toBeAbsent(options) {
			return assertLocatorEventually(locator(), { type: "absent" }, options);
		},
		toHaveCount(expected, options) {
			validateLocatorCount(expected);
			const target = locator();
			if (target.index !== void 0) throw new Error("toHaveCount requires an unindexed locator");
			return assertLocatorEventually(target, {
				type: "count",
				expected
			}, options);
		},
		toHaveValue(expected, options) {
			return assertLocatorEventually(locator(), {
				type: "value",
				expected
			}, options);
		},
		toHaveRange(expected, options = {}) {
			const entries = Object.entries(expected);
			if (entries.length === 0) throw new RangeError("expected locator numeric range cannot be empty");
			const supported = /* @__PURE__ */ new Set([
				"value",
				"min",
				"max"
			]);
			if (entries.some(([key]) => !supported.has(key))) throw new RangeError("expected locator numeric range may only contain value, min, and max");
			if (entries.some(([, value]) => !Number.isFinite(value))) throw new RangeError("expected locator numeric range must contain finite numbers");
			const tolerance = options.tolerance ?? 1e-9;
			validateTolerance("numeric range assertion", tolerance);
			return assertLocatorEventually(locator(), {
				type: "numericRange",
				expected: { ...expected },
				tolerance
			}, options);
		},
		toBeDisabled(options) {
			return assertLocatorEventually(locator(), {
				type: "disabled",
				expected: true
			}, options);
		},
		toBeEnabled(options) {
			return assertLocatorEventually(locator(), {
				type: "disabled",
				expected: false
			}, options);
		},
		toBeChecked(options) {
			return assertLocatorEventually(locator(), {
				type: "checked",
				expected: true
			}, options);
		},
		toBeUnchecked(options) {
			return assertLocatorEventually(locator(), {
				type: "checked",
				expected: false
			}, options);
		},
		toBeIndeterminate(options) {
			return assertLocatorEventually(locator(), {
				type: "checked",
				expected: "mixed"
			}, options);
		},
		toBeSelected(options) {
			return assertLocatorEventually(locator(), {
				type: "selected",
				expected: true
			}, options);
		},
		toBeDeselected(options) {
			return assertLocatorEventually(locator(), {
				type: "selected",
				expected: false
			}, options);
		},
		toBeExpanded(options) {
			return assertLocatorEventually(locator(), {
				type: "expanded",
				expected: true
			}, options);
		},
		toBeCollapsed(options) {
			return assertLocatorEventually(locator(), {
				type: "expanded",
				expected: false
			}, options);
		},
		toBePressed(options) {
			return assertLocatorEventually(locator(), {
				type: "pressed",
				expected: true
			}, options);
		},
		toBeUnpressed(options) {
			return assertLocatorEventually(locator(), {
				type: "pressed",
				expected: false
			}, options);
		},
		toBeFocused(options) {
			return assertLocatorEventually(locator(), {
				type: "focused",
				expected: true
			}, options);
		},
		toBeBlurred(options) {
			return assertLocatorEventually(locator(), {
				type: "focused",
				expected: false
			}, options);
		},
		toHaveBounds(expected, options = {}) {
			const entries = Object.entries(expected);
			if (entries.length === 0) throw new RangeError("expected locator bounds cannot be empty");
			const supported = /* @__PURE__ */ new Set([
				"x",
				"y",
				"width",
				"height"
			]);
			if (entries.some(([key]) => !supported.has(key))) throw new RangeError("expected locator bounds may only contain x, y, width, and height");
			if (entries.some(([, value]) => !Number.isFinite(value))) throw new RangeError("expected locator bounds must be finite numbers");
			const tolerance = options.tolerance ?? .5;
			validateTolerance("locator bounds", tolerance);
			return assertLocatorEventually(locator(), {
				type: "bounds",
				expected: { ...expected },
				tolerance
			}, options);
		}
	};
}
expect.poll = function poll(read, options = {}) {
	return { async toBe(expected) {
		const result = await pollUntil(read, (actual) => Object.is(actual, expected), {
			timeout: options.timeout,
			interval: options.interval ?? 10
		});
		if (!result.matched) throw new Error(`expected ${JSON.stringify(result.value)} to become ${JSON.stringify(expected)}`);
	} };
};
async function run() {
	const results = [];
	let activeTest;
	if (registrationErrors.length > 0) results.push({
		name: "test suite",
		passed: false,
		error: registrationErrors.join("\n"),
		traceStart: 0,
		traceEnd: 0,
		durationMs: 0
	});
	else if (tests.length === 0) results.push({
		name: "test suite",
		passed: false,
		error: "no tests registered",
		traceStart: 0,
		traceEnd: 0,
		durationMs: 0
	});
	if (registrationErrors.length === 0 && tests.length > 0) try {
		await withSuiteTimeout(SUITE_TIMEOUT, async () => {
			for (const entry of tests) {
				const traceStart = trace.length;
				const startedAt = performance.now();
				activeTest = {
					name: entry.name,
					traceStart,
					startedAt
				};
				try {
					await withTestTimeout(entry.name, entry.timeout, () => entry.body(context));
					results.push({
						name: entry.name,
						passed: true,
						traceStart,
						traceEnd: trace.length,
						durationMs: performance.now() - startedAt
					});
				} catch (error) {
					results.push({
						name: entry.name,
						passed: false,
						error: error instanceof Error ? `${error.message}${error.stack ? `\n${error.stack}` : ""}` : String(error),
						traceStart,
						traceEnd: trace.length,
						durationMs: performance.now() - startedAt
					});
					if (error instanceof TestTimeoutError) break;
				} finally {
					activeTest = void 0;
				}
			}
		}, () => activeTest?.name);
	} catch (error) {
		if (!(error instanceof SuiteTimeoutError)) throw error;
		results.push({
			name: activeTest?.name ?? "test suite",
			passed: false,
			error: `${error.message}${error.stack ? `\n${error.stack}` : ""}`,
			traceStart: activeTest?.traceStart ?? trace.length,
			traceEnd: trace.length,
			durationMs: activeTest === void 0 ? SUITE_TIMEOUT : performance.now() - activeTest.startedAt
		});
	}
	const report = {
		version: 1,
		passed: results.every((result) => result.passed),
		tests: results,
		trace
	};
	capability().finish(JSON.stringify(report));
}
queueMicrotask(() => {
	run().catch((error) => {
		const report = {
			version: 1,
			passed: false,
			tests: [{
				name: "test runner",
				passed: false,
				error: error instanceof Error ? `${error.message}${error.stack ? `\n${error.stack}` : ""}` : String(error),
				traceStart: 0,
				traceEnd: 0,
				durationMs: 0
			}],
			trace: []
		};
		capability().finish(JSON.stringify(report));
	});
});
//#endregion
export { TEST_ARTIFACT_VERSION, expect, replay, test };

//# sourceMappingURL=index.mjs.map