import { test } from "@wabou/test";

test("capture message anatomy at HiDPI", async ({ page }) => {
  await page.getByRole("button", { name: "Message" }).click();
  await page.getByRole("heading", { name: "Message" }).waitFor();
  await page.getByRole("group", { name: "Conversation preview" }).waitFor();
});
