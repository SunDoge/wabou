import { expect, test } from "@wabou/test";

test("scroll the compact Dashboard for capture", async ({ page }) => {
  const engine = page.getByRole("group", {
    name: "DOWNLOAD SERVICE statistic",
  });
  const transfer = page.getByRole("group", { name: "Transfer overview" });
  await engine.wheel(900);
  await expect(transfer).toBeInViewport();
});
