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
  await page.getByRole("button", { name: "Advanced HTTP options" }).click();
  await page.getByRole("textbox", { name: "HTTP request headers" }).waitFor();
  await page.getByRole("textbox", { name: "Download checksum" }).waitFor();
  await page.getByRole("textbox", { name: "Task HTTP proxy" }).waitFor();
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

  await downloads.press("n", { control: true, shift: true });
  await page.getByRole("dialog", { name: "Add download task" }).waitFor();
  await page.getByRole("label", { name: "Choose a .torrent file" }).waitFor();
  await page.getByRole("button", { name: "Cancel" }).click();

  await downloads.press(",", { control: true });
  await page.getByRole("heading", { name: "Settings" }).waitFor();
  await page
    .getByRole("button", { name: "Settings" })
    .press("l", { control: true });
  await page.getByRole("heading", { name: "All Downloads" }).waitFor();
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

test(
  "Motrix warns before quitting with an unfinished task",
  async ({ page }) => {
    await page.getByRole("button", { name: "New task" }).click();
    await page.getByRole("dialog", { name: "Add download task" }).waitFor();
    await page
      .getByRole("textbox", { name: "Download URLs" })
      .type("magnet:?xt=urn:btih:0123456789abcdef0123456789abcdef01234567");
    await page
      .getByRole("textbox", { name: "Output filename" })
      .type("queued-magnet-test");
    await page.getByRole("button", { name: "Create task" }).click();
    await page
      .getByRole("button", { name: "Pause all" })
      .click({ timeout: 5_000 });
    await page
      .getByRole("button", { name: "Downloads" })
      .press("q", { control: true });
    await page.getByRole("dialog", { name: "Confirm quit" }).waitFor();
    await page.getByRole("button", { name: "Keep running" }).click();
    await expect(
      page.getByRole("dialog", { name: "Confirm quit" }),
    ).toBeAbsent();
    await page
      .getByRole("button", {
        name: "Resume [METADATA]0123456789abcdef0123456789abcdef01234567",
      })
      .waitFor({ timeout: 5_000 });
    await page
      .getByRole("button", {
        name: "Inspect [METADATA]0123456789abcdef0123456789abcdef01234567",
      })
      .click({ timeout: 5_000 });
    await page.getByRole("button", { name: "Task activity" }).click();
    await page.getByRole("button", { name: "Task pieces" }).click();
    await page.getByRole("button", { name: "Task trackers" }).click();
    await page
      .getByRole("textbox", { name: "Task tracker URLs" })
      .waitFor({ timeout: 5_000 });
    await page
      .getByRole("button", { name: "Save task trackers" })
      .waitFor({ timeout: 5_000 });
    await page.getByRole("button", { name: "Task overview" }).click();
    await page.getByRole("button", { name: "Move to top" }).click();
    await page.getByRole("button", { name: "Move to bottom" }).click();
  },
  { timeout: 15_000 },
);

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
      .getByRole("alert", {
        name: "Download failed: wabou-behavior-test.bin",
      })
      .waitFor({ timeout: 5_000 });
    await page.getByRole("button", { name: "Dismiss" }).click();
    await page
      .getByRole("button", { name: "Retry wabou-behavior-test.bin" })
      .waitFor({ timeout: 5_000 });
    await page
      .getByRole("button", { name: "Inspect wabou-behavior-test.bin" })
      .click({ timeout: 5_000 });
    await page
      .getByRole("button", { name: "Task files" })
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
  await page.getByRole("button", { name: "Dashboard" }).click();
  const speedProfile = page.getByRole("combobox", { name: "Speed profile" });
  await speedProfile.click();
  await page.getByRole("option", { name: "Balanced" }).click();
  await speedProfile.click();
  await expect(page.getByRole("option", { name: "Balanced" })).toBeSelected();
  await page.getByRole("option", { name: "Balanced" }).click();
  await page.getByRole("button", { name: "Trackers" }).click();
  await page.getByRole("heading", { name: "Trackers" }).waitFor();
  await page.getByRole("textbox", { name: "BitTorrent trackers" }).waitFor();
  await page.getByRole("button", { name: "Notifications" }).click();
  await page.getByRole("heading", { name: "Notifications" }).waitFor();
  await page.getByRole("button", { name: "Settings" }).click();
  await page.getByRole("heading", { name: "Settings" }).waitFor();
  await page.getByRole("button", { name: "Browse…" }).waitFor();
  await page.getByRole("button", { name: "Configure Downloads" }).click();
  await page.getByRole("textbox", { name: "Connections per server" }).waitFor();
  await page.getByRole("textbox", { name: "Minimum split size" }).waitFor();
  await page.getByRole("textbox", { name: "Speed profile 1 name" }).waitFor();
  await page.getByRole("button", { name: "Add profile" }).click();
  await page.getByRole("textbox", { name: "Speed profile 4 name" }).waitFor();
  await page.getByRole("button", { name: "Remove Profile 4" }).click();
  await expect(
    page.getByRole("textbox", { name: "Speed profile 4 name" }),
  ).toBeAbsent();
  await page
    .getByRole("button", { name: "Use none file allocation" })
    .waitFor();
  await page.getByRole("button", { name: "Configure BitTorrent" }).click();
  await page
    .getByRole("switch", { name: "Enable DHT peer discovery" })
    .waitFor();
  await page
    .getByRole("switch", { name: "Enable peer exchange (PEX)" })
    .waitFor();
  await page
    .getByRole("textbox", { name: "Maximum peers per torrent" })
    .waitFor();
  await page.getByRole("textbox", { name: "BT listen port" }).waitFor();
  await page.getByRole("textbox", { name: "DHT listen port" }).waitFor();
  await page.getByRole("textbox", { name: "Seed ratio" }).waitFor();
  await page.getByRole("textbox", { name: "Seed time in minutes" }).waitFor();
  await page.getByRole("button", { name: "Configure Network" }).click();
  await page
    .getByRole("switch", { name: "Use an external aria2 controller" })
    .waitFor();
  await page
    .getByRole("switch", { name: "Enable automatic port mapping" })
    .waitFor();
  await page.getByRole("button", { name: "Automatic" }).waitFor();
  await page.getByRole("status", { name: "Port mapping status" }).waitFor();
  const downloadProxy = page.getByRole("switch", {
    name: "Enable download proxy",
  });
  await downloadProxy.waitFor();
  await downloadProxy.click();
  await page.getByRole("textbox", { name: "Proxy host" }).waitFor();
  await page.getByRole("textbox", { name: "Proxy port" }).waitFor();
  await page.getByRole("textbox", { name: "Proxy bypass hosts" }).waitFor();
  await downloadProxy.click();
  await page.getByRole("button", { name: "Configure Appearance" }).click();
  await page.getByRole("button", { name: "Save settings" }).waitFor();
  await page.getByRole("button", { name: "Use System theme" }).click();
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
