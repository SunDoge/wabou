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

  await page.getByRole("button", { name: "Show 24 rows" }).click();
  await page.waitForIdle();

  // The host clamps its now-unreachable native offset and reports the
  // correction to TanStack Virtual. A stale JS offset would leave the list
  // empty after this immutable extent change.
  await page
    .getByRole("option", { name: "Row 23 — gamma variant 3" })
    .waitFor();
});
