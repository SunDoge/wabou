import { defaultHost } from "@wabou/core/renderer";
import { expect, test } from "@wabou/test";

test("gallery layout control paints native diagnostic bounds", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Overview" }).click();
  const overlay = page.getByRole("switch", { name: "Layout overlay" });
  await expect(overlay).toBeUnchecked();
  await overlay.click();
  await expect(overlay).toBeChecked();
  await page.waitForIdle();

  const paint = defaultHost.diagnostics.overlayPaintStats();
  if (!paint) throw new Error("native overlay paint stats are unavailable");
  if (!paint.enabled || paint.layout_bounds === 0) {
    throw new Error(
      `native layout overlay did not paint bounds: ${JSON.stringify(paint)}`,
    );
  }
  expect(paint.sequence > 0).toBe(true);

  const frame = defaultHost.diagnostics.frameStats();
  if (!frame || frame.node_count === 0) {
    throw new Error(
      `headless host did not publish native frame diagnostics: ${JSON.stringify(frame)}`,
    );
  }
  expect(frame.viewport_w > 0).toBe(true);
  expect(frame.viewport_h > 0).toBe(true);
});
