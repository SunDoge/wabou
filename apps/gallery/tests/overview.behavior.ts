import { expect, test } from "@wabou/test";

test("overview demonstrates native inspection and routes into components", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Overview" }).click();
  await page
    .getByRole("label", { name: "Desktop UI, without a browser engine." })
    .waitFor();

  await page.getByRole("button", { name: "FrameTimeline" }).click();
  await page.getByRole("label", { name: "retained vector scene" }).waitFor();

  const slider = page.getByRole("slider", { name: "Motion intensity" });
  await slider.press("End");
  await expect(slider).toHaveRange({ value: 100, min: 0, max: 100 });

  await page.getByRole("button", { name: "Explore components" }).click();
  await page.getByRole("heading", { name: "Button" }).waitFor();
});
