import { test } from "@wabou/test";

const routes = [
  { open: "Open Skills", page: "Skills catalog" },
  { open: "Open Folio Desk", page: "Folio Desk correspondence" },
  { open: "Open Tendra", page: "Tendra bid pipeline" },
  { open: "Open OceanView", page: "OceanView fleet command" },
  { open: "Open Signed", page: "Signed PDF workspace" },
  { open: "Open Omanta", page: "Omanta file browser" },
  { open: "Open DiskTree", page: "DiskTree explorer" },
] as const;

test("all visual shells route through one native application", async ({
  page,
}) => {
  for (const route of routes) {
    await page.getByRole("button", { name: route.open }).click();
    await page.getByRole("group", { name: route.page }).waitFor();
    await page.getByRole("button", { name: "Back to benchmarks" }).click();
    await page.getByRole("list", { name: "Benchmark applications" }).waitFor();
  }
});
