import { expect, test } from "@wabou/test";

test("hover card exposes a keyboard-operable preview", async ({ page }) => {
  await page.getByRole("button", { name: "Hover card" }).click();
  const trigger = page.getByRole("button", { name: "Preview Wabou" });

  await expect(trigger).toBeCollapsed();
  await trigger.click();
  await expect(trigger).toBeExpanded();
  await expect(
    page.getByRole("status", { name: "Project summary" }),
  ).toHaveText(
    "Native desktop applications composed with Solid and rendered by Rust.",
  );
  await trigger.press("Escape");
  await expect(trigger).toBeCollapsed();
});
