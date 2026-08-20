import { test } from "@wabou/test";

test("open Downloads for capture", async ({ page }) => {
  const dashboard = page.getByRole("button", { name: "Dashboard" });
  const sidebarToggle = page.getByRole("button", { name: "Hide sidebar" });
  await page.getByRole("button", { name: "Downloads" }).click();
  const heading = page.getByRole("heading", { name: "All Downloads" });
  await heading.waitFor();
  const [dashboardBox, toggleBox, headingBox] = await Promise.all([
    dashboard.snapshot(),
    sidebarToggle.snapshot(),
    heading.snapshot(),
  ]);
  if (
    dashboardBox.bounds.y < 60 ||
    toggleBox.bounds.y < 12 ||
    headingBox.bounds.y < 12
  )
    throw new Error(
      `capture viewport clipped fixed chrome (sidebar y=${dashboardBox.bounds.y}, toggle y=${toggleBox.bounds.y}, heading y=${headingBox.bounds.y})`,
    );
});
