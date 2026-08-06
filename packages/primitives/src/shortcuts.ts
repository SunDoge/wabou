const MODIFIER_BITS = {
  shift: 1,
  control: 2,
  alt: 4,
  meta: 8,
} as const;

const ALL_MODIFIERS = 1 | 2 | 4 | 8;

export interface ShortcutEvent {
  key: string;
  mods: number;
  repeat?: boolean;
  preventDefault(): void;
}

export type ShortcutHandler = (event: ShortcutEvent) => void;

export interface ShortcutDefinition {
  handler: ShortcutHandler;
  /** Repeated keydown events are ignored by default. */
  allowRepeat?: boolean;
  /** Defaults to true so application shortcuts preempt focused widgets. */
  preventDefault?: boolean;
}

export type ShortcutMap = Record<string, ShortcutHandler | ShortcutDefinition>;

export interface ShortcutsResult {
  handleKeyDown: (event: ShortcutEvent) => boolean;
  bindings: {
    onKeyDown: (event: ShortcutEvent) => void;
  };
}

interface CompiledShortcut extends ShortcutDefinition {
  chord: string;
  key: string;
  modifierMasks: readonly number[];
}

/**
 * Compile declarative application shortcuts into one keydown binding.
 *
 * Chords use names such as `Primary+T`, `Control+Tab`, and
 * `Control+Shift+Tab`. `Primary` accepts either Control or Meta while still
 * requiring an exact modifier match, which keeps app code platform-agnostic.
 */
export function createShortcuts(shortcuts: ShortcutMap): ShortcutsResult {
  const compiled = Object.entries(shortcuts).map(([chord, value]) =>
    compileShortcut(chord, value),
  );
  assertNoAmbiguousShortcuts(compiled);

  const handleKeyDown = (event: ShortcutEvent) => {
    const key = normalizeKey(event.key);
    const modifiers = event.mods & ALL_MODIFIERS;
    const shortcut = compiled.find(
      (candidate) =>
        candidate.key === key && candidate.modifierMasks.includes(modifiers),
    );
    if (shortcut === undefined || (event.repeat && !shortcut.allowRepeat)) {
      return false;
    }
    if (shortcut.preventDefault !== false) event.preventDefault();
    shortcut.handler(event);
    return true;
  };

  return {
    handleKeyDown,
    bindings: { onKeyDown: handleKeyDown },
  };
}

function compileShortcut(
  chord: string,
  value: ShortcutHandler | ShortcutDefinition,
): CompiledShortcut {
  const parts = chord
    .split("+")
    .map((part) => part.trim())
    .filter(Boolean);
  if (parts.length === 0) throw new Error("Shortcut chord cannot be empty");
  const key = normalizeKey(parts.pop()!);
  if (key.length === 0) throw new Error(`Shortcut chord has no key: ${chord}`);

  let mask = 0;
  let primary = false;
  for (const rawModifier of parts) {
    const modifier = rawModifier.toLowerCase();
    const bit =
      modifier === "shift"
        ? MODIFIER_BITS.shift
        : modifier === "control" || modifier === "ctrl"
          ? MODIFIER_BITS.control
          : modifier === "alt" || modifier === "option"
            ? MODIFIER_BITS.alt
            : modifier === "meta" ||
                modifier === "cmd" ||
                modifier === "command"
              ? MODIFIER_BITS.meta
              : undefined;
    if (modifier === "primary" || modifier === "mod") {
      if (primary) throw new Error(`Duplicate modifier in shortcut: ${chord}`);
      primary = true;
    } else if (bit === undefined) {
      throw new Error(`Unknown shortcut modifier '${rawModifier}' in ${chord}`);
    } else if ((mask & bit) !== 0) {
      throw new Error(`Duplicate modifier in shortcut: ${chord}`);
    } else {
      mask |= bit;
    }
  }
  if (primary && (mask & (MODIFIER_BITS.control | MODIFIER_BITS.meta)) !== 0) {
    throw new Error(
      `Primary cannot be combined with Control or Meta: ${chord}`,
    );
  }

  const definition = typeof value === "function" ? { handler: value } : value;
  return {
    ...definition,
    chord,
    key,
    modifierMasks: primary
      ? [mask | MODIFIER_BITS.control, mask | MODIFIER_BITS.meta]
      : [mask],
  };
}

function assertNoAmbiguousShortcuts(
  shortcuts: readonly CompiledShortcut[],
): void {
  const owners = new Map<string, string>();
  for (const shortcut of shortcuts) {
    for (const mask of shortcut.modifierMasks) {
      const signature = `${mask}:${shortcut.key}`;
      const previous = owners.get(signature);
      if (previous !== undefined) {
        throw new Error(
          `Ambiguous shortcuts '${previous}' and '${shortcut.chord}'`,
        );
      }
      owners.set(signature, shortcut.chord);
    }
  }
}

function normalizeKey(key: string): string {
  return key.trim().toLowerCase();
}
