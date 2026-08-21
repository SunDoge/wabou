import { test } from "@wabou/test";

test("capture button, keyboard and dragged carousel navigation", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Theme: Dark", index: 0 }).click();
  await page.getByRole("button", { name: "Carousel" }).click();
  await page.getByRole("heading", { name: "Carousel" }).waitFor();

  await page.getByRole("button", { name: "Next slide" }).click();
  await page.getByRole("status", { name: "Slide 2 of 3" }).waitFor();
  await page.getByRole("button", { name: "Next slide" }).press("ArrowRight");
  await page.getByRole("status", { name: "Slide 3 of 3" }).waitFor();
  await page.getByRole("group", { name: "Feature slides" }).dragBy(260, 0);
  await page.getByRole("status", { name: "Slide 2 of 3" }).waitFor();

  await page.getByRole("button", { name: "Next vertical slide" }).click();
  await page.getByRole("status", { name: "Vertical slide 2 of 3" }).waitFor();
});
