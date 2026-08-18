import { test } from "@wabou/test";

test("compiled messages react to locale and input changes", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Tabs" }).wheel(800);
  await page.getByRole("button", { name: "Internationalization" }).click();
  await page.getByRole("label", { name: "Compiled localization" }).waitFor();

  await page.getByRole("button", { name: "中文" }).click();
  await page.getByRole("label", { name: "编译型本地化" }).waitFor();
  await page.getByRole("label", { name: "1 个原生组件" }).waitFor();

  await page.getByRole("button", { name: "+1" }).click();
  await page.getByRole("label", { name: "2 个原生组件" }).waitFor();
});
