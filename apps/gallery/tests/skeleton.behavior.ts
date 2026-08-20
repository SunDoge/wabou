import { test } from "@wabou/test";

test("skeleton renders a measured transform-only loading sweep", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Skeleton" }).click();
  await page.getByRole("heading", { name: "Skeleton" }).waitFor();
});
