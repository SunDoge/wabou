import { expect, test } from "@wabou/test";

test("expand add-task advanced options for capture", async ({ page }) => {
  await page.getByRole("button", { name: "New task" }).click();
  const advanced = page.getByRole("button", { name: "Advanced HTTP options" });
  await advanced.click();
  await advanced.wheel(100);
  const priority = page.getByRole("button", { name: "High" });
  await priority.click();
  await expect(priority).toBePressed();
  await page.getByRole("textbox", { name: "HTTP request headers" }).waitFor();
});
