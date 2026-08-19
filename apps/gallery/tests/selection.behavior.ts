import { expect, test } from "@wabou/test";

test("selection controls expose semantic interaction state", async ({
  page,
}) => {
  await page.getByRole("button", { name: "Checkbox" }).click();

  const terms = page.getByRole("checkbox", {
    name: "Accept the terms and conditions",
  });
  await expect(terms).toBeUnchecked();
  await terms.click();
  await expect(terms).toBeChecked();
  await expect(
    page.getByRole("checkbox", { name: "Selected by default" }),
  ).toBeChecked();
  await expect(
    page.getByRole("checkbox", { name: "Some child items selected" }),
  ).toBeIndeterminate();

  await page.getByRole("button", { name: "Toggle" }).click();
  await expect(page.getByRole("button", { name: "Toggle bold" })).toBePressed();
  await expect(
    page.getByRole("button", { name: "Toggle italic" }),
  ).toBeUnpressed();

  await page.getByRole("button", { name: "Radio group" }).click();
  await page.getByRole("radiogroup", { name: "Subscription plan" }).waitFor();
  const free = page.getByRole("radio", { name: "Free — local projects" });
  const pro = page.getByRole("radio", { name: "Pro — team collaboration" });
  await expect(free).toBeUnchecked();
  await expect(pro).toBeChecked();
  await free.click();
  await expect(free).toBeChecked();
  await expect(pro).toBeUnchecked();

  await page.getByRole("button", { name: "Switch" }).click();
  const notifications = page.getByRole("switch", {
    name: "Desktop notifications",
  });
  await expect(notifications).toBeChecked();
  await notifications.click();
  await expect(notifications).toBeUnchecked();
  const experimental = page.getByRole("switch", {
    name: "Experimental renderer",
  });
  await expect(experimental).toBeUnchecked();
  await expect(experimental).toBeDisabled();
});
