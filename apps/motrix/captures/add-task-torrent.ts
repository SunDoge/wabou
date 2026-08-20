import { test } from "@wabou/test";

function torrentFixture(files: number): string {
  const entries = Array.from({ length: files }, (_, index) => {
    const path = `folder-${String(index + 1).padStart(2, "0")}/file-${String(index + 1).padStart(2, "0")}.bin`;
    return `d6:lengthi${index + 1}e4:pathl${path.length}:${path}ee`;
  }).join("");
  return `d4:infod5:filesl${entries}e4:name12:fixture-packee`;
}

test(
  "inspect a deterministic torrent fixture for capture",
  async ({ page, effects, files }) => {
    const path = files.writeText(
      "torrents/many-files.torrent",
      torrentFixture(30),
    );
    await page.getByRole("button", { name: "New task" }).click();
    await page.getByRole("button", { name: "Torrent file" }).click();
    effects.respond("dialogOpen", [path]);
    await page.getByRole("button", { name: "Browse…" }).click();
    const summary = page.getByRole("status", { name: "30 files · 465 B" });
    await summary.waitFor({ timeout: 5_000 });
    await page
      .getByRole("checkbox", { name: "folder-01/file-01.bin" })
      .wheel(200);
    await page
      .getByRole("checkbox", { name: "folder-10/file-10.bin" })
      .waitFor({ timeout: 5_000 });
  },
  { timeout: 10_000 },
);
