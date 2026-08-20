import { expect, test } from "@wabou/test";

test("DirectoryPicker applies a deterministic native response", async ({
  page,
  effects,
}) => {
  await page.getByRole("button", { name: "Overview" }).wheel(4_000);
  await page.getByRole("button", { name: "System APIs" }).click();
  await page.getByRole("button", { name: "Browse directory" }).waitFor();

  effects.respond("dialogPickDirectory", ["/tmp/wabou-gallery"]);
  await page.getByRole("button", { name: "Browse directory" }).click();
  await expect(
    page.getByRole("textbox", { name: "Gallery directory" }),
  ).toHaveValue("/tmp/wabou-gallery");
});
