import { renderComponent } from "@wabou/test/component";
import { LabeledSeparator, Separator, Text } from "@wabou/ui";
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

test("labeled separator keeps bounded content between two flexible rules", () => {
  const screen = renderComponent(() => (
    <LabeledSeparator role="group" aria-label="Turn boundary">
      <Text>Worked for 12s</Text>
    </LabeledSeparator>
  ));

  const separator = screen.getByRole("group", { name: "Turn boundary" });
  expect(separator.children).toHaveLength(3);
  expect(separator.children[0]?.className).toContain("flex-1");
  expect(separator.children[0]?.className).toContain("h-px");
  expect(separator.children[1]?.className).toContain("max-w-4/5");
  expect(separator.children[2]?.className).toContain("flex-1");
  expect(separator.children[2]?.className).toContain("h-px");
});
