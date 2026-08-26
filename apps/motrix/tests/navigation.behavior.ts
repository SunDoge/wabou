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
  const wideActivity = await activity.snapshot();
  if (
    wideActivity.bounds.height < 200 ||
    wideActivity.bounds.y + wideActivity.bounds.height < 900 - 48
  ) {
    throw new Error(
      `dashboard activity did not consume the available viewport: ${JSON.stringify(wideActivity.bounds)}`,
    );
  }

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

  await window.resize(window.current, 700, 500);
  const clippedTransfer = await transferOverview.snapshot();
  if (clippedTransfer.bounds.y + clippedTransfer.bounds.height <= 500) {
    throw new Error(
      "small-window dashboard did not produce scrollable overflow",
    );
  }
  await engine.wheel(900);
  await expect(transferOverview).toBeInViewport();

  await window.resize(window.current, 1280, 820);
  const standardActivity = await activity.snapshot();
  // Restored transparent windows reserve a 12px client-decoration inset on
  // every side for their shadow. Keep the activity useful without assuming
  // the full native surface is application content.
  if (
    standardActivity.bounds.height < 200 ||
    standardActivity.bounds.y + standardActivity.bounds.height < 820 - 48
  ) {
    throw new Error(
      `dashboard roomy breakpoint starved activity at the default window size: ${JSON.stringify(standardActivity.bounds)}`,
    );
  }
});

test("Motrix route changes keep the sidebar chrome fixed", async ({
  page,
  window,
}) => {
  await window.resize(window.current, 1024, 700);
  const dashboard = page.getByRole("button", { name: "Dashboard" });
  await expect(
    page.getByRole("button", { name: "Minimize window" }),
  ).toBeInViewport();
  await expect(
    page.getByRole("button", { name: "Maximize window" }),
  ).toBeInViewport();
  await expect(
    page.getByRole("button", { name: "Close window" }),
  ).toBeInViewport();
  await page.getByRole("button", { name: "Hide sidebar" }).click();
  const showSidebar = page.getByRole("button", { name: "Show sidebar" });
  await showSidebar.waitFor();
  await showSidebar.click();
  await page.getByRole("button", { name: "Hide sidebar" }).waitFor();
  const before = await dashboard.snapshot();
  await expect(dashboard).toBeWithinBounds({
    x: 0,
    y: 40,
    width: 1024,
    height: 660,
  });
  await page.getByRole("button", { name: "Settings" }).click();
  const heading = page.getByRole("heading", { name: "Settings" });
  await heading.waitFor();
  await expect(dashboard).toHaveBounds({
    x: before.bounds.x,
    y: before.bounds.y,
  });
  await expect(heading).toBeInViewport();
  await page.getByRole("group", { name: "Settings categories" }).wheel(800);
  await dashboard.click();
  const dashboardHeading = page.getByRole("heading", { name: "Dashboard" });
  await expect(dashboardHeading).toBeInViewport();
  const dashboardHeadingBounds = (await dashboardHeading.snapshot()).bounds;
  const engine = await page
    .getByRole("group", { name: "DOWNLOAD SERVICE statistic" })
    .snapshot();
  if (
    engine.bounds.y <
    dashboardHeadingBounds.y + dashboardHeadingBounds.height
  )
    throw new Error(
      `route navigation retained the previous page scroll offset: heading=${dashboardHeadingBounds.y}, first card=${engine.bounds.y}`,
    );
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
  const search = page.getByRole("textbox", { name: "Search downloads" });
  await expect(search).toBeInViewport();
  await expect(
    page.getByRole("button", { name: "Sort downloads: Newest first" }),
  ).toBeInViewport();
  const downloadsPage = await page
    .getByRole("group", { name: "Downloads page" })
    .snapshot();
  const downloadListLocator = page.getByRole("group", {
    name: "Download list",
  });
  await expect(downloadListLocator).toBeWithinBounds(downloadsPage.bounds, {
    tolerance: 1,
  });
  const downloadList = await downloadListLocator.snapshot();
  const statusSummaryLocator = page.getByRole("status", {
    name: "Download status summary",
  });
  await expect(statusSummaryLocator).toBeWithinBounds(downloadsPage.bounds, {
    tolerance: 1,
  });
  const statusSummary = await statusSummaryLocator.snapshot();
  const pageBottom = downloadsPage.bounds.y + downloadsPage.bounds.height;
  const listBottom = downloadList.bounds.y + downloadList.bounds.height;
  const statusBottom = statusSummary.bounds.y + statusSummary.bounds.height;
  const contentGap = statusSummary.bounds.y - listBottom;
  if (
    Math.abs(contentGap - 12) > 1 ||
    Math.abs(pageBottom - statusBottom) > 1
  ) {
    throw new Error(
      `download content did not fill the page: gap=${contentGap}, page bottom=${pageBottom}, status bottom=${statusBottom}`,
    );
  }
  await page.getByRole("button", { name: "Add a download" }).click();
  await page.getByRole("dialog", { name: "Add download task" }).waitFor();
  await page.getByRole("button", { name: "Cancel" }).click();
  await search.type("no-such-download");
  const clearSearch = page.getByRole("button", { name: "Clear search" });
  await clearSearch.waitFor();
  await clearSearch.click();
  await expect(search).toHaveValue("");
  await expect(
    page.getByRole("button", { name: "Add a download" }),
  ).toBeInViewport();
  const allDownloads = page.getByRole("button", { name: "All" });
  await allDownloads.click();
  await allDownloads.press("ArrowRight");
  await expect(page.getByRole("button", { name: "Downloading" })).toBePressed();
  await page.getByRole("button", { name: "Downloading" }).press("ArrowLeft");
  await expect(allDownloads).toBePressed();
  await window.resize(window.current, 700, 500);
  await expect(search).toBeInViewport();
  await expect(
    page.getByRole("combobox", { name: "Download status" }),
  ).toBeInViewport();
  await expect(
    page.getByRole("button", { name: "Add a download" }),
  ).toBeInViewport();
  await window.resize(window.current, 1280, 820);
});

