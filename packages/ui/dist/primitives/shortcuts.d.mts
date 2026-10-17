//#region src/primitives/shortcuts.d.ts
export interface ShortcutEvent {
  key: string;
  /** Physical Shift, Control, Alt, and Meta modifier bits. */
  mods: number;
  /** Whether the physical modifiers form this platform's Primary chord. */
  primary: boolean;
  repeat?: boolean;
  preventDefault(): void;
}
export type ShortcutHandler = (event: ShortcutEvent) => unknown;
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
    onKeyDown: (event: ShortcutEvent) => unknown;
  };
}
/**
 * Compile declarative application shortcuts into one keydown binding.
 *
 * Chords use names such as `Primary+T`, `Control+Tab`, and
 * `Control+Shift+Tab`. `Primary` resolves to Command on macOS and Control on
 * other platforms while still requiring an exact modifier match.
 */
export declare function createShortcuts(shortcuts: ShortcutMap): ShortcutsResult;
//#endregion
//# sourceMappingURL=shortcuts.d.mts.map