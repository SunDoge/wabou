import { renderComponent } from "@wabou/test/component";
import { ActivityStatus, ActivityStatusIndicator } from "@wabou/ui";
import { expect, test } from "vitest";

test("activity status owns shrink-safe status geometry and semantics", () => {
  const screen = renderComponent(() => (
    <ActivityStatus
      label="Indexing a workspace with a deliberately long descriptive name"
      animated
    />
  ));

  const status = screen.getByRole("status", {
    name: "Indexing a workspace with a deliberately long descriptive name",
  });
  expect(status.className).toContain("min-w-0");
  expect(status.className).toContain("max-w-full");
  expect(status.children[0]?.className).toContain("flex-none");
  expect(status.children[1]?.className).toContain("truncate");
  expect(status.text).toBe(
    "Indexing a workspace with a deliberately long descriptive name",
  );
});

test("activity indicator only animates when requested", () => {
  const idle = renderComponent(() => (
    <ActivityStatusIndicator tone="success" />
  ));
  expect(idle.roots[0]?.className).toContain("bg-success-primary");
  expect(idle.roots[0]?.style("opacity")).toBeNull();
  idle.dispose();

  const live = renderComponent(() => <ActivityStatusIndicator animated />);
  expect(live.roots[0]?.style("opacity")).toBe("0.3");
});
