import { renderComponent } from "@wabou/test/component";
import { CellsTask } from "../../apps/7guis/ui/tasks/cells";
import { CrudTask } from "../../apps/7guis/ui/tasks/crud";
import { TemperatureTask } from "../../apps/7guis/ui/tasks/temperature";
import { expect, test } from "vitest";

test("temperature reports invalid numeric input without corrupting its peer", () => {
  const screen = renderComponent(TemperatureTask);
  const celsius = screen.getByRole("textbox", { name: "Celsius" });
  const fahrenheit = screen.getByRole("textbox", { name: "Fahrenheit" });

  celsius.input("warm");

  expect(celsius.attribute("aria-invalid")).toBe("true");
  expect(screen.getByRole("alert").text).toContain("Enter a valid number");
  expect(fahrenheit.value).toBe("32");
});

test("CRUD validates both required names before creating a record", () => {
  const screen = renderComponent(CrudTask);

  screen.getByRole("button", { name: "Create person" }).click();

  const errors = screen.getAllByRole("alert");
  expect(errors.map((error) => error.text)).toEqual([
    "First name is required.",
    "Surname is required.",
  ]);
  expect(
    screen
      .getByRole("textbox", { name: "First name" })
      .attribute("aria-invalid"),
  ).toBe("true");
});

test("Cells uses semantic state colors and exposes formula errors", () => {
  const screen = renderComponent(CellsTask);
  const selected = screen.getByRole("gridcell", { name: "Cell A1" });

  expect(selected.className).toContain("bg-selected");
  expect(selected.className).toContain("border-accent");

  const formula = screen.getByRole("textbox", { name: "Cell formula" });
  formula.input("=SUM(");
  expect(formula.attribute("aria-invalid")).toBe("true");
  expect(screen.getByRole("alert").text).toContain(
    "valid, non-circular formula",
  );
});
