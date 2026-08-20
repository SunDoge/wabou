import { expect, test } from "@wabou/test";

function torrentFixture(files: number): string {
  const entries = Array.from({ length: files }, (_, index) => {
    const path = `folder-${String(index + 1).padStart(2, "0")}/file-${String(index + 1).padStart(2, "0")}.bin`;
    return `d6:lengthi${index + 1}e4:pathl${path.length}:${path}ee`;
  }).join("");
  return `d4:infod5:filesl${entries}e4:name12:fixture-packee`;
}

test("Motrix dashboard reflows at its minimum window size", async ({
  page,
  window,
}) => {
  await expect(page.getByRole("button", { name: "Dashboard" })).toBeCurrent(
    "page",
  );
  await expect(page.getByRole("button", { name: "Plugins" })).toBeAbsent();
  const engine = page.getByRole("group", {
    name: "DOWNLOAD SERVICE statistic",
  });
  const upload = page.getByRole("group", { name: "UPLOAD statistic" });
  const uploadCharts = page.getByRole("img", {
    name: "Upload throughput chart",
  });
  const uploadCardChart = page.getByRole("img", {
    name: "Upload throughput chart",
    index: 0,
  });
  const transferOverview = page.getByRole("group", {
    name: "Transfer overview",
  });
  const activity = page.getByRole("group", { name: "Download activity" });

  await expect(uploadCharts).toHaveCount(2);
  await expect(activity).toBeInViewport();

  await window.resize(window.current, 1440, 900);
  const wideEngine = await engine.snapshot();
  const wideUpload = await upload.snapshot();
  const wideChart = await uploadCardChart.snapshot();
  await expect(upload).toHaveBounds(
    { y: wideEngine.bounds.y },
    { tolerance: 1 },
  );
  await expect(uploadCardChart).toBeWithinBounds(wideUpload.bounds, {
    tolerance: 1,
  });

  await window.resize(window.current, 900, 600);
  const compactEngine = await engine.snapshot();
  const compactBelowEngine =
    compactEngine.bounds.y + compactEngine.bounds.height;
  await expect(upload).toBeWithinBounds(
    {
      x: 0,
      y: compactBelowEngine,
      width: 900,
      height: 600 - compactBelowEngine,
    },
    { tolerance: 1 },
  );
  const compactUpload = await upload.snapshot();
  await expect(uploadCardChart).toBeWithinBounds(compactUpload.bounds, {
    tolerance: 1,
  });
  await expect(transferOverview).toBeInViewport();
  await expect(activity).toBeAbsent();
  const compactChart = await uploadCardChart.snapshot();
  if (Math.abs(compactChart.bounds.width - wideChart.bounds.width) < 1) {
    throw new Error(
      `responsive chart kept its stale width after reflow: ${wideChart.bounds.width} -> ${compactChart.bounds.width}`,
    );
  }

  await window.resize(window.current, 1280, 820);
});

test("Motrix route changes keep the sidebar chrome fixed", async ({
  page,
  window,
}) => {
  await window.resize(window.current, 1024, 700);
  const dashboard = page.getByRole("button", { name: "Dashboard" });
  const before = await dashboard.snapshot();
  await expect(dashboard).toBeWithinBounds({
    x: 0,
    y: 60,
    width: 1024,
    height: 640,
  });
  await page.getByRole("button", { name: "Settings" }).click();
  const heading = page.getByRole("heading", { name: "Settings" });
  await heading.waitFor();
  await expect(dashboard).toHaveBounds({
    x: before.bounds.x,
    y: before.bounds.y,
  });
  await expect(heading).toBeInViewport();
  await dashboard.click();
  await window.resize(window.current, 1280, 820);
});

test("Motrix download controls remain inside the minimum viewport", async ({
  page,
  window,
}) => {
  await page.getByRole("button", { name: "Downloads" }).click();
  await expect(page.getByRole("button", { name: "Downloads" })).toBeCurrent(
    "page",
  );
  await window.resize(window.current, 900, 600);
  await expect(
    page.getByRole("textbox", { name: "Search downloads" }),
  ).toBeInViewport();
  await expect(
    page.getByRole("button", { name: "Sort downloads" }),
  ).toBeInViewport();
  const allDownloads = page.getByRole("button", { name: "All" });
  await allDownloads.click();
  await allDownloads.press("ArrowRight");
  await expect(page.getByRole("button", { name: "Downloading" })).toBePressed();
  await page.getByRole("button", { name: "Downloading" }).press("ArrowLeft");
  await expect(allDownloads).toBePressed();
  await window.resize(window.current, 1280, 820);
});

