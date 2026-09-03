import { expect, test } from "@wabou/test";

test("accordion expansion state and animated presence stay in sync", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Accordion" }).click();

  const native = page.getByRole("button", {
    name: "Is Wabou rendered by the browser?",
  });
  const css = page.getByRole("button", {
    name: "Does it support arbitrary browser CSS?",
  });
  const nativeContent = page.getByRole("label", {
    name: "No. Solid produces a retained native tree projected into GPUI by Rust.",
  });
  const cssContent = page.getByRole("label", {
    name: "No. Utilities compile to explicit native style values with predictable semantics.",
  });

  await expect(native).toBeExpanded();
  await expect(css).toBeCollapsed();
  await nativeContent.waitFor();
  await expect(cssContent).toBeAbsent();

  await css.click();
  await expect(native).toBeCollapsed();
  await expect(css).toBeExpanded();
  await expect(nativeContent).toBeAbsent();
  await cssContent.waitFor();

  await css.click();
  await expect(css).toBeCollapsed();
  await expect(cssContent).toBeAbsent();

  const advanced = page.getByRole("button", { name: "Advanced options" });
  const advancedContent = page.getByRole("label", {
    name: "Tracing and renderer diagnostics are available here.",
  });
  await expect(advanced).toBeCollapsed();
  await expect(advancedContent).toBeAbsent();
  await advanced.click();
  await expect(advanced).toBeExpanded();
  await advancedContent.waitFor();
});
