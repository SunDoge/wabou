import { test } from "@wabou/test";

test("capture message anatomy in the light theme", async ({ page }) => {
  await page.getByRole("button", { name: "Theme: Dark", index: 0 }).click();
  await page.getByRole("button", { name: "Message" }).click();
  await page.getByRole("heading", { name: "Message" }).waitFor();
  await page.getByRole("group", { name: "Conversation preview" }).waitFor();
});
