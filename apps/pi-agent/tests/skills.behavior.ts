import { expect, test } from "@wabou/test";

test("opens the native Skills catalog and returns to the active project", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Skills" }).click();
  await expect(page.getByRole("heading", { name: "Skills" })).toBeInViewport();
  await expect(
    page.getByRole("textbox", { name: "Search skills" }),
  ).toBeInViewport();

  await page.getByRole("button", { name: "Back to projects" }).click();
  await expect(page.getByRole("button", { name: "Start agent" })).toBeInViewport();
});
