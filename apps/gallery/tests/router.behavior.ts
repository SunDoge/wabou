import { test } from "@wabou/test";

test(
  "repeated navigation keeps route parameters and content in sync",
  async ({ page }) => {
    const destinations = ["Input", "Select", "Checkbox", "Button"] as const;
    for (let cycle = 0; cycle < 12; cycle += 1) {
      for (const destination of destinations) {
        await page.getByRole("button", { name: destination }).click();
        await page.getByRole("heading", { name: destination }).waitFor();
      }
    }
  },
  // The test deliberately crosses 96 completed native frame barriers. Under
  // the deterministic 16 ms clock those barriers consume authored virtual
  // time even when the machine executes the suite in under a second.
  { timeout: 10_000 },
);
