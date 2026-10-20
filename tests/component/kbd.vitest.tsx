import { renderComponent } from "@wabou/test/component";
import { Kbd, KbdGroup } from "@wabou/ui";
import { expect, test } from "vitest";

test("Kbd exposes its accessible label and forwards line-height overrides", () => {
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

  expect(control.tag).toBe("view");
  expect(control.children).toHaveLength(1);
  expect(control.children[0]?.tag).toBe("text");
  expect(control.children[0]?.attribute("aria-hidden")).toBe("true");
  expect(control.style("line-height")).toBe("1");
  expect(overridden.style("line-height")).toBe("1.25");
});
