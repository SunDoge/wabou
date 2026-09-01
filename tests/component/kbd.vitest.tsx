import { renderComponent } from "@wabou/test/component";
import { Kbd, KbdGroup } from "@wabou/ui";
import { expect, test } from "vitest";

test("Kbd owns a compact GPUI-aligned keycap rhythm", () => {
  const screen = renderComponent(() => (
    <KbdGroup aria-label="Shortcut">
      <Kbd aria-label="Control key">Ctrl</Kbd>
      <Kbd aria-label="K key" style={{ "line-height": 1.25 }}>
        K
      </Kbd>
    </KbdGroup>
  ));

  const control = screen.getByRole("label", { name: "Control key" });
  const overridden = screen.getByRole("label", { name: "K key" });

  expect(control.className).toContain(
    "h-5 min-w-5 px-1 py-0.5 flex-none text-center rounded bg-control text-xs font-medium text-muted",
  );
  expect(control.style("line-height")).toBe("1");
  expect(overridden.style("line-height")).toBe("1.25");
});
