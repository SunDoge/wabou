import { test } from "@wabou/test";

test("Rust plugin capability retains state across UI calls", async ({
  page,
}) => {
  const button = page.getByRole("button", { name: "Call Rust capability" });
  await button.click();
  await page.getByRole("label", { name: "Capability count: 1" }).waitFor();
  await button.click();
  await page.getByRole("label", { name: "Capability count: 2" }).waitFor();
});
