import { renderComponent } from "@wabou/test/component";
import { Rating } from "@wabou/ui";
import { expect, test } from "vitest";

test("rating selects values and follows radio-group keyboard conventions", () => {
  const changes: number[] = [];
  const screen = renderComponent(() => (
    <Rating
      label="Product rating"
      defaultValue={2}
      onValueChange={(value) => changes.push(value)}
    />
  ));

  const fourth = screen.getByRole("radio", { name: "4 stars" });
  fourth.click();
  expect(changes).toEqual([4]);
  expect(fourth.checked).toBe(true);

  fourth.press("ArrowRight");
  expect(changes).toEqual([4, 5]);
  expect(screen.getByRole("radio", { name: "5 stars" }).checked).toBe(true);

  screen.getByRole("radio", { name: "5 stars" }).press("Home");
  expect(changes).toEqual([4, 5, 1]);
  expect(screen.getByRole("radio", { name: "1 star" }).checked).toBe(true);
});

test("rating can clear the current item without inventing a zero radio", () => {
  const screen = renderComponent(() => (
    <Rating label="Clearable rating" defaultValue={3} allowClear />
  ));

  screen.getByRole("radio", { name: "3 stars" }).click();
  for (const item of screen.getAllByRole("radio")) {
    expect(item.checked).toBe(false);
  }
});

test("controlled, disabled, and read-only ratings preserve ownership", () => {
  const controlledChanges: number[] = [];
  const controlled = renderComponent(() => (
    <Rating
      label="Controlled rating"
      value={2}
      onValueChange={(value) => controlledChanges.push(value)}
    />
  ));
  controlled.getByRole("radio", { name: "4 stars" }).click();
  expect(controlledChanges).toEqual([4]);
  expect(controlled.getByRole("radio", { name: "2 stars" }).checked).toBe(true);
  controlled.dispose();

  for (const mode of ["disabled", "readOnly"] as const) {
    const changes: number[] = [];
    const screen = renderComponent(() => (
      <Rating
        label={mode}
        defaultValue={2}
        disabled={mode === "disabled"}
        readOnly={mode === "readOnly"}
        onValueChange={(value) => changes.push(value)}
      />
    ));
    const fourth = screen.getByRole("radio", { name: "4 stars" });
    if (mode === "disabled") {
      expect(fourth.disabled).toBe(true);
    } else {
      fourth.click();
    }
    expect(changes).toEqual([]);
    screen.dispose();
  }
});
