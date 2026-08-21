import { renderComponent } from "@wabou/test/component";
import { Separator } from "@wabou/ui";
import { expect, test } from "vitest";

test("keeps decorative separators out of the semantic tree by default", () => {
  const screen = renderComponent(() => <Separator />);
  const separator = screen.roots[0];

  expect(separator.role).toBe("presentation");
  expect(separator.attribute("aria-hidden")).toBe("true");
  expect(separator.orientation).toBe(null);
  expect(separator.className).toContain("h-px");
  expect(separator.className).toContain("w-full");
});

test("exposes meaningful separators with their authored orientation", () => {
  const screen = renderComponent(() => (
    <Separator
      decorative={false}
      orientation="vertical"
      aria-label="Resize boundary"
    />
  ));
  const separator = screen.getByRole("separator", {
    name: "Resize boundary",
  });

  expect(separator.orientation).toBe("vertical");
  expect(separator.attribute("aria-hidden")).toBe(null);
  expect(separator.className).toContain("w-px");
  expect(separator.className).toContain("h-full");
});
