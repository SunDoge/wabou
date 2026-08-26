import { test } from "@wabou/test";

test("table-core sorting filtering and selection reach native controls", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Data table" }).click();
  await page.getByRole("columnheader", { name: "Sort by Score" }).click();
  await page.getByRole("textbox", { name: "Filter projects" }).type("router");
  await page.getByRole("row", { name: "Select row router" }).click();
  await page.getByRole("button", { name: "Clear 1 selected" }).click();
});
