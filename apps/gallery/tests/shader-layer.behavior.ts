import { expect, test } from "@wabou/test";

test("custom shader surface keeps native identity while controls update", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Custom shader" }).click();
  const shader = page.getByRole("img", { name: "Animated aurora shader" });
  await shader.waitFor();
  await page.getByRole("button", { name: "Pause" }).click();
  const resume = page.getByRole("button", { name: "Resume" });
  await resume.waitFor();
  await resume.click();
  await page.getByRole("button", { name: "Pause" }).waitFor();
});
