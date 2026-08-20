import { expect, test } from "@wabou/test";

test("open Settings for capture", async ({ page }) => {
  const dashboard = page.getByRole("button", { name: "Dashboard" });
  await page.getByRole("button", { name: "Settings" }).click();
  const heading = page.getByRole("heading", { name: "Settings" });
  await heading.waitFor();
  const after = await dashboard.snapshot();
  const headingSnapshot = await heading.snapshot();
  if (after.bounds.y < 40 || headingSnapshot.bounds.y < 40)
    throw new Error(
      `capture viewport clipped fixed chrome (sidebar y=${after.bounds.y}, heading y=${headingSnapshot.bounds.y})`,
    );
  const categories = ["General", "Appearance", "Downloads", "BitTorrent"].map(
    (name) => page.getByRole("button", { name: `Open ${name} settings` }),
  );
  for (const category of categories.slice(1)) {
    await expect(category).toHaveSameBoundsAs(categories[0], ["width"], {
      tolerance: 1,
    });
  }
});
