import { expect, test } from "@wabou/test";

test("Motrix routes and opens the add-task modal", async ({ page }) => {
  await page.getByRole("button", { name: "Downloads" }).click();
  await page.getByRole("heading", { name: "All Downloads" }).waitFor();
  await page.getByRole("button", { name: "New task" }).click();
  await page.getByRole("dialog", { name: "Add download task" }).waitFor();
  await page.getByRole("button", { name: "Links" }).click();
  await page
    .getByRole("textbox", { name: "Download URLs" })
    .type("https://example.com/file.iso");
  await page.getByRole("button", { name: "Torrent file" }).click();
  await page.getByRole("label", { name: "Choose a .torrent file" }).waitFor();
  await page.getByRole("button", { name: "Cancel" }).click();
});

test("Motrix application shortcuts create tasks and toggle the sidebar", async ({
  page,
}) => {
  const downloads = page.getByRole("button", { name: "Downloads" });
  await downloads.press("n", { control: true });
  await page.getByRole("dialog", { name: "Add download task" }).waitFor();
  await page.getByRole("button", { name: "Cancel" }).click();

  await downloads.press("b", { control: true });
  await page.getByRole("button", { name: "Show sidebar" }).waitFor();
  await page.getByRole("button", { name: "Show sidebar" }).click();
  await downloads.waitFor();
});

test("Motrix closes to its tray and restores the native window", async ({
  page,
  window,
}) => {
  const key = window.current;
  await window.nativeClose(key, "wayland");
  await expect(window).toHaveState(key, {
    presence: "surface-released",
    surfaceGeneration: 1,
  });
  await window.show(key);
  await expect(window).toHaveState(key, {
    presence: "visible",
    surfaceGeneration: 2,
  });
  await page.getByRole("button", { name: "Downloads" }).waitFor();
});

test("Motrix warns before quitting with an unfinished task", async ({
  page,
}) => {
  await page.getByRole("button", { name: "New task" }).click();
  await page.getByRole("dialog", { name: "Add download task" }).waitFor();
  await page
    .getByRole("textbox", { name: "Download URLs" })
    .type("magnet:?xt=urn:btih:0123456789abcdef0123456789abcdef01234567");
  await page.getByRole("button", { name: "Create task" }).click();
  await page
    .getByRole("button", { name: "Pause all" })
    .click({ timeout: 5_000 });
  await page
    .getByRole("button", { name: "Downloads" })
    .press("q", { control: true });
  await page.getByRole("dialog", { name: "Confirm quit" }).waitFor();
  await page.getByRole("button", { name: "Keep running" }).click();
  await expect(page.getByRole("dialog", { name: "Confirm quit" })).toBeAbsent();
});

test(
  "Motrix inspects and safely removes a real aria2 task",
  async ({ page }) => {
    await page.getByRole("button", { name: "New task" }).click();
    await page.getByRole("dialog", { name: "Add download task" }).waitFor();
    await page.getByRole("button", { name: "Links" }).click();
    const downloadUrls = page.getByRole("textbox", { name: "Download URLs" });
    await expect(downloadUrls).toHaveValue("");
    await downloadUrls.type("http://127.0.0.1:9/wabou-behavior-test.bin");
    await page
      .getByRole("textbox", { name: "Output filename" })
      .type("wabou-behavior-test.bin");
    await page.getByRole("button", { name: "Create task" }).click();
    await page
      .getByRole("button", { name: "Inspect wabou-behavior-test.bin" })
      .click({ timeout: 5_000 });
    await page
      .getByRole("button", { name: "Files 1" })
      .click({ timeout: 5_000 });
    await page
      .getByRole("button", { name: "Remove wabou-behavior-test.bin" })
      .click();
    await page.getByRole("dialog", { name: "Remove download tasks" }).waitFor();
    await page
      .getByRole("checkbox", {
        name: "Also move downloaded files to Trash",
      })
      .click();
    await page.getByRole("button", { name: "Remove" }).click();
    await expect(
      page.getByRole("button", { name: "Inspect wabou-behavior-test.bin" }),
    ).toBeAbsent({ timeout: 5_000 });
  },
  { timeout: 15_000 },
);

test("Motrix exposes trackers, notifications and engine settings", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Trackers" }).click();
  await page.getByRole("heading", { name: "Trackers" }).waitFor();
  await page.getByRole("textbox", { name: "BitTorrent trackers" }).waitFor();
  await page.getByRole("button", { name: "Notifications" }).click();
  await page.getByRole("heading", { name: "Notifications" }).waitFor();
  await page.getByRole("button", { name: "Settings" }).click();
  await page.getByRole("heading", { name: "Settings" }).waitFor();
  await page.getByRole("button", { name: "Browse…" }).waitFor();
  await page.getByRole("button", { name: "Configure Network" }).click();
  await page
    .getByRole("switch", { name: "Use an external aria2 controller" })
    .waitFor();
  await page.getByRole("button", { name: "Configure Appearance" }).click();
  await page.getByRole("button", { name: "Save settings" }).waitFor();
  await page.getByRole("button", { name: "Use Dark theme" }).click();
  await page.getByRole("button", { name: "Save settings" }).click();
  await page
    .getByRole("label", {
      name: "Settings saved and engine connection updated.",
    })
    .waitFor();
  await page.getByRole("button", { name: "Configure Advanced" }).click();
  await page.getByRole("button", { name: "Clear history" }).waitFor();
  await page.getByRole("button", { name: "Open folder" }).waitFor();
  await page.getByRole("button", { name: "Configure About" }).click();
  await page.getByRole("button", { name: "Open repository" }).waitFor();
});
