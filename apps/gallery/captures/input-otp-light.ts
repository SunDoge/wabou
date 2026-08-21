import { test } from "@wabou/test";

test("capture filtered OTP input in the light theme", async ({ page }) => {
  await page.getByRole("button", { name: "Theme: Dark", index: 0 }).click();
  await page.getByRole("button", { name: "Input OTP" }).click();
  await page.getByRole("heading", { name: "Input OTP" }).waitFor();
  await page
    .getByRole("textbox", { name: "Verification code" })
    .type("12a34567");
  await page.getByRole("label", { name: "Code complete: 123456" }).waitFor();
});
