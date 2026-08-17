import { test } from "@wabou/test";

test("virtual list materializes new rows through native scrolling", async ({
  page,
}) => {
  const list = page.getByRole("listbox", { name: "Virtual rows" });
  await list.waitFor();
  const firstRow = page.getByRole("option", {
    name: "Row 0 — alpha variant 0",
  });
  await firstRow.waitFor();

  await firstRow.wheel(3_200);
  await page.waitForIdle();

  await page
    .getByRole("option", { name: "Row 100 — gamma variant 14" })
    .waitFor();
});
