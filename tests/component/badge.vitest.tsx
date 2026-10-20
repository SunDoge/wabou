import { renderComponent } from "@wabou/test/component";
import { Badge, badgeClass, Fps } from "@wabou/ui";
import { expect, test } from "vitest";

test("forwards semantic text props and exposes shadcn badge styling", () => {
  const screen = renderComponent(() => (
    <Badge role="status" aria-label="Build status" variant="success">
      Ready
    </Badge>
  ));

  const badge = screen.getByRole("status", { name: "Build status" });
  expect(badge.text).toBe("Ready");
  expect(badge.className).toContain("bg-success-surface");
});

test("provides the complete visual variant set without competing weights", () => {
  expect(badgeClass("info")).toContain("bg-selected");
  expect(badgeClass("ghost")).toContain("border-transparent");
  expect(badgeClass("link")).toContain("text-accent");
  expect(badgeClass("destructive")).toContain("bg-danger-surface");
  expect(badgeClass("outline", "normal")).toContain("font-normal");
  expect(badgeClass("outline", "normal")).not.toContain("font-medium");
});

test("provides stable status-pill sizes without leaking private props", () => {
  const screen = renderComponent(() => (
    <>
      <Badge role="status" aria-label="Small badge" size="sm">
        2
      </Badge>
      <Badge role="status" aria-label="Default badge">
        Ready
      </Badge>
      <Badge role="status" aria-label="Large badge" size="lg">
        Published
      </Badge>
    </>
  ));

  const small = screen.getByRole("status", { name: "Small badge" });
  const standard = screen.getByRole("status", { name: "Default badge" });
  const large = screen.getByRole("status", { name: "Large badge" });
  expect(small.className).toContain("h-4");
  expect(standard.className).toContain("h-5");
  expect(large.className).toContain("h-6");
  expect(large.className).toContain("text-sm");
  expect(small.attribute("size")).toBeNull();
});

test("FPS badges retain one monospace weight", () => {
  const screen = renderComponent(() => <Fps value={60} />);
  const badge = screen.getByRole("label", { name: "60 fps" });
  expect(badge.className).toContain("font-mono");
  expect(badge.className).toContain("font-normal");
  expect(badge.className).not.toContain("font-medium");
});
