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
    "h-5 min-w-5 px-1 inline-flex items-center justify-center flex-none rounded bg-control text-xs font-medium text-muted",
  );
  expect(control.tag).toBe("view");
  expect(control.children).toHaveLength(1);
  expect(control.children[0]?.tag).toBe("text");
  expect(control.children[0]?.attribute("aria-hidden")).toBe("true");
  expect(control.style("line-height")).toBe("1");
  expect(overridden.style("line-height")).toBe("1.25");
});
