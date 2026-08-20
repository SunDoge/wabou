import { test } from "@wabou/test";

test("open Settings for capture", async ({ page }) => {
  const dashboard = page.getByRole("button", { name: "Dashboard" });
  await page.getByRole("button", { name: "Settings" }).click();
  const heading = page.getByRole("heading", { name: "Settings" });
  await heading.waitFor();
  const after = await dashboard.snapshot();
  const headingSnapshot = await heading.snapshot();
  if (after.bounds.y < 60 || headingSnapshot.bounds.y < 12)
    throw new Error(
      `capture viewport clipped fixed chrome (sidebar y=${after.bounds.y}, heading y=${headingSnapshot.bounds.y})`,
    );
  const sectionTabs = await Promise.all(
    ["General", "Appearance", "Downloads", "BitTorrent"].map((name) =>
      page.getByRole("tab", { name: `Configure ${name}` }).snapshot(),
    ),
  );
  const widths = sectionTabs.map((tab) => tab.bounds.width);
  if (Math.max(...widths) - Math.min(...widths) > 1)
    throw new Error(`settings section columns have unequal widths: ${widths}`);
});
