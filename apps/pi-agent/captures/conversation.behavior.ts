import { expect, test } from "@wabou/test";

test("captures a completed deterministic conversation", async ({ page }) => {
  await page.getByRole("button", { name: "Start agent" }).click();
  const composer = page.getByRole("textbox", {
    name: "Ask this agent to work in its repository…",
  });
  await composer.waitFor();
  await composer.type("Explain the fixture");
  await page.getByRole("button", { name: "Send" }).click();
  await expect(
    page.getByRole("label", {
      name: "Fake Pi completed: Explain the fixture",
    }),
  ).toHaveCount(1, { timeout: 5_000 });
});