test("Motrix settings controls remain inside the minimum viewport", async ({
  page,
  window,
}) => {
  await page.getByRole("button", { name: "Settings" }).click();
  await window.resize(window.current, 900, 600);
  await expect(
    page.getByRole("switch", {
      name: "Show downloads after creating a task",
    }),
  ).toBeInViewport();
  await expect(
    page.getByRole("tab", { name: "Configure About" }),
  ).toBeInViewport();
  const general = page.getByRole("tab", { name: "Configure General" });
  await general.click();
  await general.press("ArrowRight");
  await expect(
    page.getByRole("tab", { name: "Configure Appearance" }),
  ).toBeSelected();
  await page
    .getByRole("tab", { name: "Configure Appearance" })
    .press("ArrowLeft");
  await expect(general).toBeSelected();
  const save = page.getByRole("button", { name: "Save settings" });
  await general.wheel(600);
  await expect(save).toBeInViewport();
  await save.wheel(-600);
  await window.resize(window.current, 1280, 820);
});

test("Motrix settings distinguish saved config from an unsaved draft", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Settings" }).click();
  await page.getByRole("status", { name: "Settings up to date" }).waitFor();
  const save = page.getByRole("button", { name: "Save settings" });
  await expect(save).toBeDisabled();
  await page.getByRole("tab", { name: "Configure Downloads" }).click();
  const split = page.getByRole("textbox", { name: "Default split count" });
  await split.type("x");
  await page
    .getByRole("alert", { name: "Settings validation error" })
    .waitFor();
  await expect(save).toBeDisabled();
  await split.press("Backspace");
  await expect(
    page.getByRole("alert", { name: "Settings validation error" }),
  ).toBeAbsent();
  await expect(save).toBeDisabled();
  const downloadLimit = page.getByRole("textbox", {
    name: "Maximum download speed",
  });
  await downloadLimit.type(".5M");
  await page
    .getByRole("alert", { name: "Settings validation error" })
    .waitFor();
  await expect(save).toBeDisabled();
  await downloadLimit.press("Backspace");
  await downloadLimit.press("Backspace");
  await downloadLimit.press("Backspace");
  await expect(
    page.getByRole("alert", { name: "Settings validation error" }),
  ).toBeAbsent();
  await page.getByRole("tab", { name: "Configure General" }).click();
  const setting = page.getByRole("switch", {
    name: "Show downloads after creating a task",
  });
  await setting.click();
  await page.getByRole("status", { name: "Unsaved settings" }).waitFor();
  await expect(save).toBeEnabled();
  await save.click();
  await page
    .getByRole("status", { name: "Settings saved and applied." })
    .waitFor();
  await expect(save).toBeDisabled();

  // Restore the original value so the behavior suite does not persist a
  // user-visible configuration change.
  await setting.click();
  await save.click();
  await page
    .getByRole("status", { name: "Settings saved and applied." })
    .waitFor();
});

test("Motrix routes and opens the add-task modal", async ({
  page,
  effects,
  window,
}) => {
  await window.resize(window.current, 900, 600);
  await page.getByRole("button", { name: "Downloads" }).click();
  await page.getByRole("heading", { name: "All Downloads" }).waitFor();
  await page.getByRole("button", { name: "New task" }).click();
  await page.getByRole("dialog", { name: "Add download task" }).waitFor();
  await page.getByRole("button", { name: "Links" }).click();
  await page.getByRole("button", { name: "Paste" }).waitFor();
  await page
    .getByRole("textbox", { name: "Download URLs" })
    .type("https://example.com/file.iso\nhttps://example.com/file-2.iso");
  await page
    .getByRole("textbox", { name: "Output filename" })
    .type("shared.iso");
  await page
    .getByRole("alert", {
      name: "A custom output filename can only be used with one URL.",
    })
    .waitFor();
  effects.respond("dialogPickDirectory", ["/tmp/wabou-motrix-downloads"]);
  await page.getByRole("button", { name: "Browse save directory" }).click();
  await expect(
    page.getByRole("textbox", { name: "Save directory" }),
  ).toHaveValue("/tmp/wabou-motrix-downloads");
  await page.getByRole("button", { name: "Advanced HTTP options" }).click();
  await page.getByRole("textbox", { name: "HTTP request headers" }).waitFor();
  await expect(
    page.getByRole("dialog", { name: "Add download task" }),
  ).toBeWithinBounds({ x: 0, y: 16, width: 900, height: 568 });
  await expect(
    page.getByRole("button", { name: "Create task" }),
  ).toBeInViewport();
  const split = page.getByRole("textbox", { name: "Split count" });
  await split.type("x");
  await page
    .getByRole("alert", { name: "Add task validation error" })
    .waitFor();
  await expect(
    page.getByRole("button", { name: "Create task" }),
  ).toBeDisabled();
  await split.press("Backspace");
  await expect(
    page.getByRole("alert", { name: "Add task validation error" }),
  ).toBeAbsent();
  await page.getByRole("button", { name: "Torrent file" }).click();
  await page.getByRole("label", { name: "Choose a .torrent file" }).waitFor();
  effects.respond("dialogOpen", ["/wabou-test-does-not-exist/missing.torrent"]);
  await page.getByRole("button", { name: "Browse…" }).click();
  await page.getByRole("alert", { name: "Add task error" }).waitFor();
  await page.getByRole("button", { name: "Cancel" }).click();
  await window.resize(window.current, 1280, 820);
});