test("Motrix settings controls remain inside the minimum viewport", async ({
  page,
  window,
}) => {
  await page.getByRole("button", { name: "Settings" }).click();
  await window.resize(window.current, 900, 600);
  const categories = page.getByRole("group", { name: "Settings categories" });
  const categoryBounds = (await categories.snapshot()).bounds;
  for (const name of [
    "General",
    "Appearance",
    "Downloads",
    "BitTorrent",
    "Integration",
    "Network",
    "Advanced",
    "About",
  ]) {
    const category = page.getByRole("button", {
      name: `Open ${name} settings`,
    });
    await expect(category).toBeWithinBounds(categoryBounds, { tolerance: 1 });
    const { bounds } = await category.snapshot();
    if (bounds.height < 96) {
      throw new Error(
        `settings category ${name} collapsed below its content-safe height: ${bounds.height}px`,
      );
    }
  }
  await categories.wheel(800);
  await expect(
    page.getByRole("button", { name: "Open About settings" }),
  ).toBeInViewport();
  await categories.wheel(-800);
  await page.getByRole("button", { name: "Open General settings" }).click();
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
  const themeChoiceBounds = (
    await page.getByRole("group", { name: "Theme choices" }).snapshot()
  ).bounds;
  const themeChoices = ["System", "Light", "Dark"].map((name) =>
    page.getByRole("button", { name: `Use ${name} theme` }),
  );
  for (const choice of themeChoices) {
    await expect(choice).toBeInViewport();
    await expect(choice).toBeWithinBounds(themeChoiceBounds, {
      tolerance: 1,
    });
  }
  await page
    .getByRole("tab", { name: "Configure Appearance" })
    .press("ArrowLeft");
  await expect(general).toBeSelected();
  const save = page.getByRole("button", { name: "Save settings" });
  await general.wheel(600);
  await expect(save).toBeInViewport();
  await save.wheel(-600);
  await page
    .getByRole("button", { name: "Back to settings categories" })
    .click();
  await window.resize(window.current, 1280, 820);
});

