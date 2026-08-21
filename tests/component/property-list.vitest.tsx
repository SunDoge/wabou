import { renderComponent } from "@wabou/test/component";
import { PropertyList, PropertyRow } from "@wabou/ui";
import { expect, test } from "vitest";

test("authors inspector rows with table semantics", () => {
  const screen = renderComponent(() => (
    <PropertyList aria-label="Runtime properties">
      <PropertyRow name="engine" value="QuickJS" />
      <PropertyRow name="frameRate" value="120 fps" />
    </PropertyList>
  ));
  expect(
    screen.getByRole("table", { name: "Runtime properties" }),
  ).not.toBeNull();
  expect(screen.getAllByRole("row")).toHaveLength(2);
  expect(screen.getByRole("row", { name: "frameRate" }).text).toContain(
    "120 fps",
  );
});
