import { expect, test } from "@wabou/test";

test("managed pagination changes pages without application-side range logic", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Pagination" }).click();
  await expect(page.getByRole("link", { name: "Page 2" })).toBeCurrent("page");

  await page.getByRole("button", { name: "Next page" }).click();
  await expect(
    page.getByRole("status", { name: "Selected pagination page" }),
  ).toHaveText("Selected page 3");
  await expect(page.getByRole("link", { name: "Page 3" })).toBeCurrent("page");
});