test("Motrix settings distinguish saved config from an unsaved draft", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Settings" }).click();
  await page.getByRole("button", { name: "Open General settings" }).click();
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
  const maxConcurrent = page.getByRole("textbox", {
    name: "Concurrent downloads",
  });
  await expect(maxConcurrent).toHaveValue("5");
  await maxConcurrent.press("a", { control: true });
  await maxConcurrent.type("6");
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
  await page.getByRole("tab", { name: "Configure Downloads" }).click();
  await maxConcurrent.press("a", { control: true });
  await maxConcurrent.type("5");
  await expect(maxConcurrent).toHaveValue("5");
  await expect(save).toBeEnabled();
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
  const downloadUrls = page.getByRole("textbox", { name: "Download URLs" });
  await downloadUrls.type("file:///tmp/private");
  await page
    .getByRole("alert", { name: "Download URI validation error" })
    .waitFor();
  await expect(
    page.getByRole("button", { name: "Create task" }),
  ).toBeDisabled();
  await downloadUrls.press("a", { control: true });
  await downloadUrls.press("Backspace");
  await expect(
    page.getByRole("alert", { name: "Download URI validation error" }),
  ).toBeAbsent();
  await downloadUrls.type(
    "https://example.com/file.iso\nhttps://example.com/file-2.iso",
  );
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
  const advanced = page.getByRole("button", {
    name: "Advanced HTTP options",
  });
  await advanced.click();
  await advanced.wheel(420);
  await page.getByRole("textbox", { name: "HTTP request headers" }).waitFor();
  await expect(
    page.getByRole("dialog", { name: "Add download task" }),
  ).toBeWithinBounds({ x: 0, y: 16, width: 900, height: 568 });
  await expect(
    page.getByRole("button", { name: "Create task" }),
  ).toBeInViewport();
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

test("Motrix file drops expose lifecycle feedback and open torrents", async ({
  page,
  window,
  files,
}) => {
  const invalid = files.writeText("drops/readme.txt", "not a torrent");
  await window.fileDrop(window.current, "entered", [invalid]);
  await page.getByRole("status", { name: "Torrent drop target" }).waitFor();
  await window.fileDrop(window.current, "dropped", [invalid]);
  const dropError = page.getByRole("alert", {
    name: "Only .torrent files can be dropped here.",
  });
  await dropError.waitFor();
  await page.getByRole("button", { name: "Dismiss file drop error" }).click();
  await expect(dropError).toBeAbsent();

  const torrent = files.writeText("drops/behavior.torrent", torrentFixture(3));
  await window.fileDrop(window.current, "entered", [torrent]);
  await page.getByRole("status", { name: "Torrent drop target" }).waitFor();
  await window.fileDrop(window.current, "dropped", [torrent]);
  await page.getByRole("dialog", { name: "Add download task" }).waitFor();
  await page
    .getByRole("status", { name: "3 files · 6 B" })
    .waitFor({ timeout: 5_000 });
  await page.getByRole("button", { name: "Cancel" }).click();
});

test("Motrix closes to its tray and restores the native window", async ({
  effects,
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
  effects.respond("windowClose", null);
  await page.getByRole("button", { name: "Close window" }).click();
  await page.waitForIdle();
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
    const priority = page.getByRole("combobox", { name: "Task priority" });
    await priority.click();
    await expect(page.getByRole("option", { name: "Normal" })).toBeSelected();
    await page.getByRole("option", { name: "High" }).click();
    await priority.click();
    await expect(page.getByRole("option", { name: "High" })).toBeSelected({
      timeout: 5_000,
    });
    await page.getByRole("option", { name: "High" }).click();
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
  async ({ effects, page, window }) => {
    await page.getByRole("button", { name: "New task" }).click();
    await page.getByRole("dialog", { name: "Add download task" }).waitFor();
    await page.getByRole("button", { name: "Links" }).click();
    const downloadUrls = page.getByRole("textbox", { name: "Download URLs" });
    await expect(downloadUrls).toHaveValue("");
    await downloadUrls.type("http://127.0.0.1:9/wabou-behavior-test.bin");
    await page
      .getByRole("textbox", { name: "Output filename" })
      .type("wabou-behavior-test.bin");
    const highPriority = page.getByRole("button", { name: "High" });
    await highPriority.click();
    await expect(highPriority).toBePressed();
    await page.getByRole("button", { name: "Create task" }).click();
    await page
      .getByRole("alert", {
        name: "Download failed: wabou-behavior-test.bin",
      })
      .waitFor({ timeout: 5_000 });
    await page.getByRole("button", { name: "Dismiss" }).click();
    await page.getByRole("label", { name: "High priority" }).waitFor();
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
    const retryTask = page.getByRole("button", {
      name: "Retry wabou-behavior-test.bin",
    });
    await retryTask.waitFor({ timeout: 5_000 });
    await retryTask.click();
    await page
      .getByRole("alert", {
        name: "Download failed: wabou-behavior-test.bin",
      })
      .waitFor({ timeout: 5_000 });
    await page.getByRole("button", { name: "Dismiss" }).click();
    await page.getByRole("label", { name: "High priority" }).waitFor();
    await page.getByRole("button", { name: "Notifications" }).click();
    await page
      .getByRole("button", {
        name: "View Download failed: wabou-behavior-test.bin",
        index: 0,
      })
      .click({ timeout: 5_000 });
    await expect(page.getByRole("button", { name: "Downloads" })).toBeCurrent(
      "page",
    );
    await page
      .getByRole("button", { name: "Close inspector" })
      .waitFor({ timeout: 5_000 });
    await page.getByRole("group", { name: "Added" }).waitFor();
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
    effects.respond("clipboardWrite", null);
    await page.getByRole("button", { name: "Copy source" }).click();
    // Crossing the host barrier and the per-test fixture-consumption check
    // prove the clipboard request completed without coupling this workflow to
    // a transient visual notice inside the inspector.
    await page.waitForIdle();
    const compactDetailsBox = await compactDetails.snapshot();
    if (compactDetailsBox.bounds.height < 540) {
      throw new Error(
        `compact detail surface wastes viewport height: ${compactDetailsBox.bounds.height}px of 600px`,
      );
    }
    await expect(
      page.getByRole("button", { name: "Close inspector" }),
    ).toBeInViewport();
    await window.resize(window.current, 700, 500);
    const narrowDetailsBox = await compactDetails.snapshot();
    if (
      narrowDetailsBox.bounds.x < 0 ||
      narrowDetailsBox.bounds.x + narrowDetailsBox.bounds.width > 700 ||
      narrowDetailsBox.bounds.y < 0 ||
      narrowDetailsBox.bounds.y + narrowDetailsBox.bounds.height > 500
    ) {
      throw new Error(
        `narrow task details escaped the viewport: ${JSON.stringify(narrowDetailsBox.bounds)}`,
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
    effects.respond("contextMenuShow", "priority");
    const sortDownloads = page.getByRole("button", {
      name: "Sort downloads: Newest first",
    });
    await sortDownloads.click();
    await page
      .getByRole("button", { name: "Sort downloads: Priority" })
      .waitFor();
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

test(
  "Motrix respects the show-downloads-after-adding preference",
  async ({ page }) => {
    const preference = page.getByRole("switch", {
      name: "Show downloads after creating a task",
    });
    await page.getByRole("button", { name: "Settings" }).click();
    await page.getByRole("button", { name: "Open General settings" }).click();
    await preference.click();
    await page.getByRole("button", { name: "Save settings" }).click();
    await page.getByRole("button", { name: "Dashboard" }).click();
    await page.getByRole("button", { name: "New task" }).click();
    await page
      .getByRole("textbox", { name: "Download URLs" })
      .type("http://127.0.0.1:9/stay-on-dashboard.bin");
    await page.getByRole("button", { name: "Create task" }).click();
    await page
      .getByRole("alert", { name: "Download failed: stay-on-dashboard.bin" })
      .waitFor({ timeout: 5_000 });
    await expect(page.getByRole("button", { name: "Dashboard" })).toBeCurrent(
      "page",
    );
    await expect(
      page.getByRole("heading", { name: "Dashboard" }),
    ).toBeInViewport();
    await page.getByRole("button", { name: "Dismiss" }).click();

    await page.getByRole("button", { name: "Settings" }).click();
    await page.getByRole("button", { name: "Open General settings" }).click();
    await preference.click();
    await page.getByRole("button", { name: "Save settings" }).click();
    await page.getByRole("button", { name: "Downloads" }).click();
    await page
      .getByRole("button", { name: "Remove stay-on-dashboard.bin" })
      .click();
    await page.getByRole("button", { name: "Remove" }).click();
    await expect(
      page.getByRole("button", { name: "Remove stay-on-dashboard.bin" }),
    ).toBeAbsent({ timeout: 5_000 });
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
  await page
    .getByRole("status", { name: "Speed profile Balanced applied." })
    .waitFor();
  await speedProfile.click();
  await expect(page.getByRole("option", { name: "Balanced" })).toBeSelected();
  await page.getByRole("option", { name: "Unlimited" }).click();
  await page
    .getByRole("status", { name: "Speed profile Unlimited applied." })
    .waitFor();
  await page.getByRole("button", { name: "Notifications" }).click();
  await page.getByRole("heading", { name: "Notifications" }).waitFor();
  await expect(
    page.getByRole("group", { name: "Notification history" }),
  ).toBeInViewport();
  await expect(
    page.getByRole("button", {
      name: "View Download failed: wabou-behavior-test.bin",
    }),
  ).toHaveCount(2);
  await page
    .getByRole("button", {
      name: "View Download failed: wabou-behavior-test.bin",
      index: 0,
    })
    .click();
  await page
    .getByRole("alert", {
      name: "Task unavailable: wabou-behavior-test.bin",
    })
    .waitFor();
  await expect(
    page.getByRole("group", { name: "Notification history" }),
  ).toBeInViewport();
  await page.getByRole("button", { name: "Clear all" }).click();
  await expect(
    page.getByRole("heading", { name: "No recent notifications" }),
  ).toBeInViewport();
  await expect(
    page.getByRole("group", { name: "Notification history" }),
  ).toBeAbsent();
  await page.getByRole("button", { name: "Settings" }).click();
  await page.getByRole("heading", { name: "Settings" }).waitFor();
  await page.getByRole("button", { name: "Open General settings" }).click();
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
      name: "Settings saved and applied.",
    })
    .waitFor();
  await expect(page.getByRole("button", { name: "Restart now" })).toBeAbsent();
  await page.getByRole("tab", { name: "Configure Advanced" }).click();
  await page.getByRole("button", { name: "Open folder" }).waitFor();
  await page.getByRole("tab", { name: "Configure About" }).click();
  await page.getByRole("button", { name: "Open repository" }).waitFor();
});

test(
  "Motrix clears stopped tasks through one confirmed batch action",
  async ({ page }) => {
    await page.getByRole("button", { name: "Downloads" }).click();
    await page.getByRole("button", { name: "New task" }).click();
    await page
      .getByRole("textbox", { name: "Download URLs" })
      .type("http://127.0.0.1:9/clear-stopped.bin");
    await page
      .getByRole("textbox", { name: "Output filename" })
      .type("clear-stopped.bin");
    await page.getByRole("button", { name: "Create task" }).click();
    await page
      .getByRole("alert", { name: "Download failed: clear-stopped.bin" })
      .waitFor({ timeout: 5_000 });
    await page.getByRole("button", { name: "Dismiss" }).click();
    await page.getByRole("button", { name: "Clear stopped" }).click();
    await page.getByRole("dialog", { name: "Remove download tasks" }).waitFor();
    await expect(
      page.getByRole("checkbox", {
        name: "Also move downloaded files to Trash",
      }),
    ).toBeInViewport();
    await page.getByRole("button", { name: "Remove" }).click();
    await expect(
      page.getByRole("button", { name: "Inspect clear-stopped.bin" }),
    ).toBeAbsent({ timeout: 5_000 });
    await expect(
      page.getByRole("button", { name: "Clear stopped" }),
    ).toBeAbsent();
  },
  { timeout: 10_000 },
);
