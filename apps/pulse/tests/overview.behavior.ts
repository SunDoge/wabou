import { expect, test } from "@wabou/test";
test("sample history controls work through native hit testing", async ({
  page,
}) => {
  const five = page.getByRole("button", { name: "History 5m" });
  await five.click();
  await expect(five).toBePressed();
  await page.getByRole("button", { name: "Pause sample history" }).click();
  await expect(
    page.getByRole("button", { name: "Resume sample history" }),
  ).toBePressed();
  await page.getByRole("button", { name: "Resume sample history" }).click();
  await page.getByRole("button", { name: "Pause sample history" }).waitFor();
});

test("custom title bar replaces native decorations", async ({ page }) => {
  if (
    JSON.parse(
      (
        globalThis as typeof globalThis & {
          __wabou_window_options_json: string;
        }
      ).__wabou_window_options_json,
    ).decorations !== false
  )
    throw new Error("Pulse must disable native window decorations");
  await page.getByRole("button", { name: "Close window" }).waitFor();
  await page.getByRole("button", { name: "Minimize window" }).waitFor();
  await page.getByRole("button", { name: "Toggle window maximized" }).waitFor();
  await page.getByRole("group", { name: "Move Pulse window" }).waitFor();
});
