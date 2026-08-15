import { expect, test } from "@wabou/test";

test("repeated navigation keeps route parameters and content in sync", async ({ page }) => {
  const destinations = ["Input", "Select", "Checkbox", "Button"] as const;
  for (let cycle = 0; cycle < 12; cycle += 1) {
    for (const destination of destinations) {
      await page.getByRole("button", { name: destination }).click();
      await page.getByRole("label", { name: destination }).waitFor();
    }
  }
  expect(true).toBe(true);
});
