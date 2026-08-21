import { renderComponent } from "@wabou/test/component";
import { ShortcutRecorder, shortcutFromKeyEvent } from "@wabou/ui";
import { expect, test } from "vitest";

test("normalizes platform-primary shortcut events", () => {
  expect(shortcutFromKeyEvent({ key: "k", mods: 3, primary: true })).toEqual({
    chord: "Primary+Shift+K",
    parts: ["Primary", "Shift", "K"],
  });
  expect(shortcutFromKeyEvent({ key: "Control", mods: 2 })).toBeUndefined();
});

test("records and clears a desktop keyboard shortcut", () => {
  const changes: string[] = [];
  const screen = renderComponent(() => (
    <ShortcutRecorder
      label="Open command palette"
      onValueChange={(value) => changes.push(value)}
    />
  ));
  const recorder = screen.getByRole("button", { name: "Open command palette" });
  recorder.click();
  recorder.press("k", { mods: 3, primary: true, code: "KeyK" });
  expect(changes).toEqual(["Primary+Shift+K"]);
  expect(recorder.text).toContain("PrimaryShiftK");
  recorder.click();
  recorder.press("Backspace");
  expect(changes).toEqual(["Primary+Shift+K", ""]);
});
