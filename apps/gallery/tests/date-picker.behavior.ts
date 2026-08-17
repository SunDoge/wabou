import { expect, test } from "@wabou/test";

test("date picker changes month and exposes the selected date", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Date picker" }).click();
  await page.getByRole("button", { name: "Deployment date" }).click();
  await page.getByRole("button", { name: "Next month" }).click();
  await page
    .getByRole("button", { name: "Thursday, September 17, 2026" })
    .click();
  await page.waitForIdle();
  await expect(page.getByRole("status", { name: "Selected date" })).toHaveText(
    "2026-09-17",
  );
});

test("date picker uses standard Intl locale week data and labels", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Date picker" }).click();
  await page.getByRole("button", { name: "本地化日期" }).click();
  await page
    .getByRole("button", { name: "2026年8月17日星期一" })
    .click();
});

test("today uses the host-provided local calendar date", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Date picker" }).click();
  await page.getByRole("button", { name: "Deployment date" }).click();
  await page.getByRole("button", { name: "Select today" }).click();
  await page.waitForIdle();
});
