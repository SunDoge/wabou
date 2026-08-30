import { expect, test } from "@wabou/test";

test("keeps the conversation and draft when a prompt request fails", async ({
  page,
}) => {
  await page
    .getByRole("button", { name: "Start agent" })
    .click({ timeout: 100 })
    .catch(() => {});
  const composer = page.getByRole("textbox", {
    name: "Ask this agent to work in its repository…",
  });
  await composer.waitFor();
  await composer.type("Reject request");
  await page.getByRole("button", { name: "Send" }).click();

  await expect(composer).toHaveValue("Reject request", { timeout: 5_000 });
  await expect(
    page.getByRole("label", { name: "Fixture rejected request" }),
  ).toHaveCount(1);
  await expect(page.getByRole("button", { name: "Send" })).toBeEnabled();
  await expect(page.getByRole("button", { name: "Start agent" })).toBeAbsent();

  await page.getByRole("button", { name: "Retry request" }).click();
  await expect(
    page.getByRole("label", { name: "Fixture rejected request" }),
  ).toHaveCount(2, { timeout: 5_000 });
  await expect(composer).toHaveValue("Reject request", { timeout: 5_000 });

  // Leave the shared deterministic suite in its normal stopped state.
  await composer.press("a", { control: true });
  await composer.press("Backspace");
  await composer.type("Exit fixture");
  await page.getByRole("button", { name: "Send" }).click();
  await expect(
    page.getByRole("button", { name: "Start agent" }),
  ).toBeInViewport({
    timeout: 5_000,
  });
});
