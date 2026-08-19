import { test } from "@wabou/test";

test("D3 chart geometry reaches typed native paths", async ({ page }) => {
  await page.getByRole("button", { name: "Chart experiment" }).click();
  await page
    .getByRole("img", { name: "Download and upload speed chart" })
    .waitFor();
  await page.getByRole("label", { name: "Download 82 MB/s" }).waitFor();
  await page.getByRole("label", { name: "Upload 24 MB/s" }).waitFor();
});