test("Motrix application shortcuts create tasks and toggle the sidebar", async ({
  page,
}) => {
  const downloads = page.getByRole("button", { name: "Downloads" });
  await downloads.press("n", { control: true });
  await page.getByRole("dialog", { name: "Add download task" }).waitFor();
  await expect(
    page.getByRole("textbox", { name: "Download URLs" }),
  ).toHaveValue("");
  await expect(
    page.getByRole("textbox", { name: "Output filename" }),
  ).toHaveValue("");
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

test(
  "Motrix inspects and selects files from an isolated torrent fixture",
  async ({ page, effects, files }) => {
    const path = files.writeText(
      "torrents/behavior.torrent",
      torrentFixture(30),
    );
    await page.getByRole("button", { name: "New task" }).click();
    await page.getByRole("button", { name: "Torrent file" }).click();
    effects.respond("dialogOpen", [path]);
    await page.getByRole("button", { name: "Browse…" }).click();

    const summary = page.getByRole("status", { name: "30 files · 465 B" });
    await summary.waitFor({ timeout: 5_000 });

    const selectAll = page.getByRole("checkbox", {
      name: "Select all files",
    });
    await expect(selectAll).toBeChecked();
    const first = page.getByRole("checkbox", {
      name: "folder-01/file-01.bin",
    });
    await expect(first).toBeChecked();
    await first.wheel(200);
    const tenth = page.getByRole("checkbox", {
      name: "folder-10/file-10.bin",
    });
    await expect(tenth).toBeChecked();
    await tenth.click();
    await expect(tenth).toBeUnchecked();
    await expect(selectAll).toBeIndeterminate();
    await page.getByRole("button", { name: "Cancel" }).click();
  },
  { timeout: 10_000 },
);

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
      .type(
        "magnet:?xt=urn:btih:0123456789abcdef0123456789abcdef01234567&dn=queued-magnet-test",
      );
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
        name: "Resume queued-magnet-test",
      })
      .waitFor({ timeout: 5_000 });
    await page
      .getByRole("button", {
        name: "Inspect queued-magnet-test",
      })
      .click({ timeout: 5_000 });
    await page.getByRole("tab", { name: "Task activity" }).click();
    await page.getByRole("tab", { name: "Task overview" }).click();
    await page
      .getByRole("button", { name: "Remove queued-magnet-test" })
      .click();
    await page.getByRole("dialog", { name: "Remove download tasks" }).waitFor();
    await page.getByRole("button", { name: "Remove" }).click();
    await expect(
      page.getByRole("button", { name: "Inspect queued-magnet-test" }),
    ).toBeAbsent({ timeout: 5_000 });
  },
  { timeout: 15_000 },
);

