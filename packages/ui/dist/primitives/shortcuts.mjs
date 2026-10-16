//#region src/primitives/shortcuts.ts
const MODIFIER_BITS = {
	shift: 1,
	control: 2,
	alt: 4,
	meta: 8
};
const ALL_MODIFIERS = 15;
/**
* Compile declarative application shortcuts into one keydown binding.
*
* Chords use names such as `Primary+T`, `Control+Tab`, and
* `Control+Shift+Tab`. `Primary` resolves to Command on macOS and Control on
* other platforms while still requiring an exact modifier match.
*/
function createShortcuts(shortcuts) {
	const compiled = Object.entries(shortcuts).map(([chord, value]) => compileShortcut(chord, value));
	assertNoAmbiguousShortcuts(compiled);
	const invokeKeyDown = (event) => {
		const key = normalizeKey(event.key);
		const modifiers = event.mods & ALL_MODIFIERS;
		const shortcut = compiled.find((candidate) => candidate.key === key && (candidate.primary ? event.primary && candidate.modifierMasks.includes(modifiers & ~(MODIFIER_BITS.control | MODIFIER_BITS.meta)) : candidate.modifierMasks.includes(modifiers)));
		if (shortcut === void 0 || event.repeat && !shortcut.allowRepeat) return { handled: false };
		if (shortcut.preventDefault !== false) event.preventDefault();
		return {
			handled: true,
			result: shortcut.handler(event)
		};
	};
	return {
		handleKeyDown: (event) => invokeKeyDown(event).handled,
		bindings: { onKeyDown: (event) => invokeKeyDown(event).result }
	};
}
function compileShortcut(chord, value) {
	const parts = chord.split("+").map((part) => part.trim()).filter(Boolean);
	if (parts.length === 0) throw new Error("Shortcut chord cannot be empty");
	const key = normalizeKey(parts.pop());
	if (key.length === 0) throw new Error(`Shortcut chord has no key: ${chord}`);
	let mask = 0;
	let primary = false;
	for (const rawModifier of parts) {
		const modifier = rawModifier.toLowerCase();
		const bit = modifier === "shift" ? MODIFIER_BITS.shift : modifier === "control" || modifier === "ctrl" ? MODIFIER_BITS.control : modifier === "alt" || modifier === "option" ? MODIFIER_BITS.alt : modifier === "meta" || modifier === "cmd" || modifier === "command" ? MODIFIER_BITS.meta : void 0;
		if (modifier === "primary" || modifier === "mod") {
			if (primary) throw new Error(`Duplicate modifier in shortcut: ${chord}`);
			primary = true;
		} else if (bit === void 0) throw new Error(`Unknown shortcut modifier '${rawModifier}' in ${chord}`);
		else if ((mask & bit) !== 0) throw new Error(`Duplicate modifier in shortcut: ${chord}`);
		else mask |= bit;
	}
	if (primary && (mask & (MODIFIER_BITS.control | MODIFIER_BITS.meta)) !== 0) throw new Error(`Primary cannot be combined with Control or Meta: ${chord}`);
	return {
		...typeof value === "function" ? { handler: value } : value,
		chord,
		key,
		modifierMasks: [mask],
		primary
	};
}
function assertNoAmbiguousShortcuts(shortcuts) {
	const owners = /* @__PURE__ */ new Map();
	for (const shortcut of shortcuts) {
		const masks = shortcut.primary ? shortcut.modifierMasks.flatMap((mask) => [mask | MODIFIER_BITS.control, mask | MODIFIER_BITS.meta]) : shortcut.modifierMasks;
		for (const mask of masks) {
			const signature = `${mask}:${shortcut.key}`;
			const previous = owners.get(signature);
			if (previous !== void 0) throw new Error(`Ambiguous shortcuts '${previous}' and '${shortcut.chord}'`);
			owners.set(signature, shortcut.chord);
		}
	}
}
function normalizeKey(key) {
	return key.trim().toLowerCase();
}
//#endregion
export { createShortcuts };

//# sourceMappingURL=shortcuts.mjs.map