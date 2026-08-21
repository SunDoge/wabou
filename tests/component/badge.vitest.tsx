import { renderComponent } from "@wabou/test/component";
import { Badge, badgeClass } from "@wabou/ui";
import { expect, test } from "vitest";

test("forwards semantic text props and exposes shadcn badge styling", () => {
  const screen = renderComponent(() => (
    <Badge role="status" aria-label="Build status" variant="success">
      Ready
    </Badge>
  ));

  const badge = screen.getByRole("status", { name: "Build status" });
  expect(badge.text).toBe("Ready");
  expect(badge.className).toContain("rounded-full");
  expect(badge.className).toContain("bg-success-surface");
});

test("provides the complete visual variant set without competing weights", () => {
  expect(badgeClass("ghost")).toContain("border-transparent");
  expect(badgeClass("link")).toContain("text-accent");
  expect(badgeClass("destructive")).toContain("bg-danger-surface");
  expect(badgeClass("outline", "normal")).toContain("font-normal");
  expect(badgeClass("outline", "normal")).not.toContain("font-medium");
});
