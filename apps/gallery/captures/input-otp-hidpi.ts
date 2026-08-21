import { test } from "@wabou/test";

test("capture OTP input at HiDPI", async ({ page }) => {
  await page.getByRole("button", { name: "Input OTP" }).click();
  await page.getByRole("heading", { name: "Input OTP" }).waitFor();
  await page.getByRole("textbox", { name: "Verification code" }).type("123456");
  await page.getByRole("label", { name: "Code complete: 123456" }).waitFor();
});
