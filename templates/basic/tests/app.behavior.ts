import { test } from "@wabou/test";

test("application starts", async ({ page }) => {
  await page
    .getByRole("label", { name: "Your Wabou application is ready." })
    .waitFor();
});
