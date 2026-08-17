import { expect, test } from "@wabou/test";

test("slider exposes native value state and keyboard interaction", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Slider" }).click();
  const slider = page.getByRole("slider", { name: "Volume" });
  await slider.press("End");
  await expect(slider).toHaveValue("100 percent");
  await expect(page.getByRole("status", { name: "Slider value" })).toHaveText(
    "100%",
  );
  await expect(
    page.getByRole("slider", { name: "Unavailable range" }),
  ).toBeDisabled();
});
