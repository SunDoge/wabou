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
  const table = screen.getByRole("table", { name: "Runtime properties" });
  expect(table).not.toBeNull();
  expect(table.className).toContain("rounded-xl");
  expect(screen.getAllByRole("row")).toHaveLength(2);
  expect(screen.getByRole("row", { name: "frameRate" }).text).toContain(
    "120 fps",
  );
});
