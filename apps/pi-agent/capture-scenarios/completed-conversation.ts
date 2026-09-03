import { expect, test } from "@wabou/test";

export function registerCompletedConversationCapture(): void {
  test("captures a completed deterministic conversation", async ({ page }) => {
    const composer = page.getByRole("textbox", {
      name: "Ask this agent to work in its repository…",
    });
    await composer.waitFor();
    await composer.type("Review the Wabou renderer");
    await page.getByRole("button", { name: "Send" }).click();
    await expect(
      page.getByRole("label", {
        name: "Fake Pi completed: Review the Wabou renderer",
      }),
    ).toHaveCount(1, { timeout: 5_000 });
  });
}
