import { renderComponent } from "@wabou/test/component";
import {
  Table,
  TableBody,
  TableCaption,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
  Text,
} from "@wabou/ui";
import { expect, test } from "vitest";

test("exposes the complete shadcn table anatomy without a native host", () => {
  const screen = renderComponent(() => (
    <Table aria-label="Invoices" contentClass="min-w-3xl">
      <TableCaption>Recent invoices</TableCaption>
      <TableHeader>
        <TableRow>
          <TableHead>Invoice</TableHead>
          <TableHead>Total</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        <TableRow selected aria-label="INV-001">
          <TableCell>
            <Text>INV-001</Text>
          </TableCell>
          <TableCell>
            <Text>$250.00</Text>
          </TableCell>
        </TableRow>
      </TableBody>
    </Table>
  ));

  const table = screen.getByRole("table", { name: "Invoices" });
  expect(table.className).toContain("overflow-x-auto");
  expect(screen.getAllByRole("row")).toHaveLength(2);
  expect(screen.getAllByRole("columnheader")).toHaveLength(2);
  expect(screen.getAllByRole("cell")).toHaveLength(2);
  expect(screen.getByRole("row", { name: "INV-001" }).selected).toBe(true);
  expect(table.children[0]?.className).toContain("min-w-3xl");
});

test("applies hover feedback to passive table rows", () => {
  const screen = renderComponent(() => (
    <Table aria-label="Files">
      <TableBody>
        <TableRow aria-label="README">
          <TableCell>
            <Text>README</Text>
          </TableCell>
        </TableRow>
      </TableBody>
    </Table>
  ));
  const row = screen.getByRole("row", { name: "README" });
  expect(row.className).not.toContain("bg-control-hover");
  row.hover();
  expect(row.className).toContain("bg-control-hover");
  row.unhover();
  expect(row.className).not.toContain("bg-control-hover");
});
