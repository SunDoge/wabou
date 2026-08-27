import { expect, test } from "@wabou/test";

test("returns to an existing agent after creating a new one", async ({
  page,
}) => {
  const first = page.getByRole("button", { name: "Agent 1" });
  await expect(first).toBeSelected();

  await page.getByRole("button", { name: "New agent" }).click();
  const second = page.getByRole("button", { name: "Agent 2" });
  await expect(second).toBeSelected();

  await first.click();
  await expect(first).toBeSelected();
  await expect(second).toBeDeselected();
});
