import { test } from "@wabou/test";

test("show invalid file-drop feedback for capture", async ({
  page,
  window,
  files,
}) => {
  const path = files.writeText("capture/not-a-torrent.txt", "not a torrent");
  await window.fileDrop(window.current, "entered", [path]);
  await page.getByRole("status", { name: "Torrent drop target" }).waitFor();
  await window.fileDrop(window.current, "dropped", [path]);
  await page
    .getByRole("alert", { name: "Only .torrent files can be dropped here." })
    .waitFor();
});
