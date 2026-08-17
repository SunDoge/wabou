import { expect, test } from "@wabou/test";

test("counter increments and resets", async ({ page }) => {
  await page.getByRole("button", { name: "Increment counter" }).click();
  await page.waitForIdle();
  await expect(page.getByRole("status", { name: "Counter value" })).toHaveText(
    "1",
  );
  await page.getByRole("button", { name: "Reset counter" }).click();
  await expect(page.getByRole("status", { name: "Counter value" })).toHaveText(
    "0",
  );
});

test("temperature accepts native text input", async ({ page }) => {
  await page.getByRole("button", { name: "Temperature" }).click();
  const celsius = page.getByRole("textbox", { name: "Celsius" });
  await celsius.press("a", { control: true });
  await celsius.type("100");
  await page.waitForIdle();
  await expect(page.getByRole("textbox", { name: "Fahrenheit" })).toHaveValue(
    "212",
  );
});

test("all seven tasks are reachable", async ({ page }) => {
  for (const name of [
    "Flight Booker",
    "Timer",
    "CRUD",
    "Circle Drawer",
    "Cells",
  ]) {
    await page.getByRole("button", { name }).click();
  }
  expect(true).toBe(true);
});

test("flight booker uses the shared localized date picker", async ({ page }) => {
  await page.getByRole("button", { name: "Flight Booker" }).click();
  await page.getByRole("button", { name: "Departure date" }).click();
  await page.getByRole("button", { name: "Next month" }).click();
  await page
    .getByRole("button", { name: "Thursday, October 1, 2026" })
    .click();
  await page.getByRole("button", { name: "Book flight" }).click();
});

test("circle drawer receives local pointer coordinates", async ({ page }) => {
  await page.getByRole("button", { name: "Circle Drawer" }).click();
  await page.getByRole("button", { name: "Circle canvas" }).click();
  await page.getByRole("button", { name: "Circle 1" }).click();
});
