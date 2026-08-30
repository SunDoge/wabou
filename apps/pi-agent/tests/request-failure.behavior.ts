import { expect, test } from "@wabou/test";

test("keeps the conversation and draft when a prompt request fails", async ({
  page,
}) => {
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

  // Leave the shared deterministic suite stopped while retaining the local
  // workspace surface. A later prompt can lazily restart Pi.
  await composer.press("a", { control: true });
  await composer.press("Backspace");
  await composer.type("Exit fixture");
  await page.getByRole("button", { name: "Send" }).click();
  await expect(composer).toBeInViewport({ timeout: 5_000 });
  await expect(
    page.getByRole("button", { name: "Workspace files" }),
  ).toBeEnabled();
});