test(
  "Motrix inspects and safely removes a real download task",
  async ({ page, window }) => {
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
    const taskSelection = page.getByRole("checkbox", {
      name: "Select wabou-behavior-test.bin",
    });
    await taskSelection.click();
    await expect(
      page.getByRole("button", { name: "Close inspector" }),
    ).toBeAbsent();
    // The host publishes immutable task snapshots on a timer. A stable keyed
    // virtual row must update its fields without replacing the focused control.
    // Keep this interval in the recorded trace so replay proves the same race.
    await expect(taskSelection).toBeFocused({
      stableFor: 400,
      timeout: 1_000,
    });
    await taskSelection.click();
    await page
      .getByRole("button", { name: "Retry wabou-behavior-test.bin" })
      .waitFor({ timeout: 5_000 });
    await page
      .getByRole("button", { name: "Inspect wabou-behavior-test.bin" })
      .click({ timeout: 5_000 });
    await expect(
      page.getByRole("dialog", {
        name: "Task details: wabou-behavior-test.bin",
      }),
    ).toBeAbsent();
    await window.resize(window.current, 900, 600);
    const compactDetails = page.getByRole("dialog", {
      name: "Task details: wabou-behavior-test.bin",
    });
    await compactDetails.waitFor();
    const compactDetailsBox = await compactDetails.snapshot();
    if (compactDetailsBox.bounds.height < 540) {
      throw new Error(
        `compact detail surface wastes viewport height: ${compactDetailsBox.bounds.height}px of 600px`,
      );
    }
    await expect(
      page.getByRole("button", { name: "Close inspector" }),
    ).toBeInViewport();
    await page.getByRole("button", { name: "Close inspector" }).click();
    await expect(
      page.getByRole("dialog", {
        name: "Task details: wabou-behavior-test.bin",
      }),
    ).toBeAbsent();
    await window.resize(window.current, 1280, 820);
    await page
      .getByRole("button", { name: "Inspect wabou-behavior-test.bin" })
      .click();
    await page
      .getByRole("tab", { name: "Task files" })
      .click({ timeout: 5_000 });
    await page
      .getByRole("row", { name: "wabou-behavior-test.bin" })
      .waitFor({ timeout: 5_000 });
    await taskSelection.click();
    await page.getByRole("status", { name: "1 downloads selected" }).waitFor();
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
    await expect(
      page.getByRole("status", { name: "1 downloads selected" }),
    ).toBeAbsent();
  },
  { timeout: 15_000 },
);

test("Motrix exposes notifications and supported engine settings", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Dashboard" }).click();
  const speedProfile = page.getByRole("combobox", { name: "Speed profile" });
  await speedProfile.click();
  await page.getByRole("option", { name: "Balanced" }).click();
  await speedProfile.click();
  await expect(page.getByRole("option", { name: "Balanced" })).toBeSelected();
  await page.getByRole("option", { name: "Balanced" }).click();
  await page.getByRole("button", { name: "Notifications" }).click();
  await page.getByRole("heading", { name: "Notifications" }).waitFor();
  await page.getByRole("button", { name: "Settings" }).click();
  await page.getByRole("heading", { name: "Settings" }).waitFor();
  await page
    .getByRole("button", { name: "Browse default download directory" })
    .waitFor();
  await page.getByRole("tab", { name: "Configure Downloads" }).click();
  await page.getByRole("textbox", { name: "Connections per server" }).waitFor();
  await page.getByRole("textbox", { name: "Minimum split size" }).waitFor();
  await page.getByRole("textbox", { name: "Speed profile 1 name" }).waitFor();
  await page.getByRole("button", { name: "Add profile" }).click();
  await page.getByRole("textbox", { name: "Speed profile 4 name" }).waitFor();
  await page.getByRole("button", { name: "Remove Profile 4" }).click();
  await expect(
    page.getByRole("textbox", { name: "Speed profile 4 name" }),
  ).toBeAbsent();
  await page.getByRole("tab", { name: "Configure BitTorrent" }).click();
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
  await page.getByRole("textbox", { name: "Seed ratio" }).waitFor();
  await page.getByRole("tab", { name: "Configure Network" }).click();
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
  await downloadProxy.click();
  await page.getByRole("tab", { name: "Configure Appearance" }).click();
  await page.getByRole("button", { name: "Save settings" }).waitFor();
  await page.getByRole("button", { name: "Use System theme" }).click();
  await page.getByRole("button", { name: "Save settings" }).click();
  await page
    .getByRole("status", {
      name: "Settings saved. Engine changes apply after restart.",
    })
    .waitFor();
  await page.getByRole("button", { name: "Restart now" }).waitFor();
  await page.getByRole("tab", { name: "Configure Advanced" }).click();
  await page.getByRole("button", { name: "Open folder" }).waitFor();
  await page.getByRole("tab", { name: "Configure About" }).click();
  await page.getByRole("button", { name: "Open repository" }).waitFor();
});
