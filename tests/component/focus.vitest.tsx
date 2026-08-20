import { renderComponent } from "@wabou/test/component";
import type { Handle } from "@wabou/ui";
import { Button, View } from "@wabou/ui";
import { createEffect, createSignal } from "solid-js";
import { expect, test, vi } from "vitest";

test("component focus follows native focus and blur ordering", () => {
  const events: string[] = [];
  const screen = renderComponent(() => (
    <View>
      <Button
        aria-label="First"
        onFocus={() => events.push("first focus")}
        onBlur={() => events.push("first blur")}
      />
      <Button
        aria-label="Second"
        onFocus={() => events.push("second focus")}
        onBlur={() => events.push("second blur")}
      />
    </View>
  ));

  screen.getByRole("button", { name: "First" }).focus();
  screen.getByRole("button", { name: "Second" }).focus();
  screen.getByRole("button", { name: "Second" }).blur();

  expect(events).toEqual([
    "first focus",
    "first blur",
    "second focus",
    "second blur",
  ]);
});

test("effect-driven native focus does not recursively flush Solid", () => {
  const warnings: string[] = [];
  const warn = vi
    .spyOn(console, "warn")
    .mockImplementation((...args) => warnings.push(args.map(String).join(" ")));
  try {
    const FocusFromEffect = () => {
      const [active, setActive] = createSignal(false);
      let target: Handle | undefined;
      createEffect(active, (isActive) => {
        if (isActive) target?.focus();
      });
      return (
        <View>
          <Button aria-label="Activate focus" onClick={() => setActive(true)} />
          <Button aria-label="Focus target" ref={(node) => (target = node)} />
        </View>
      );
    };
    const screen = renderComponent(FocusFromEffect);

    screen.getByRole("button", { name: "Activate focus" }).click();

    expect(screen.getByRole("button", { name: "Focus target" }).focused).toBe(
      true,
    );
    expect(
      warnings.some((message) => message.includes("FLUSH_IN_EFFECT_CALLBACK")),
    ).toBe(false);
  } finally {
    warn.mockRestore();
  }
});
